use client_resources::model::geometry::baked_quad::BakedQuad;
use core::direction::Direction;

pub trait BlockStateModelPart {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad];
    fn material_flags(&self) -> u32;
}
