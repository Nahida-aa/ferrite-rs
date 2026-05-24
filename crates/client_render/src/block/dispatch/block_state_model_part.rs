use client_resources::model::geometry::baked_quad::BakedQuad;
use ferrite_core::direction::Direction;

pub trait BlockStateModelPart {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad];
    fn material_flags(&self) -> u32;
}
