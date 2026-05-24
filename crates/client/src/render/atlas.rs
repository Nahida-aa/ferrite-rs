use bevy::prelude::*;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::{
    AddressMode, Extent3d, FilterMode, SamplerDescriptor, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages,
};
use bevy::render::texture::ImageSampler;

use super::block_models::BlockRegistry;

#[derive(Resource)]
pub struct TextureAtlasRes {
    pub uvs: Vec<[f32; 4]>,
    pub atlas_handle: Handle<Image>,
}

const LOCAL_ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

fn load_texture_rgba(texture_path: &str, archive: &mut Option<zip::ZipArchive<std::fs::File>>) -> Option<Vec<u8>> {
    let local_path = format!("{LOCAL_ASSETS_DIR}/minecraft/textures/{texture_path}.png");
    match std::fs::read(&local_path) {
        Ok(data) => {
            let img = image::load_from_memory(&data).ok()?.to_rgba8();
            let raw = img.into_raw();
            if raw.len() == 16 * 16 * 4 {
                return Some(raw);
            }
        }
        Err(_) => {
            let jar_path = format!("assets/minecraft/textures/{texture_path}.png");
            if let Some(ref mut archive) = archive {
                if let Ok(mut entry) = archive.by_name(&jar_path) {
                    let mut buf = Vec::new();
                    std::io::Read::read_to_end(&mut entry, &mut buf).ok()?;
                    let img = image::load_from_memory(&buf).ok()?.to_rgba8();
                    let raw = img.into_raw();
                    if raw.len() == 16 * 16 * 4 {
                        return Some(raw);
                    }
                }
            }
        }
    }
    None
}

fn open_jar_archive() -> Option<zip::ZipArchive<std::fs::File>> {
    let home = std::env::var("HOME").ok()?;
    let candidates = [
        format!("{home}/.minecraft/versions/1.21.8/1.21.8.jar"),
        format!("{home}/.cache/mc-launcher-cli/jars/1.21.8.jar"),
    ];
    for c in &candidates {
        if let Ok(file) = std::fs::File::open(c) {
            if let Ok(archive) = zip::ZipArchive::new(file) {
                return Some(archive);
            }
        }
    }
    None
}

pub fn build_texture_atlas(
    registry: &BlockRegistry,
    assets: &mut Assets<Image>,
) -> TextureAtlasRes {
    let mut archive = open_jar_archive();

    let mut loaded: Vec<Vec<u8>> = Vec::with_capacity(registry.textures.len());
    for texture_path in &registry.textures {
        if texture_path.is_empty() {
            loaded.push(vec![0u8; 16 * 16 * 4]);
            continue;
        }
        match load_texture_rgba(texture_path, &mut archive) {
            Some(data) => loaded.push(data),
            None => {
                tracing::warn!("Texture not found: {texture_path}, using white fallback");
                loaded.push(vec![255u8; 16 * 16 * 4]);
            }
        }
    }

    pack_atlas(loaded, assets)
}

fn pack_atlas(
    loaded: Vec<Vec<u8>>,
    assets: &mut Assets<Image>,
) -> TextureAtlasRes {
    let tex_count = loaded.len();
    let tex_size: usize = 16;
    let per_row: usize = 16;
    let atlas_width = tex_size * per_row;
    let row_count = (tex_count + per_row - 1) / per_row;
    let atlas_height = tex_size * row_count;

    let atlas_data_len = atlas_width * atlas_height * 4;
    let mut atlas_data = vec![0u8; atlas_data_len];
    let mut uvs = Vec::with_capacity(tex_count);

    for (i, pixels) in loaded.iter().enumerate() {
        let row = i / per_row;
        let col = i % per_row;
        let x = col * tex_size;
        let y = row * tex_size;

        for ty in 0..tex_size {
            let src_start = ty * tex_size * 4;
            let src_end = src_start + tex_size * 4;
            let dst_start = (y + ty) * atlas_width * 4 + x * 4;
            atlas_data[dst_start..dst_start + (src_end - src_start)]
                .copy_from_slice(&pixels[src_start..src_end]);
        }

        let u_min = x as f32 / atlas_width as f32;
        let v_min = y as f32 / atlas_height as f32;
        let u_max = (x + tex_size) as f32 / atlas_width as f32;
        let v_max = (y + tex_size) as f32 / atlas_height as f32;
        uvs.push([u_min, v_min, u_max, v_max]);
    }

    let image = Image {
        data: atlas_data,
        texture_descriptor: TextureDescriptor {
            label: Some("block_texture_atlas"),
            size: Extent3d {
                width: atlas_width as u32,
                height: atlas_height as u32,
                depth_or_array_layers: 1,
            },
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        },
        sampler: ImageSampler::Descriptor(
            SamplerDescriptor {
                label: Some("block_texture_atlas_sampler"),
                mag_filter: FilterMode::Nearest,
                min_filter: FilterMode::Nearest,
                mipmap_filter: FilterMode::Nearest,
                address_mode_u: AddressMode::Repeat,
                address_mode_v: AddressMode::Repeat,
                address_mode_w: AddressMode::Repeat,
                ..Default::default()
            }
            .into(),
        ),
        asset_usage: RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
        ..Default::default()
    };

    let atlas_handle = assets.add(image);

    TextureAtlasRes {
        uvs,
        atlas_handle,
    }
}
