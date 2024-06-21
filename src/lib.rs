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
///
/// These traits are suitable for NOR flash backed memory, but given the wide range of flash
/// technologies, not all NOR flashes may implement all those traits, nor <!-- no pun intended -->
/// will every instance necessarily be backed by NOR flash. The documentation of the individual
/// traits and their members define the precise criteria for the applicability of the trait.
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
