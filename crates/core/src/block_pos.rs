use glam::IVec3;

use crate::direction::Direction;
use crate::vec3i::Vec3i;

/// Block coordinate (integer)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockPos(pub i32, pub i32, pub i32);

// Java: PACKED_HORIZONTAL_LENGTH = 1 + Mth.log2(Mth.smallestEncompassingPowerOfTwo(30000000))
// 30000000 rounds up to 2^25, so log2 = 25, +1 = 26
const PACKED_HORIZONTAL_LENGTH: i32 = 26;
const PACKED_Y_LENGTH: i32 = 64 - 2 * PACKED_HORIZONTAL_LENGTH; // 12
const PACKED_X_MASK: i64 = (1i64 << PACKED_HORIZONTAL_LENGTH) - 1;
const PACKED_Y_MASK: i64 = (1i64 << PACKED_Y_LENGTH) - 1;
const PACKED_Z_MASK: i64 = (1i64 << PACKED_HORIZONTAL_LENGTH) - 1;
const BLOCK_Y_OFFSET: i32 = 0;
const BLOCK_Z_OFFSET: i32 = PACKED_Y_LENGTH;
const BLOCK_X_OFFSET: i32 = PACKED_Y_LENGTH + PACKED_HORIZONTAL_LENGTH;

impl BlockPos {
    pub const ZERO: BlockPos = BlockPos(0, 0, 0);
    pub const MAX_HORIZONTAL_COORDINATE: i32 = (1i32 << PACKED_HORIZONTAL_LENGTH) / 2 - 1;

    pub fn containing(x: f64, y: f64, z: f64) -> Self {
        Self(x.floor() as i32, y.floor() as i32, z.floor() as i32)
    }

    pub fn min(a: BlockPos, b: BlockPos) -> BlockPos {
        Self(a.0.min(b.0), a.1.min(b.1), a.2.min(b.2))
    }

    pub fn max(a: BlockPos, b: BlockPos) -> BlockPos {
        Self(a.0.max(b.0), a.1.max(b.1), a.2.max(b.2))
    }

    pub fn as_long(&self) -> i64 {
        Self::pack_long(self.0, self.1, self.2)
    }

    pub fn pack_long(x: i32, y: i32, z: i32) -> i64 {
        ((x as i64 & PACKED_X_MASK) << BLOCK_X_OFFSET as u64)
            | ((y as i64 & PACKED_Y_MASK) << BLOCK_Y_OFFSET as u64)
            | ((z as i64 & PACKED_Z_MASK) << BLOCK_Z_OFFSET as u64)
    }

    pub fn x_from_packed(packed: i64) -> i32 {
        ((packed << (64 - BLOCK_X_OFFSET - PACKED_HORIZONTAL_LENGTH)) >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32
    }

    pub fn y_from_packed(packed: i64) -> i32 {
        ((packed << (64 - PACKED_Y_LENGTH)) >> (64 - PACKED_Y_LENGTH)) as i32
    }

    pub fn z_from_packed(packed: i64) -> i32 {
        ((packed << (64 - BLOCK_Z_OFFSET - PACKED_HORIZONTAL_LENGTH)) >> (64 - PACKED_HORIZONTAL_LENGTH)) as i32
    }

    pub fn from_packed(packed: i64) -> Self {
        Self(Self::x_from_packed(packed), Self::y_from_packed(packed), Self::z_from_packed(packed))
    }

    pub fn offset_packed(packed: i64, direction: Direction) -> i64 {
        Self::pack_long(
            Self::x_from_packed(packed) + direction.step_x(),
            Self::y_from_packed(packed) + direction.step_y(),
            Self::z_from_packed(packed) + direction.step_z(),
        )
    }

    pub fn at_y(&self, y: i32) -> Self {
        Self(self.0, y, self.2)
    }
}

impl From<BlockPos> for IVec3 {
    fn from(pos: BlockPos) -> Self {
        IVec3::new(pos.0, pos.1, pos.2)
    }
}

impl From<IVec3> for BlockPos {
    fn from(v: IVec3) -> Self {
        Self(v.x, v.y, v.z)
    }
}

impl Vec3i for BlockPos {
    fn x(&self) -> i32 { self.0 }
    fn y(&self) -> i32 { self.1 }
    fn z(&self) -> i32 { self.2 }
    fn with_x(&self, x: i32) -> Self { Self(x, self.1, self.2) }
    fn with_y(&self, y: i32) -> Self { Self(self.0, y, self.2) }
    fn with_z(&self, z: i32) -> Self { Self(self.0, self.1, z) }
}
