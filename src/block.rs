use core::ops::{Add, AddAssign, Sub, SubAssign};

/// A device which can read and write whole numbers of blocks.
///
/// Blocks are also referred to as sectors in some contexts.
pub trait BlockDevice<const BLOCK_SIZE: usize = 512> {
	/// The error type returned by methods on this trait.
	type Error;

	/// Returns the size of the device in blocks.
	fn block_count(&self) -> Result<BlockCount, Self::Error>;

	/// Reads some number of blocks from the device, starting at `first_block_index`.
	///
	/// `first_block_index + blocks.len()` must not be greater than the size returned by
	/// `block_count`.
	fn read(
		&mut self,
		first_block_index: BlockIdx,
		blocks: &mut [[u8; BLOCK_SIZE]],
	) -> Result<(), Self::Error>;

	/// Writes some number of blocks to the device, starting at `first_block_index`.
	///
	/// `first_block_index + blocks.len()` must not be greater than the size returned by
	/// `block_count`.
	fn write(
		&mut self,
		first_block_index: BlockIdx,
		blocks: &[[u8; BLOCK_SIZE]],
	) -> Result<(), Self::Error>;
}

/// The linear numeric address of a block (or sector).
#[derive(Copy, Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
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
		if self.current.0 >= self.inclusive_end.0 {
			None
		} else {
			let this = self.current;
			self.current += BlockCount(1);
			Some(this)
		}
	}
}
