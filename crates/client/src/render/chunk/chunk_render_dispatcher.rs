use bevy::{
    app::{App, Plugin},
    asset::Assets,
    ecs::system::Resource,
    render::texture::Image,
};

use crate::render::{block::block_model_set::BlockModelSet, texture::texture_atlas::TextureAtlas};

#[derive(Resource)]
pub struct ChunkRenderRes {
    pub registry: BlockModelSet,
    pub atlas: TextureAtlas,
}

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        let registry = BlockModelSet::new();

        let atlas = {
            let mut images = app.world_mut().resource_mut::<Assets<Image>>();
            TextureAtlas::build(registry.textures(), &mut images)
        };

        app.insert_resource(ChunkRenderRes { registry, atlas });
    }
}
