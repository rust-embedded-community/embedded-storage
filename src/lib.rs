//! # embedded-storage - A Storage Abstraction Layer for Embedded Systems
//!
//! Storage traits to allow on and off board storage devices to read and write
//! data.
//!
//! Implementation based on `Cuervo`s great work in
//! https://www.ecorax.net/as-above-so-below-1/ and
//! https://www.ecorax.net/as-above-so-below-2/

#![no_std]
#![deny(missing_docs)]
#![deny(unsafe_code)]

use core::ops::{Add, Sub};
use nb;

/// Currently contains [`OverlapIterator`]
pub mod iter;

/// An address denotes the read/write address of a single word.
#[derive(Default, Copy, Clone, Debug, PartialOrd, PartialEq, Eq, Ord)]
pub struct Address(pub u32);

impl Add<usize> for Address {
	type Output = Self;

	fn add(self, rhs: usize) -> Self::Output {
		Address(self.0 + rhs as u32)
	}
}

impl Add<isize> for Address {
	type Output = Self;

	fn add(self, rhs: isize) -> Self::Output {
		Address((self.0 as isize + rhs) as u32)
	}
}
impl Sub<usize> for Address {
	type Output = Self;

	fn sub(self, rhs: usize) -> Self::Output {
		Address(self.0 - rhs as u32)
	}
}

impl Sub<isize> for Address {
	type Output = Self;

	fn sub(self, rhs: isize) -> Self::Output {
		Address((self.0 as isize - rhs) as u32)
	}
}

impl Sub<Address> for Address {
	type Output = Self;

	fn sub(self, rhs: Address) -> Self::Output {
		Address(self.0 - rhs.0)
	}
}

/// A region denotes a contiguous piece of memory between two addresses.
pub trait Region {
	/// Check if `address` is contained in the region of `Self`
	fn contains(&self, address: Address) -> bool;
}

/// Storage trait
pub trait ReadWrite {
	/// An enumeration of storage errors
	type Error;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address, and reading until end address
	/// (`self.range().1`) or buffer length, whichever comes first.
	fn try_read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;

	/// Write a slice of data to the storage peripheral, starting the write
	/// operation at the given address.
	fn try_write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error>;

	/// The range of possible addresses within the peripheral.
	///
	/// (start_addr, end_addr)
	fn range(&self) -> (Address, Address);

	/// Erase the given storage range, clearing all data within `[from..to]`.
	///
	/// This should return an error if the range is not aligned to a proper
	/// erase resolution
	fn try_erase(&mut self, from: Address, to: Address) -> nb::Result<(), Self::Error>;
}
