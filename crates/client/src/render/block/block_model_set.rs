use std::collections::HashMap;

use ferrite_core::block::BlockState;
use super::block_state_model_set::BlockStateModelSet;
use super::model::block_model::BlockStateModelWrapper;

/// Java 对照: net.minecraft.client.renderer.block.BlockModelSet
///
/// Unlike Java which lazily populates `blockModelByStateCache` via
/// `computeIfAbsent`, this implementation eagerly populates the cache
/// at construction time. This avoids interior mutability issues and
/// keeps `get(&self)` a simple read-only query.
pub struct BlockModelSet {
    /// Java 对照: BlockStateModelSet fallback
    pub fallback: BlockStateModelSet,
    /// Java 对照: Map<BlockState, BlockModel> blockModelByStateCache
    block_model_by_state_cache: HashMap<BlockState, BlockStateModelWrapper>,
    /// Fallback for missing block states (wraps fallback.missing_model)
    missing_block_model: BlockStateModelWrapper,
}

impl BlockModelSet {
    pub fn new() -> Self {
        let fallback = BlockStateModelSet::new();

        let missing_block_model = BlockStateModelWrapper::new(fallback.missing_model.clone());

        let block_model_by_state_cache = fallback
            .model_by_state
            .iter()
            .map(|(&state, model)| {
                (state, BlockStateModelWrapper::new(model.clone()))
            })
            .collect();

        Self {
            fallback,
            block_model_by_state_cache,
            missing_block_model,
        }
    }

    // Java 对照: BlockModelSet.get
    pub fn get(&self, state: BlockState) -> &BlockStateModelWrapper {
        self.block_model_by_state_cache
            .get(&state)
            .unwrap_or(&self.missing_block_model)
    }

    pub fn textures(&self) -> &[&'static str] {
        &self.fallback.textures
    }
}
