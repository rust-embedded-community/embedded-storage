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
	/// Check if `address` is contained in the region of `Self`
	fn contains(&self, address: u32) -> bool;
}

/// Provides a method to map implementation-specific behaviors to concrete error categories.
pub trait Error {
	/// Convert the error into a defined error type.
	fn kind(&self) -> ErrorKind;
}
impl Error for core::convert::Infallible {
	fn kind(&self) -> ErrorKind {
		match *self {}
	}
}

impl core::fmt::Display for ErrorKind {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::NotAligned => write!(f, "Arguments are not properly aligned"),
			Self::OutOfBounds => write!(f, "Arguments are out of bounds"),
			Self::Other => write!(f, "An implementation specific error occurred"),
		}
	}
}

/// Error kinds.
///
/// Implementations must map their error to those generic error kinds through the
/// [`ErrorType`] trait.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
	/// The arguments are not properly aligned.
	NotAligned,

	/// The arguments are out of bounds.
	OutOfBounds,

	/// Error specific to the implementation.
	Other,
}

/// Transparent read only storage trait
pub trait ReadStorage {
	/// An enumeration of storage errors
	type Error: Error;

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

impl<T: ReadStorage> ReadStorage for &mut T {
	type Error = T::Error;

	fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
		T::read(self, offset, bytes)
	}

	fn capacity(&self) -> usize {
		T::capacity(self)
	}
}

impl<T: Storage> Storage for &mut T {
	fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
		T::write(self, offset, bytes)
	}
}
