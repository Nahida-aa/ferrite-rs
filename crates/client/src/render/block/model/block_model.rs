use super::super::dispatch::block_state_model::BlockStateModel;

/// Java 对照: net.minecraft.client.renderer.block.model.BlockStateModelWrapper
///
/// Wraps a BlockStateModel with optional tint sources and transformation.
/// In Java, BlockModel is an interface and BlockStateModelWrapper implements it.
/// Here we use a simple struct since there's only one implementation.
pub struct BlockStateModelWrapper {
    pub model: BlockStateModel,
    // TODO: tints: Vec<BlockTintSource>
    // TODO: transformation: Matrix4fc
}

impl BlockStateModelWrapper {
    // Java 对照: BlockStateModelWrapper(model, tints, IDENTITY)
    pub fn new(model: BlockStateModel) -> Self {
        Self { model }
    }
}
