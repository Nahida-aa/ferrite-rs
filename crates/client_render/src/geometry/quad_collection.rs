use std::collections::HashMap;

use super::baked_quad::BakedQuad;
use crate::block::dispatch::block_state_model::Direction;

pub struct QuadCollection {
    pub by_direction: HashMap<Direction, Vec<BakedQuad>>,
    pub unculled: Vec<BakedQuad>,
}

impl QuadCollection {
    pub fn new() -> Self {
        Self {
            by_direction: HashMap::new(),
            unculled: Vec::new(),
        }
    }

    pub fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad] {
        match direction {
            Some(dir) => self.by_direction.get(&dir).map_or(&[], |v| v.as_slice()),
            None => &self.unculled,
        }
    }
}
