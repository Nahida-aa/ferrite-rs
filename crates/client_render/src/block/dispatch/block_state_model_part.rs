use ferrite_core::direction::Direction;
use ferrite_core::direction::Direction;
use ferrite_core::direction::Direction;
use ferrite_core::direction::Direction;
use crate::geometry::baked_quad::BakedQuad;
use super::block_state_model::Direction;

pub trait BlockStateModelPart {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad];
    fn material_flags(&self) -> u32;
}
