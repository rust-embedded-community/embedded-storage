use crate::{iter::IterableByOverlaps, Region, Storage};

/// NOR flash trait.
pub trait NorFlash {
	/// An enumeration of storage errors
	type Error;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address, and reading `bytes.len()` bytes.
	///
	///	This should throw an error in case `bytes.len()` will be larger than
	/// the peripheral end address.
	fn try_read(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), Self::Error>;

	/// Erase the given storage range, clearing all data within `[from..to]`.
	/// The given range will contain all 1s afterwards.
	///
	/// This should return an error if the range is not aligned to a proper
	/// erase resolution
	/// Erases page at addr, sets it all to 0xFF
	/// If power is lost during erase, contents of the page are undefined.
	/// `from` and `to` must both be multiples of `erase_size()` and `from` <= `to`.
	fn try_erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error>;

	/// Writes data to addr, bitwise ANDing if there's already data written at that location,
	/// If power is lost during write, the contents of the written words are undefined.
	/// The rest of the page is guaranteed to be unchanged.
	/// It is not allowed to write to the same word twice.
	/// `address` and `bytes.len()` must both be multiples of `write_size()` and properly aligned.
	fn try_write(&mut self, address: u32, bytes: &[u8]) -> Result<(), Self::Error>;

	/// The erase granularity of the storage peripheral
	fn erase_size(&self) -> u32;

	/// The minumum write size of the storage peripheral
	fn write_size(&self) -> u32;

	/// The capacity of the peripheral in bytes.
	fn capacity(&self) -> u32;
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

///
pub struct RmwNorFlashStorage<S: NorFlash>(S);

// FIXME: Not sure how to do this correctly? Ideally we could have `const fn erase_size()` or some const generic?
const MAX_PAGE_SIZE: usize = 2048;

impl<S: NorFlash> RmwNorFlashStorage<S> {
	/// Instantiate a new generic `Storage` from a `NorFlash` peripheral
	pub fn new(nor_flash: S) -> Self {
		Self(nor_flash)
	}
}

struct Page {
	pub start: u32,
	pub size: u32,
}

impl Page {
	fn new(index: u32, size: u32) -> Self {
		Self {
			start: index * size,
			size,
		}
	}

	/// The end address of the page
	const fn end(&self) -> u32 {
		self.start + self.size as u32
	}
}

impl Region for Page {
	fn contains(&self, address: u32) -> bool {
		(self.start <= address) && (self.end() > address)
	}
}

impl<S: NorFlash> Storage for RmwNorFlashStorage<S> {
	type Error = S::Error;

	fn try_read(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
		// Nothing special to be done for reads
		self.0.try_read(address, bytes)
	}

	fn try_write(&mut self, address: u32, bytes: &[u8]) -> Result<(), Self::Error> {
		// Perform read/modify/write operations on the byte slice.
		let erase_size = self.0.erase_size();
		let last_page = (self.0.capacity() / erase_size) - 1;

		// `data` is the part of `bytes` contained within `page`,
		// and `addr` in the address offset of `page` + any offset into the page as requested by `address`
		for (data, page, addr) in (0..last_page)
			.map(move |i| Page::new(i, erase_size))
			.overlaps(bytes, address)
		{
			let merge_buffer = &mut [0u8; MAX_PAGE_SIZE][0..erase_size as usize];
			let offset_into_page = addr.saturating_sub(page.start) as usize;

			self.try_read(page.start, merge_buffer)?;

			// If we cannot write multiple times to the same page, we will have to erase it
			self.0.try_erase(page.start, page.end())?;
			merge_buffer
				.iter_mut()
				.skip(offset_into_page)
				.zip(data)
				.for_each(|(byte, input)| *byte = *input);
			self.0.try_write(page.start, merge_buffer)?;
		}
		Ok(())
	}

	fn capacity(&self) -> u32 {
		self.0.capacity()
	}
}

// FIXME: Requires specialization to take advantage of MultiwriteNorFlash?
// impl<S: MultiwriteNorFlash> Storage for RmwNorFlashStorage<S> {
// 	type Error = S::Error;

// 	fn try_read(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
// 		// Nothing special to be done for reads
// 		self.0.try_read(address, bytes)
// 	}

// 	fn try_write(&mut self, address: u32, bytes: &[u8]) -> Result<(), Self::Error> {
// 		// Perform read/modify/write operations on the byte slice.
// 		let erase_size = self.0.erase_size();
// 		let last_page = (self.0.capacity() / erase_size) - 1;

// 		// `data` is the part of `bytes` contained within `page`,
// 		// and `addr` in the address offset of `page` + any offset into the page as requested by `address`
// 		for (data, page, addr) in (0..last_page)
// 			.map(move |i| Page::new(i, erase_size))
// 			.overlaps(bytes, address)
// 		{
// 			let merge_buffer = &mut [0u8; MAX_PAGE_SIZE][0..erase_size as usize];
// 			let offset_into_page = addr.saturating_sub(page.start) as usize;

// 			self.try_read(page.start, merge_buffer)?;

// 			let rhs = &merge_buffer[offset_into_page..];
// 			let is_subset =
// 			 	data.len() < rhs.len() && data.iter().zip(rhs.iter()).all(|(a, b)| (*a | *b) == *b);

// 			// Check if we can write the data block directly, under the limitations imposed by NorFlash:
// 			// - We can only change 1's to 0's
// 			if is_subset {
// 				self.0.try_write(addr, data)?;
// 			} else {
// 				self.0.try_erase(page.start, page.end())?;
// 				merge_buffer
// 					.iter_mut()
// 					.skip(offset_into_page)
// 					.zip(data)
// 					.for_each(|(byte, input)| *byte = *input);
// 				self.0.try_write(page.start, merge_buffer)?;
// 			}
// 		}
// 		Ok(())
// 	}

// 	fn capacity(&self) -> u32 {
// 		self.0.capacity()
// 	}
// }
