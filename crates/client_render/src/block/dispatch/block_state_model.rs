use crate::block::dispatch::block_state_model_part::BlockStateModelPart;

pub trait BlockStateModel {
    fn collect_parts(&self, parts: &mut Vec<Box<dyn BlockStateModelPart>>);
    fn particle_texture(&self) -> &str;
    fn material_flags(&self) -> u32;
}
