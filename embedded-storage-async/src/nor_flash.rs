use core::future::Future;
use embedded_storage::nor_flash::ErrorType;

/// Read only NOR flash trait.
pub trait AsyncReadNorFlash: ErrorType {
	/// The minumum number of bytes the storage peripheral can read
	const READ_SIZE: usize;

	type ReadFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
	where
		Self: 'a;

	/// Read a slice of data from the storage peripheral, starting the read
	/// operation at the given address offset, and reading `bytes.len()` bytes.
	///
	/// # Errors
	///
	/// Returns an error if the arguments are not aligned or out of bounds. The implementation
	/// can use the [`check_read`] helper function.
	fn read<'a>(&'a mut self, offset: u32, bytes: &'a mut [u8]) -> Self::ReadFuture<'a>;

	/// The capacity of the peripheral in bytes.
	fn capacity(&self) -> usize;
}

/// NOR flash trait.
pub trait AsyncNorFlash: AsyncReadNorFlash {
	/// The minumum number of bytes the storage peripheral can write
	const WRITE_SIZE: usize;

	/// The minumum number of bytes the storage peripheral can erase
	const ERASE_SIZE: usize;

	type EraseFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
	where
		Self: 'a;

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
	fn erase<'a>(&'a mut self, from: u32, to: u32) -> Self::EraseFuture<'a>;

	type WriteFuture<'a>: Future<Output = Result<(), Self::Error>> + 'a
	where
		Self: 'a;
	/// If power is lost during write, the contents of the written words are undefined,
	/// but the rest of the page is guaranteed to be unchanged.
	/// It is not allowed to write to the same word twice.
	///
	/// # Errors
	///
	/// Returns an error if the arguments are not aligned or out of bounds. The implementation
	/// can use the [`check_write`] helper function.
	fn write<'a>(&'a mut self, offset: u32, bytes: &'a [u8]) -> Self::WriteFuture<'a>;
}
