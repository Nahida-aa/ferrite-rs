// Map namings are aligned with MC Java 26.1.2 (net.minecraft.client.resources.model.geometry.BakedQuad)

use glam::Vec3;
use ferrite_core::direction::Direction;

/// Baked quad with vertex positions, UVs, direction, and material info.
/// Java: record BakedQuad(position0..3, packedUV0..3, direction, materialInfo)
#[derive(Clone, Debug)]
pub struct BakedQuad {
    pub positions: [Vec3; 4],
    pub uvs: [[f32; 2]; 4],
    pub direction: Direction,
    pub tint_index: i32,
    pub shade: bool,
    pub light_emission: i32,
}

impl BakedQuad {
    pub const VERTEX_COUNT: usize = 4;
}
