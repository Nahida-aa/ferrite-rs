use crate::block::BlockState;

pub const SECTION_HEIGHT: usize = 16;
pub const CHUNK_WIDTH: usize = 16;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub sections: Vec<ChunkSection>,
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub blocks: Vec<BlockState>,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            blocks: vec![BlockState::AIR; SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH],
        }
    }
}
