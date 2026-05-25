use std::collections::HashMap;

use ferrite_core::block::BlockState;
use super::block_state_model_set::{BlockStateModelSet, build_default_block_state_model_set};
use super::model::block_model::{BlockModel, BlockStateModelWrapper};

/// Java 对照: net.minecraft.client.renderer.block.BlockModelSet
///
/// Unlike Java which lazily populates `blockModelByStateCache` via
/// `computeIfAbsent`, this implementation eagerly populates the cache
/// at construction time. This avoids interior mutability issues and
/// keeps `get(&self)` a simple read-only query.
///
/// Missing model fallback is handled by `BlockStateModelSet.missing_model`,
/// not here — same as Java where `BlockModelSet.createFallbackModel` delegates
/// to `this.fallback.get(blockState)` which returns `missingModel` when not found.
pub struct BlockModelSet {
    /// Java 对照: BlockStateModelSet fallback
    pub fallback: BlockStateModelSet,
    /// Java 对照: Map<BlockState, BlockModel> blockModelByStateCache
    block_model_by_state_cache: HashMap<BlockState, BlockModel>,
}

impl BlockModelSet {
    pub fn new() -> Self {
        let fallback = build_default_block_state_model_set();

        let block_model_by_state_cache = fallback
            .model_by_state
            .iter()
            .map(|(&state, model)| {
                (state, BlockModel::StateWrapper(BlockStateModelWrapper::new(model.clone())))
            })
            .collect();

        Self {
            fallback,
            block_model_by_state_cache,
        }
    }

    // Java 对照: BlockModelSet.get
    pub fn get(&self, state: BlockState) -> &BlockModel {
        self.block_model_by_state_cache
            .get(&state)
            .unwrap_or_else(|| {
                panic!("BlockModelSet: no model for state {:?}", state)
            })
    }

    pub fn textures(&self) -> &[&'static str] {
        &self.fallback.textures
    }
}
