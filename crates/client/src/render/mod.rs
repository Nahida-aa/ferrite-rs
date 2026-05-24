pub mod atlas;
pub mod block_models;

use bevy::prelude::*;

use self::atlas::build_texture_atlas;
use self::block_models::BlockRegistry;
use crate::net_plugin::ChunkRenderRes;

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
