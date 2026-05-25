use crate::render::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::render::geometry::baked_quad::BakedQuad;
use crate::render::geometry::quad_collection::QuadCollection;
use crate::resources::model::sprite::material::Baked;
use ferrite_core::direction::Direction;

/// Java 对照: net.minecraft.client.resources.model.SimpleModelWrapper
pub struct SimpleModelWrapper {
    /// Java 对照: quads
    pub quads: QuadCollection,
    /// Java 对照: useAmbientOcclusion
    pub use_ambient_occlusion: bool,
    /// Java 对照: particleMaterial
    pub particle_material: Baked,
}

impl BlockStateModelPart for SimpleModelWrapper {
    fn get_quads(&self, direction: Option<Direction>) -> &[BakedQuad] {
        self.quads.get_quads(direction)
    }

    fn use_ambient_occlusion(&self) -> bool {
        self.use_ambient_occlusion
    }

    fn particle_material(&self) -> Baked {
        self.particle_material.clone()
    }

    fn material_flags(&self) -> u32 {
        // TODO: delegate to quads.materialFlags() like Java
        0
    }
}
