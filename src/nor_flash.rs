use crate::{iter::IterableByOverlaps, ErrorKind, ReadStorage, Region, Storage};

/// Read only NOR flash trait.
pub trait ReadNorFlash: ReadStorage {
	/// The minumum number of bytes the storage peripheral can read
	const READ_SIZE: usize;
}

/// Return whether a read operation is within bounds.
pub fn check_read<T: ReadNorFlash>(
	flash: &T,
	offset: u32,
	length: usize,
) -> Result<(), super::ErrorKind> {
	check_slice(flash, T::READ_SIZE, offset, length)
}

/// NOR flash trait.
pub trait NorFlash: ReadNorFlash + Storage {
	/// The minumum number of bytes the storage peripheral can write
	const WRITE_SIZE: usize;

	/// The minumum number of bytes the storage peripheral can erase
	const ERASE_SIZE: usize;

	/// Erase the given storage range, clearing all data within `[from..to]`.
	/// The given range will contain all 1s afterwards.
	///
	/// If power is lost during erase, contents of the page are undefined.
	///
	/// # Errors
	///
	/// Returns an error if the arguments are not aligned or out of bounds (the case where `to >
	/// from` is considered out of bounds). The implementation can use the [`check_erase`]
	/// helper function.
	fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error>;
}

/// Return whether an erase operation is aligned and within bounds.
pub fn check_erase<T: NorFlash>(flash: &T, from: u32, to: u32) -> Result<(), ErrorKind> {
	let (from, to) = (from as usize, to as usize);
	if from > to || to > flash.capacity() {
		return Err(ErrorKind::OutOfBounds);
	}
	if from % T::ERASE_SIZE != 0 || to % T::ERASE_SIZE != 0 {
		return Err(ErrorKind::NotAligned);
	}
	Ok(())
}

/// Return whether a write operation is aligned and within bounds.
pub fn check_write<T: NorFlash>(flash: &T, offset: u32, length: usize) -> Result<(), ErrorKind> {
	check_slice(flash, T::WRITE_SIZE, offset, length)
}

fn check_slice<T: ReadNorFlash>(
	flash: &T,
	align: usize,
	offset: u32,
	length: usize,
) -> Result<(), ErrorKind> {
	let offset = offset as usize;
	if length > flash.capacity() || offset > flash.capacity() - length {
		return Err(ErrorKind::OutOfBounds);
	}
	if offset % align != 0 || length % align != 0 {
		return Err(ErrorKind::NotAligned);
	}
	Ok(())
}

impl<T: ReadNorFlash> ReadNorFlash for &mut T {
	const READ_SIZE: usize = T::READ_SIZE;
}

impl<T: NorFlash> NorFlash for &mut T {
	const WRITE_SIZE: usize = T::WRITE_SIZE;
	const ERASE_SIZE: usize = T::ERASE_SIZE;

	fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
		T::erase(self, from, to)
	}
}

/// Marker trait for NorFlash relaxing the restrictions on `write`.
///
/// Writes to the same word twice are now allowed. The result is the logical AND of the
/// previous data and the written data. That is, it is only possible to change 1 bits to 0 bits.
///
/// If power is lost during write:
/// - Bits that were 1 on flash and are written to 1 are guaranteed to stay as 1
/// - Bits that were 1 on flash and are written to 0 are undefined
/// - Bits that were 0 on flash are guaranteed to stay as 0
/// - Rest of the bits in the page are guaranteed to be unchanged
pub trait MultiwriteNorFlash: NorFlash {}

struct Page {
	pub start: u32,
	pub size: usize,
}

impl Page {
	fn new(index: u32, size: usize) -> Self {
		Self {
			start: index * size as u32,
			size,
		}
	}

	/// The end address of the page
	const fn end(&self) -> u32 {
		self.start + self.size as u32
	}
}

impl Region for Page {
	/// Checks if an address offset is contained within the page
	fn contains(&self, address: u32) -> bool {
		(self.start <= address) && (self.end() > address)
	}
}

///
pub struct RmwNorFlashStorage<'a, S> {
	storage: S,
	merge_buffer: &'a mut [u8],
}

impl<'a, S> RmwNorFlashStorage<'a, S>
where
	S: NorFlash,
{
	/// Instantiate a new generic `Storage` from a `NorFlash` peripheral
	///
	/// **NOTE** This will panic if the provided merge buffer,
	/// is smaller than the erase size of the flash peripheral
	pub fn new(nor_flash: S, merge_buffer: &'a mut [u8]) -> Self {
		if merge_buffer.len() < S::ERASE_SIZE {
			panic!("Merge buffer is too small");
		}

		Self {
			storage: nor_flash,
			merge_buffer,
		}
	}
}

