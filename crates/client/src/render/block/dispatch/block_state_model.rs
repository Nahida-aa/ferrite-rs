// use crate::block::dispatch::single_variant::CubeBlockModel;
// use client_resources::model::sprite::material::Baked;

use crate::{
    render::block::dispatch::single_variant::CubeBlockModel,
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
    SingleVariant(CubeBlockModel),
    // Java 对照: net.minecraft.client.renderer.block.dispatch.WeightedVariants
    // TODO: WeightedVariants(WeightedList<BlockStateModel>),
}

impl BlockStateModel {
    // Java 对照: BlockStateModel.collectParts
    pub fn collect_parts(&self, parts: &mut Vec<CubeBlockModel>) {
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
