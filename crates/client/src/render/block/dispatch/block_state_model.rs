use crate::{
    render::block::dispatch::single_variant::SingleVariant,
    resources::model::sprite::material::Baked,
};

/// Java 对照: net.minecraft.client.renderer.block.dispatch.BlockStateModel
///
/// In Java this is an interface with SingleVariant and WeightedVariants
/// implementations. Here we use an enum for compile-time polymorphism
/// (no vtable, no heap indirection, no downcast needed).
#[derive(Clone)]
pub enum BlockStateModel {
    /// Java 对照: net.minecraft.client.renderer.block.dispatch.SingleVariant
    SingleVariant(SingleVariant),
    // Java 对照: net.minecraft.client.renderer.block.dispatch.WeightedVariants
    // TODO: WeightedVariants(WeightedList<BlockStateModel>),
}

impl BlockStateModel {
    // Java 对照: BlockStateModel.collectParts
    pub fn collect_parts(&self, parts: &mut Vec<SingleVariant>) {
        match self {
            BlockStateModel::SingleVariant(model) => parts.push(model.clone()),
        }
    }

    // Java 对照: BlockStateModel.particleMaterial
    pub fn particle_material(&self) -> Baked {
        match self {
            BlockStateModel::SingleVariant(model) => model.particle_material(),
        }
    }

    // Java 对照: BlockStateModel.materialFlags
    pub fn material_flags(&self) -> u32 {
        match self {
            BlockStateModel::SingleVariant(model) => model.material_flags(),
        }
    }
}
