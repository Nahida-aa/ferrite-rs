pub mod atlas;
pub mod block_models;

use bevy::prelude::*;

use self::atlas::{build_texture_atlas, TextureAtlasRes};
use self::block_models::BlockRegistry;

/// Combined resource to keep system parameter count under 16.
#[derive(Resource)]
pub struct ChunkRenderRes {
    pub registry: BlockRegistry,
    pub atlas: TextureAtlasRes,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let registry = BlockRegistry::new();

        let atlas = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            build_texture_atlas(&registry, &mut images)
        };

        app.insert_resource(ChunkRenderRes { registry, atlas });
    }
}
