// Re-export from resources/model/geometry to maintain backward compatibility
// during migration. New code should use crate::resources::model::geometry directly.
pub use crate::resources::model::geometry::baked_quad::BakedQuad;
pub use crate::resources::model::geometry::quad_collection::QuadCollection;
