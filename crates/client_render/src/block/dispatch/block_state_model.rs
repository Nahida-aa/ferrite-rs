use crate::block::dispatch::block_state_model_part::BlockStateModelPart;

pub trait BlockStateModel {
    fn collect_parts(&self, parts: &mut Vec<Box<dyn BlockStateModelPart>>);
    fn particle_texture(&self) -> &str;
    fn material_flags(&self) -> u32;
}



pub const ALL_DIRECTIONS: [Direction; 6] = [
    Direction::Down,
    Direction::Up,
    Direction::North,
    Direction::South,
    Direction::West,
    Direction::East,
];

pub trait BlockStateModel {
    fn collect_parts(&self, parts: &mut Vec<Box<dyn BlockStateModelPart>>);
    fn particle_texture(&self) -> &str;
    fn material_flags(&self) -> u32;
}
