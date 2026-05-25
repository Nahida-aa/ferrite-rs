// Map namings are aligned with MC Java 26.1.2 (net.minecraft.client.renderer.chunk.SectionCompiler)

use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

use ferrite_core::block_pos::BlockPos;
use ferrite_core::chunk::{ChunkSection, SECTION_HEIGHT, CHUNK_WIDTH};

use crate::render::block::{
    block_and_tint_getter::BlockAndTintGetter,
    block_model_set::BlockModelSet,
    model::block_model::BlockModel,
    model_block_renderer::{ModelBlockRenderer, SectionMeshOutput},
};
use crate::render::chunk::render_section_region::RenderSectionRegion;

/// Java 对照: net.minecraft.client.renderer.chunk.SectionCompiler
pub struct SectionCompiler<'a> {
    pub registry: &'a BlockModelSet,
    renderer: ModelBlockRenderer,
}

impl<'a> SectionCompiler<'a> {
    pub fn new(registry: &'a BlockModelSet) -> Self {
        Self {
            registry,
            renderer: ModelBlockRenderer::new(true), // cull = true
        }
    }

    /// Java 对照: SectionCompiler.compile
    ///
    /// Compiles a single section (16×16×16) into a Bevy Mesh by iterating
    /// over every block position in the section, checking if it has a model,
    /// and tesselating visible faces.
    pub fn compile(
        &self,
        sections: &[ChunkSection],
        section_index: usize,
        section_origin: BlockPos,
    ) -> Mesh {
        let region = RenderSectionRegion::new(sections, section_index, section_origin);
        let mut output = SectionMeshOutput::new();

        // Java: for (BlockPos blockPos : BlockPos.betweenClosed(minPos, maxPos))
        let min_x = section_origin.0;
        let min_y = section_origin.1;
        let min_z = section_origin.2;

        for ly in 0..SECTION_HEIGHT {
            for lz in 0..CHUNK_WIDTH {
                for lx in 0..CHUNK_WIDTH {
                    let world_pos = BlockPos(min_x + lx as i32, min_y + ly as i32, min_z + lz as i32);
                    let block_state = region.get_block_state(world_pos);

                    if block_state.is_air() {
                        continue;
                    }

                    let block_model = self.registry.get(block_state);

                    // Java: blockState.getRenderShape() != RenderShape.MODEL → skip
                    // Currently Empty model means no rendering
                    if matches!(block_model, BlockModel::Empty) {
                        continue;
                    }

                    // Section-relative float position for vertex offset
                    let x = lx as f32;
                    let y = ly as f32;
                    let z = lz as f32;

                    self.renderer.tesselate_block(
                        &mut output,
                        x, y, z,
                        &region,
                        world_pos,
                        block_state,
                        block_model,
                    );
                }
            }
        }

        // Add section world offset to all positions
        let offset_x = section_origin.0 as f32;
        let offset_y = section_origin.1 as f32;
        let offset_z = section_origin.2 as f32;
        for pos in &mut output.positions {
            pos[0] += offset_x;
            pos[1] += offset_y;
            pos[2] += offset_z;
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, output.positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, output.normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, output.uvs);
        mesh.insert_indices(Indices::U32(output.indices));
        mesh
    }
}
