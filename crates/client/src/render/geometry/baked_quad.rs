use crate::block::dispatch::block_state_model::Direction;

pub struct BakedQuad {
    pub texture_name: String,
    pub direction: Direction,
    pub tint_index: i32,
    pub shade: bool,
}
