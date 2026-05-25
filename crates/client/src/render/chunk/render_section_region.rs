// Map namings are aligned with MC Java 26.1.2 (net.minecraft.client.renderer.chunk.RenderSectionRegion)

use ferrite_core::block::BlockState;
use ferrite_core::block_pos::BlockPos;
use ferrite_core::chunk::{ChunkSection, SECTION_HEIGHT, CHUNK_WIDTH};

use crate::render::block::block_and_tint_getter::BlockAndTintGetter;

/// Provides block state access for a section and its neighbors.
/// Java: class RenderSectionRegion implements BlockAndTintGetter
pub struct RenderSectionRegion<'a> {
    /// Sections indexed [section_y], each 16x16x16.
    /// The target section is at index `section_index`.
    pub sections: &'a [ChunkSection],
    /// Index of the target section in the sections slice.
    pub section_index: usize,
    /// World origin of the target section (min block x, y, z).
    pub section_origin: BlockPos,
}

impl<'a> RenderSectionRegion<'a> {
    pub fn new(
        sections: &'a [ChunkSection],
        section_index: usize,
        section_origin: BlockPos,
    ) -> Self {
        Self { sections, section_index, section_origin }
    }

    /// Get block state at world position. Returns AIR for out-of-bounds.
    fn get_state_at(&self, pos: BlockPos) -> BlockState {
        let dx = pos.0 - self.section_origin.0;
        let dy = pos.1 - self.section_origin.1;
        let dz = pos.2 - self.section_origin.2;

        // Allow neighbor access: expand Y range by 1 section in each direction,
        // X/Z by 1 block. For simplicity, just do direct lookup within the
        // section bounds; neighbors beyond return AIR.
        if dx < 0 || dx >= CHUNK_WIDTH as i32 || dz < 0 || dz >= CHUNK_WIDTH as i32 {
            return BlockState::AIR;
        }
        if dy < 0 || dy >= (SECTION_HEIGHT * self.sections.len()) as i32 {
            return BlockState::AIR;
        }

        let si = dy as usize / SECTION_HEIGHT;
        let local_y = dy as usize % SECTION_HEIGHT;
        let idx = local_y * CHUNK_WIDTH * CHUNK_WIDTH + (dz as usize) * CHUNK_WIDTH + (dx as usize);
        self.sections[si].blocks[idx]
    }
}

impl BlockAndTintGetter for RenderSectionRegion<'_> {
    fn get_block_state(&self, pos: BlockPos) -> BlockState {
        self.get_state_at(pos)
    }

    fn get_light_emission(&self, _pos: BlockPos) -> i32 {
        0 // TODO: implement light emission lookup
    }
}
