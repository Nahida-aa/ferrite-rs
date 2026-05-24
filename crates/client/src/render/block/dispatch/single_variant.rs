use crate::resources::model::sprite::material::Baked;

use super::block_state_model::BlockStateModel;
// use client_resources::model::sprite::material::Baked;

#[derive(Clone, Copy)]
pub struct BlockFace {
    pub texture: usize,
}

#[derive(Clone, Copy)]
pub struct OverlayFace {
    pub texture: usize,
    pub side_only: bool,
}

#[derive(Clone)]
pub struct CubeBlockModel {
    pub faces: [BlockFace; 6],
    pub overlay: Option<OverlayFace>,
    pub transparent: bool,
    pub texture_names: Vec<String>,
    pub face_texture_names: [usize; 6],
}

impl CubeBlockModel {
    pub fn face_texture_name(&self, face_idx: usize) -> &str {
        self.texture_names
            .get(self.faces[face_idx].texture)
            .map_or("", |s| s.as_str())
    }

    pub fn particle_material(&self) -> Baked {
        Baked::new(self.face_texture_name(1).to_string(), false) // up face
    }

    pub fn material_flags(&self) -> u32 {
        if self.transparent { 1 } else { 0 }
    }

    /// Convenience: wrap as SingleVariant
    pub fn into_model(self) -> BlockStateModel {
        BlockStateModel::SingleVariant(self)
    }
}
