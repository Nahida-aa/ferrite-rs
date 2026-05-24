#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct BlockState(u16);

impl BlockState {
    pub const AIR: Self = Self(0);

    pub fn from_raw(v: u16) -> Self {
        Self(v)
    }

    pub fn raw(&self) -> u16 {
        self.0
    }
}
