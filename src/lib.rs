//! # embedded-storage - A Storage Abstraction Layer for Embedded Systems
//!
//! Storage traits to allow on and off board storage devices to read and write
//! data.

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// Currently contains [`OverlapIterator`]
pub mod iter;
/// Technology specific traits for NOR Flashes
pub mod nor_flash;

/// A region denotes a contiguous piece of memory between two addresses.
pub trait Region {
	/// Check if `address` is contained in the region of `Self`
	fn contains(&self, address: u32) -> bool;
}

/// Transparent read only storage trait
pub trait ReadStorage {
	/// An enumeration of storage errors
	type Error;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address, and reading until end address
	/// (`self.range().1`) or buffer length, whichever comes first.
	fn try_read(&mut self, address: u32, bytes: &mut [u8]) -> Result<(), Self::Error>;

	/// The capacity of the storage peripheral in bytes.
	fn capacity(&self) -> usize;
}

/// Transparent read/write storage trait
pub trait Storage: ReadStorage {
	/// Write a slice of data to the storage peripheral, starting the write
	/// operation at the given address.
	///
	/// **NOTE:**
	/// This function will automatically erase any pages necessary to write the given data,
	/// and might as such do RMW operations at an undesirable performance impact.
	///
	/// CONSIDERATIONS:
	/// - Should the address here be normalized (always start from zero?)
	fn try_write(&mut self, address: u32, bytes: &[u8]) -> Result<(), Self::Error>;
}
