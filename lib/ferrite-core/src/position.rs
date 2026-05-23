use glam::IVec3;

/// Block coordinate (integer)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockPos(pub i32, pub i32, pub i32);

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

/// Chunk coordinate
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct ChunkPos(pub i32, pub i32);
