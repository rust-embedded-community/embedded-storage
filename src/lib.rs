//! # embedded-storage - A Storage Abstraction Layer for Embedded Systems
//!
//! Storage traits to allow on and off board storage devices to read and write
//! data.

#![doc(html_root_url = "https://docs.rs/embedded-storage/0.1.0")]
#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// Currently contains [`OverlapIterator`]
pub mod iter;
/// Technology specific traits for NOR Flashes
pub mod nor_flash;

/// A region denotes a contiguous piece of memory between two addresses.
pub trait Region {
	/// Start address of the region of `Self`
	fn start(&self) -> u32;

	/// End address of the region of `Self`
	fn end(&self) -> u32;

	/// Check if `address` is contained in the region of `Self`
	fn contains(&self, address: u32) -> bool {
		(address >= self.start()) && (address < self.end())
	}
}

/// Transparent read only storage trait
pub trait ReadStorage {
	/// An enumeration of storage errors
	type Error;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address offset, and reading `bytes.len()` bytes.
	///
	/// This should throw an error in case `bytes.len()` will be larger than
	/// `self.capacity() - offset`.
	fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error>;

	/// The capacity of the storage peripheral in bytes.
	fn capacity(&self) -> usize;
}

/// Transparent read/write storage trait
pub trait Storage: ReadStorage {
	/// Write a slice of data to the storage peripheral, starting the write
	/// operation at the given address offset (between 0 and `self.capacity()`).
	///
	/// **NOTE:**
	/// This function will automatically erase any pages necessary to write the given data,
	/// and might as such do RMW operations at an undesirable performance impact.
	fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error>;
}

/// The size in bytes of a single block read or written by a [`BlockDevice`].
pub const BLOCK_SIZE: usize = 512;

/// A single block which may be read or written by a [`BlockDevice`].
///
/// Also referred to as a sector in some contexts.
pub type Block = [u8; BLOCK_SIZE];

/// A device which can read and write whole numbers of blocks.
pub trait BlockDevice {
	/// The error type returned by methods on this trait.
	type Error;

	/// Returns the size of the device in blocks.
	fn block_count(&self) -> Result<usize, Self::Error>;

	/// Reads some number of blocks from the device, starting at `first_block_index`.
	///
	/// `first_block_index + blocks.len()` must not be greater than the size returned by
	/// `block_count`.
	fn read(&mut self, first_block_index: u64, blocks: &mut [Block]) -> Result<(), Self::Error>;

	/// Writes some number of blocks to the device, starting at `first_block_index`.
	///
	/// `first_block_index + blocks.len()` must not be greater than the size returned by
	/// `block_count`.
	fn write(&mut self, first_block_index: u64, blocks: &[Block]) -> Result<(), Self::Error>;
}
