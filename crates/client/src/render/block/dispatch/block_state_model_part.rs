use crate::render::geometry::baked_quad::BakedQuad;
use crate::resources::model::sprite::material::Baked;
use ferrite_core::direction::Direction;

/// Java 对照: net.minecraft.client.renderer.block.dispatch.BlockStateModelPart
pub trait BlockStateModelPart {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad];
    fn use_ambient_occlusion(&self) -> bool;
    fn particle_material(&self) -> Baked;
    fn material_flags(&self) -> u32;
}