impl<'a, S> ReadStorage for RmwNorFlashStorage<'a, S>
where
	S: ReadNorFlash,
{
	type Error = S::Error;

	fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
		// Nothing special to be done for reads
		self.storage.read(offset, bytes)
	}

	fn capacity(&self) -> usize {
		self.storage.capacity()
	}
}

impl<'a, S> Storage for RmwNorFlashStorage<'a, S>
where
	S: NorFlash,
{
	fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
		// Perform read/modify/write operations on the byte slice.
		let last_page = self.storage.capacity() / S::ERASE_SIZE;

		// `data` is the part of `bytes` contained within `page`,
		// and `addr` in the address offset of `page` + any offset into the page as requested by `address`
		for (data, page, addr) in (0..last_page as u32)
			.map(move |i| Page::new(i, S::ERASE_SIZE))
			.overlaps(bytes, offset)
		{
			let offset_into_page = addr.saturating_sub(page.start) as usize;

			self.storage
				.read(page.start, &mut self.merge_buffer[..S::ERASE_SIZE])?;

			// If we cannot write multiple times to the same page, we will have to erase it
			self.storage.erase(page.start, page.end())?;
			self.merge_buffer[..S::ERASE_SIZE]
				.iter_mut()
				.skip(offset_into_page)
				.zip(data)
				.for_each(|(byte, input)| *byte = *input);
			self.storage
				.write(page.start, &self.merge_buffer[..S::ERASE_SIZE])?;
		}
		Ok(())
	}
}

///
pub struct RmwMultiwriteNorFlashStorage<'a, S> {
	storage: S,
	merge_buffer: &'a mut [u8],
}

impl<'a, S> RmwMultiwriteNorFlashStorage<'a, S>
where
	S: MultiwriteNorFlash,
{
	/// Instantiate a new generic `Storage` from a `NorFlash` peripheral
	///
	/// **NOTE** This will panic if the provided merge buffer,
	/// is smaller than the erase size of the flash peripheral
	pub fn new(nor_flash: S, merge_buffer: &'a mut [u8]) -> Self {
		if merge_buffer.len() < S::ERASE_SIZE {
			panic!("Merge buffer is too small");
		}

		Self {
			storage: nor_flash,
			merge_buffer,
		}
	}
}

impl<'a, S> ReadStorage for RmwMultiwriteNorFlashStorage<'a, S>
where
	S: ReadNorFlash,
{
	type Error = S::Error;

	fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
		// Nothing special to be done for reads
		self.storage.read(offset, bytes)
	}

	fn capacity(&self) -> usize {
		self.storage.capacity()
	}
}

impl<'a, S> Storage for RmwMultiwriteNorFlashStorage<'a, S>
where
	S: MultiwriteNorFlash,
{
	fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
		// Perform read/modify/write operations on the byte slice.
		let last_page = self.storage.capacity() / S::ERASE_SIZE;

		// `data` is the part of `bytes` contained within `page`,
		// and `addr` in the address offset of `page` + any offset into the page as requested by `address`
		for (data, page, addr) in (0..last_page as u32)
			.map(move |i| Page::new(i, S::ERASE_SIZE))
			.overlaps(bytes, offset)
		{
			let offset_into_page = addr.saturating_sub(page.start) as usize;

			self.storage
				.read(page.start, &mut self.merge_buffer[..S::ERASE_SIZE])?;

			let rhs = &self.merge_buffer[offset_into_page..S::ERASE_SIZE];
			let is_subset = data.iter().zip(rhs.iter()).all(|(a, b)| *a & *b == *a);

			// Check if we can write the data block directly, under the limitations imposed by NorFlash:
			// - We can only change 1's to 0's
			if is_subset {
				// Use `merge_buffer` as allocation for padding `data` to `WRITE_SIZE`
				let offset = addr as usize % S::WRITE_SIZE;
				let aligned_end = data.len() % S::WRITE_SIZE + offset + data.len();
				self.merge_buffer[..aligned_end].fill(0xff);
				self.merge_buffer[offset..offset + data.len()].copy_from_slice(data);
				self.storage
					.write(addr - offset as u32, &self.merge_buffer[..aligned_end])?;
			} else {
				self.storage.erase(page.start, page.end())?;
				self.merge_buffer[..S::ERASE_SIZE]
					.iter_mut()
					.skip(offset_into_page)
					.zip(data)
					.for_each(|(byte, input)| *byte = *input);
				self.storage
					.write(page.start, &self.merge_buffer[..S::ERASE_SIZE])?;
			}
		}
		Ok(())
	}
}
