use client_resources::model::sprite::material::Baked;
use crate::block::dispatch::block_state_model_part::BlockStateModelPart;

pub trait BlockStateModel: Send + Sync {
    fn collect_parts(&self, parts: &mut Vec<Box<dyn BlockStateModelPart>>);
    fn particle_material(&self) -> Baked;
    fn material_flags(&self) -> u32;
    fn as_any(&self) -> &dyn std::any::Any;
}
