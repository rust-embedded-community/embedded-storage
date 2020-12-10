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
use heapless::{consts::*, Vec};
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

/// Transparent storage trait
pub trait ReadWriteStorage {
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
	fn try_erase(&mut self, from: Address, to: Address) -> nb::Result<(), Self::Error>;
}

/// NOR flash region trait.
pub trait NorFlashRegion {
	/// The range of possible addresses within the region.
	///
	/// (start_addr, end_addr)
	fn range(&self) -> (Address, Address);
	/// Maximum number of bytes that can be written at once.
	fn page_size(&self) -> usize;
	/// List of avalable erase sizes in this region.
	/// Should be sorted in ascending order.
	/// Currently limited to 5 sizes, but could be increased if necessary.
	fn erase_sizes(&self) -> Vec<usize, U5>;
}

/// NOR flash storage trait
pub trait NorFlash {
	/// An enumeration of storage errors
	type Error;
	/// Region type
	type Region: NorFlashRegion;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address, and reading until end address
	/// (`self.range().1`) or buffer length, whichever comes first.
	fn try_read(&mut self, address: Address, bytes: &mut [u8]) -> nb::Result<(), Self::Error>;

	/// Write a slice of data to the storage peripheral, starting the write
	/// operation at the given address.
	///
	/// Since this is done on a NOR flash all bytes are anded with the current
	/// content in the flash. This means no 0s can to turned into 1s this way.
	fn try_write(&mut self, address: Address, bytes: &[u8]) -> nb::Result<(), Self::Error>;

	/// Erase the given storage range, clearing all data within `[from..to]`.
	/// The given range will contain all 1s afterwards.
	///
	/// This should return an error if the range is not aligned to a proper
	/// erase resolution
	fn try_erase(&mut self, from: Address, to: Address) -> nb::Result<(), Self::Error>;

	/// Get all distinct memory reagions. These must not overlap, but can be disjoint.
	/// Most chips will return a single region, but some chips have regions with
	/// different erase sizes.
	/// Currently limited to 4 regions, but could be increased if necessary
	fn regions(&self) -> Vec<Self::Region, U4>;
}

/// ...
pub trait UniformNorFlash: NorFlash {
	/// The range of possible addresses within the peripheral.
	///
	/// (start_addr, end_addr)
	fn range(&self) -> (Address, Address) {
		self.regions()[0].range()
	}
	/// Maximum number of bytes that can be written at once.
	fn page_size(&self) -> usize {
		self.regions()[0].page_size()
	}
	/// List of avalable erase sizes in this region.
	/// Should be sorted in ascending order.
	/// Currently limited to 5 sizes, but could be increased if necessary.
	fn erase_sizes(&self) -> Vec<usize, U5> {
		self.regions()[0].erase_sizes()
	}
}
