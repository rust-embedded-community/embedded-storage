use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A device which can read and write whole numbers of blocks.
///
/// Blocks are also referred to as sectors in some contexts.
pub trait BlockDevice<const BLOCK_SIZE: usize = 512> {
	/// The error type returned by methods on this trait.
	type Error;

	/// How aligned do we need the blocks to be?
	///
	/// See the [`aligned`] crate for more details.
	type Alignment: aligned::Alignment;

	/// Returns the size of the device in blocks.
	fn block_count(&self) -> Result<BlockCount, Self::Error>;

	/// Creates some stack allocated empty blocks, with suitable alignment.
	fn empty_blocks<const N: usize>(
		&self,
	) -> [aligned::Aligned<Self::Alignment, [u8; BLOCK_SIZE]>; N] {
		[aligned::Aligned([0u8; BLOCK_SIZE]); N]
	}

	/// Reads some blocks from the device, starting at `first_block_index`.
	///
	/// The buffer we read into must be suitably aligned. You can create a
	/// buffer like:
	///
	/// ```rust
	/// # use embedded_storage::block::{BlockDevice, BlockIdx};
	/// # fn example<T, const N: usize>(block_device: &mut T) -> Result<(), T::Error> where T: BlockDevice<N, Alignment = aligned::A4> {
	///
	/// let mut buffer = block_device.empty_blocks::<4>();
	/// block_device.read(&mut buffer[..], BlockIdx(0))?;
	///
	/// # Ok(())
	/// # }
	/// ```
	///
	/// You will get an error if you request more blocks than the block device
	/// has (i.e. if `first_block_index + blocks.len()` is greater than the size
	/// returned by `block_count`).
	fn read(
		&mut self,
		blocks: &mut [aligned::Aligned<Self::Alignment, [u8; BLOCK_SIZE]>],
		first_block_index: BlockIdx,
	) -> Result<(), Self::Error>;

	/// Writes some number of blocks to the device, starting at
	/// `first_block_index`.
	///
	/// The buffer we write out must be suitably aligned. You can create a
	/// buffer like:
	///
	/// ```rust
	/// # use embedded_storage::block::{BlockDevice, BlockIdx};
	/// # fn example<T, const N: usize>(block_device: &mut T) -> Result<(), T::Error> where T: BlockDevice<N, Alignment = aligned::A4> {
	///
	/// let mut buffer = block_device.empty_blocks::<4>();
	/// for block in buffer.iter_mut() {
	///    block.fill(0xCC);
	/// }
	/// block_device.write(&buffer[..], BlockIdx(0))?;
	///
	/// # Ok(())
	/// # }
	/// ```
	///
	/// You will get an error if you request more blocks than the block device
	/// has (i.e. if first_block_index + blocks.len() is greater than the size
	/// returned by block_count).
	fn write(
		&mut self,
		blocks: &[aligned::Aligned<Self::Alignment, [u8; BLOCK_SIZE]>],
		first_block_index: BlockIdx,
	) -> Result<(), Self::Error>;
}

/// The linear numeric address of a block (or sector).
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockIdx(pub u64);

impl BlockIdx {
	/// Creates an iterator from the current `BlockIdx` through the given number of blocks.
	pub fn range(self, num: BlockCount) -> BlockIter {
		BlockIter::new(self, self + BlockCount(num.0))
	}
}

impl From<BlockIdx> for u64 {
	fn from(value: BlockIdx) -> Self {
		value.0.into()
	}
}

impl Add<BlockCount> for BlockIdx {
	type Output = BlockIdx;

	fn add(self, rhs: BlockCount) -> BlockIdx {
		BlockIdx(self.0 + rhs.0)
	}
}

impl AddAssign<BlockCount> for BlockIdx {
	fn add_assign(&mut self, rhs: BlockCount) {
		self.0 += rhs.0
	}
}

impl Sub<BlockCount> for BlockIdx {
	type Output = BlockIdx;

	fn sub(self, rhs: BlockCount) -> BlockIdx {
		BlockIdx(self.0 - rhs.0)
	}
}

impl SubAssign<BlockCount> for BlockIdx {
	fn sub_assign(&mut self, rhs: BlockCount) {
		self.0 -= rhs.0
	}
}

/// A number of blocks (or sectors).
///
/// This may be added to a [`BlockIdx`] to get another `BlockIdx`.
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct BlockCount(pub u64);

impl Add<BlockCount> for BlockCount {
	type Output = BlockCount;

	fn add(self, rhs: BlockCount) -> BlockCount {
		BlockCount(self.0 + rhs.0)
	}
}

impl AddAssign<BlockCount> for BlockCount {
	fn add_assign(&mut self, rhs: BlockCount) {
		self.0 += rhs.0
	}
}

impl Sub<BlockCount> for BlockCount {
	type Output = BlockCount;

	fn sub(self, rhs: BlockCount) -> BlockCount {
		BlockCount(self.0 - rhs.0)
	}
}

impl SubAssign<BlockCount> for BlockCount {
	fn sub_assign(&mut self, rhs: BlockCount) {
		self.0 -= rhs.0
	}
}

/// An iterator returned from `Block::range`.
pub struct BlockIter {
	inclusive_end: BlockIdx,
	current: BlockIdx,
}

impl BlockIter {
	/// Creates a new `BlockIter`, from the given start block, through (and including) the given end
	/// block.
	pub const fn new(start: BlockIdx, inclusive_end: BlockIdx) -> BlockIter {
		BlockIter {
			inclusive_end,
			current: start,
		}
	}
}

impl Iterator for BlockIter {
	type Item = BlockIdx;
	fn next(&mut self) -> Option<Self::Item> {
		if self.current.0 <= self.inclusive_end.0 {
			let this = self.current;
			self.current += BlockCount(1);
			Some(this)
		} else {
			None
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn block_idx_addition() {
		let a = BlockIdx(100);
		let len = BlockCount(50);
		let b = a + len;
		assert_eq!(b, BlockIdx(150));
	}

	#[test]
	fn block_idx_subtraction() {
		let a = BlockIdx(100);
		let len = BlockCount(50);
		let b = a - len;
		assert_eq!(b, BlockIdx(50));
	}

	#[test]
	fn block_iter() {
		let mut block_iter = BlockIter::new(BlockIdx(10), BlockIdx(12));
		let expected = [
			Some(BlockIdx(10)),
			Some(BlockIdx(11)),
			Some(BlockIdx(12)),
			None,
		];
		let actual = [
			block_iter.next(),
			block_iter.next(),
			block_iter.next(),
			block_iter.next(),
		];
		assert_eq!(actual, expected);
	}
}
