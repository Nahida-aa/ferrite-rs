use super::super::dispatch::block_state_model::BlockStateModel;

/// Java 对照: net.minecraft.client.renderer.block.model.BlockModel
///
/// In Java this is an interface with 6 implementations. Here we use an enum
/// for compile-time polymorphism (no vtable, no heap indirection).
///
/// Java implementations:
/// - EmptyBlockModel        → Empty
/// - BlockStateModelWrapper → StateWrapper
/// - SpecialBlockModelWrapper → TODO
/// - ConditionalBlockModel  → TODO
/// - CompositeBlockModel    → TODO
/// - SelectBlockModel       → TODO
#[derive(Clone)]
pub enum BlockModel {
    /// Java 对照: net.minecraft.client.renderer.block.model.EmptyBlockModel
    Empty,
    /// Java 对照: net.minecraft.client.renderer.block.model.BlockStateModelWrapper
    StateWrapper(BlockStateModelWrapper),
    // TODO: SpecialWrapper(SpecialBlockModelWrapper)
    // TODO: Conditional { on_true: Box<BlockModel>, on_false: Box<BlockModel>, property }
    // TODO: Composite { normal: Box<BlockModel>, custom: Box<BlockModel> }
    // TODO: Select { property, models: Map<T, BlockModel> }
}

/// Java 对照: net.minecraft.client.renderer.block.model.BlockStateModelWrapper
///
/// Wraps a BlockStateModel with optional tint sources and transformation.
#[derive(Clone)]
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

impl BlockModel {
    // Java 对照: BlockModel.update
    // TODO: pub fn update(&self, render_state, block_state, display_context, seed)

    /// Convenience: access the inner BlockStateModel, if this is a StateWrapper.
    /// Returns None for other variants.
    pub fn as_state_wrapper(&self) -> Option<&BlockStateModelWrapper> {
        match self {
            BlockModel::StateWrapper(w) => Some(w),
            _ => None,
        }
    }
}
