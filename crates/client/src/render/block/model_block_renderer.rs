// Map namings are aligned with MC Java 26.1.2 (net.minecraft.client.renderer.block.ModelBlockRenderer)

use ferrite_core::block::BlockState;
use ferrite_core::block_pos::BlockPos;
use ferrite_core::direction::Direction;
use ferrite_core::vec3i::Vec3i;

use crate::render::block::{
    block_and_tint_getter::BlockAndTintGetter,
    dispatch::block_state_model::BlockStateModel,
    model::block_model::BlockModel,
};
use crate::resources::model::geometry::baked_quad::BakedQuad;

/// Java 对照: net.minecraft.client.renderer.block.ModelBlockRenderer
///
/// Tesselates a single block's model into mesh vertex data, applying face culling.
/// Currently implements flat rendering (no ambient occlusion).
pub struct ModelBlockRenderer {
    cull: bool,
}

impl ModelBlockRenderer {
    pub fn new(cull: bool) -> Self {
        Self { cull }
    }

    /// Java 对照: ModelBlockRenderer.tesselateBlock
    ///
    /// Tesselates a block at the given section-relative position (x, y, z)
    /// into the output vertex buffers.
    pub fn tesselate_block(
        &self,
        output: &mut SectionMeshOutput,
        x: f32,
        y: f32,
        z: f32,
        level: &dyn BlockAndTintGetter,
        pos: BlockPos,
        block_state: BlockState,
        model: &BlockModel,
    ) {
        let block_state_model = match model {
            BlockModel::StateWrapper(wrapper) => &wrapper.model,
            BlockModel::Empty => return,
        };

        let parts = collect_parts(block_state_model);
        if parts.is_empty() {
            return;
        }

        tesselate_flat(output, x, y, z, &parts, level, block_state, pos, self.cull);
    }
}

/// Collect parts from a BlockStateModel.
/// Currently only SingleVariant exists.
fn collect_parts(model: &BlockStateModel) -> Vec<PartData> {
    match model {
        BlockStateModel::SingleVariant(sv) => {
            // For now, create a minimal PartData from the SingleVariant faces.
            // Real BakedQuad data would come from FaceBakery during model loading.
            // This is a placeholder that creates quads for each face direction.
            let directions = [
                Direction::Down, Direction::Up, Direction::North,
                Direction::South, Direction::West, Direction::East,
            ];
            let mut quads_by_dir: Vec<(Direction, Vec<BakedQuad>)> = Vec::new();
            for (i, &dir) in directions.iter().enumerate() {
                let tex_idx = sv.faces[i].texture;
                let tex_name = sv.texture_names.get(tex_idx).map_or("", |s| s.as_str());
                if tex_name.is_empty() {
                    continue;
                }
                if let Some(quad) = create_face_quad(dir, tex_name, sv.transparent) {
                    quads_by_dir.push((dir, vec![quad]));
                }
            }
            vec![PartData { quads_by_dir }]
        }
    }
}

/// Placeholder: create a unit face quad for the given direction.
/// In the real implementation, these come from FaceBakery during model baking.
fn create_face_quad(direction: Direction, texture_name: &str, transparent: bool) -> Option<BakedQuad> {
    use glam::Vec3;

    let (positions, uvs) = match direction {
        Direction::Down => (
            [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0)],
            [[0.0, 0.0], [0.0, 1.0], [1.0, 1.0], [1.0, 0.0]],
        ),
        Direction::Up => (
            [Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 1.0)],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        Direction::North => (
            [Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 0.0)],
            [[1.0, 0.0], [0.0, 0.0], [0.0, 1.0], [1.0, 1.0]],
        ),
        Direction::South => (
            [Vec3::new(0.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 1.0)],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        Direction::West => (
            [Vec3::new(0.0, 0.0, 0.0), Vec3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 1.0), Vec3::new(0.0, 1.0, 0.0)],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
        Direction::East => (
            [Vec3::new(1.0, 0.0, 1.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(1.0, 1.0, 0.0), Vec3::new(1.0, 1.0, 1.0)],
            [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
        ),
    };

    Some(BakedQuad {
        positions,
        uvs,
        direction,
        tint_index: -1,
        shade: true,
        light_emission: 0,
    })
}

/// Intermediate representation of a BlockStateModelPart's quad data.
struct PartData {
    quads_by_dir: Vec<(Direction, Vec<BakedQuad>)>,
}

/// Java 对照: ModelBlockRenderer.tesselateFlat
fn tesselate_flat(
    output: &mut SectionMeshOutput,
    x: f32,
    y: f32,
    z: f32,
    parts: &[PartData],
    level: &dyn BlockAndTintGetter,
    _state: BlockState,
    pos: BlockPos,
    cull: bool,
) {
    let all_directions = [
        Direction::Down, Direction::Up, Direction::North,
        Direction::South, Direction::West, Direction::East,
    ];

    for part in parts {
        // Culled quads (direction-specific)
        for &direction in &all_directions {
            let quads: Vec<&BakedQuad> = part.quads_by_dir.iter()
                .filter(|(d, _)| *d == direction)
                .flat_map(|(_, q)| q.iter())
                .collect();

            if quads.is_empty() {
                continue;
            }

            if cull && !should_render_face(level, _state, direction, pos) {
                continue;
            }

            for quad in quads {
                output.put_quad(x, y, z, quad);
            }
        }

        // Unculled quads (direction = None) — not yet implemented in our model data
    }
}

/// Java 对照: ModelBlockRenderer.shouldRenderFace
fn should_render_face(
    level: &dyn BlockAndTintGetter,
    _state: BlockState,
    direction: Direction,
    pos: BlockPos,
) -> bool {
    let neighbor = pos.relative(direction, 1);
    let neighbor_state = level.get_block_state(neighbor);
    // Simplified: render face if neighbor is air.
    // Java uses Block.shouldRenderFace which checks isSolidRender, etc.
    neighbor_state.is_air()
}

/// Output buffer for section mesh data.
/// Java 对照: BlockQuadOutput (simplified — we collect into Vecs for Bevy Mesh).
pub struct SectionMeshOutput {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
    idx_offset: u32,
}

impl SectionMeshOutput {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
            idx_offset: 0,
        }
    }

    /// Java 对照: BlockQuadOutput.put
    fn put_quad(&mut self, x: f32, y: f32, z: f32, quad: &BakedQuad) {
        let normal = match quad.direction {
            Direction::Down  => [0.0, -1.0, 0.0],
            Direction::Up    => [0.0,  1.0, 0.0],
            Direction::North => [0.0,  0.0, -1.0],
            Direction::South => [0.0,  0.0,  1.0],
            Direction::West  => [-1.0, 0.0, 0.0],
            Direction::East  => [1.0,  0.0, 0.0],
        };

        for i in 0..4 {
            let p = quad.positions[i];
            self.positions.push([x + p.x, y + p.y, z + p.z]);
            self.normals.push(normal);
            self.uvs.push(quad.uvs[i]);
        }

        // Two triangles per quad
        self.indices.push(self.idx_offset + 0);
        self.indices.push(self.idx_offset + 1);
        self.indices.push(self.idx_offset + 2);
        self.indices.push(self.idx_offset + 0);
        self.indices.push(self.idx_offset + 2);
        self.indices.push(self.idx_offset + 3);
        self.idx_offset += 4;
    }
}
