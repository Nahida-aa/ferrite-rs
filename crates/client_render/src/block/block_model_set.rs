use super::block_state_model_set::BlockStateModelSet;
use super::dispatch::cube_block_model::CubeBlockModel;

pub struct BlockModelSet {
    pub models: BlockStateModelSet,
}

impl BlockModelSet {
    pub fn new() -> Self {
        Self {
            models: BlockStateModelSet::new(),
        }
    }

    pub fn get(&self, id: u16) -> Option<&CubeBlockModel> {
        self.models.get(id)
    }
}
