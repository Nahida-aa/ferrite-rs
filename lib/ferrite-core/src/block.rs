#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BlockState(u16);

impl BlockState {
    pub const AIR: Self = Self(0);
}
