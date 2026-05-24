use super::block_state_model::BlockStateModel;
use super::block_state_model_part::BlockStateModelPart;
use client_resources::model::sprite::material::Baked;
use client_resources::model::geometry::baked_quad::BakedQuad;
use ferrite_core::direction::Direction;

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
}

impl BlockStateModel for CubeBlockModel {
    fn collect_parts(&self, parts: &mut Vec<Box<dyn BlockStateModelPart>>) {
        parts.push(Box::new(self.clone()));
    }

    fn particle_material(&self) -> Baked {
        Baked::new(self.face_texture_name(1).to_string(), false) // up face
    }

    fn material_flags(&self) -> u32 {
        if self.transparent { 1 } else { 0 }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl BlockStateModelPart for CubeBlockModel {
    fn get_quads(&self, _direction: Option<Direction>) -> &[BakedQuad] {
        &[]
    }

    fn material_flags(&self) -> u32 {
        if self.transparent { 1 } else { 0 }
    }
}
