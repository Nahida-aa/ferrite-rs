use ferrite_core::block::BlockState;
use super::block_state_model_set::BlockStateModelSet;
use super::dispatch::block_state_model::BlockStateModel;

pub struct BlockModelSet {
    pub models: BlockStateModelSet,
}

impl BlockModelSet {
    pub fn new() -> Self {
        Self {
            models: BlockStateModelSet::new(),
        }
    }

    pub fn get(&self, state: BlockState) -> &dyn BlockStateModel {
        self.models.get(state)
    }

    pub fn textures(&self) -> &[&'static str] {
        &self.models.textures
    }
}
