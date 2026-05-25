// Map namings are aligned with MC Java 26.1.2 (net.minecraft.world.level.ChunkPos)

use ferrite_core::block_pos::BlockPos;

/// Chunk coordinate (2D: x, z)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos(pub i32, pub i32);

impl ChunkPos {
    pub const ZERO: ChunkPos = ChunkPos(0, 0);

    pub fn pack(x: i32, z: i32) -> i64 {
        (x as i64 & 0xFFFF_FFFF) | ((z as i64 & 0xFFFF_FFFF) << 32)
    }

    pub fn x_from_packed(packed: i64) -> i32 {
        packed as i32
    }

    pub fn z_from_packed(packed: i64) -> i32 {
        (packed >> 32) as i32
    }

    pub fn from_packed(packed: i64) -> Self {
        Self(Self::x_from_packed(packed), Self::z_from_packed(packed))
    }

    pub fn from_block_pos(pos: BlockPos) -> Self {
        Self(pos.0 >> 4, pos.2 >> 4)
    }

    pub fn min_block_x(&self) -> i32 {
        self.0 << 4
    }

    pub fn min_block_z(&self) -> i32 {
        self.1 << 4
    }

    pub fn max_block_x(&self) -> i32 {
        (self.0 << 4) + 15
    }

    pub fn max_block_z(&self) -> i32 {
        (self.1 << 4) + 15
    }

    pub fn contains(&self, pos: BlockPos) -> bool {
        pos.0 >= self.min_block_x() && pos.2 >= self.min_block_z() && pos.0 <= self.max_block_x() && pos.2 <= self.max_block_z()
    }
}
