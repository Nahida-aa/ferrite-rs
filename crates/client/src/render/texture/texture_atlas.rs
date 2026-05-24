use bevy::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;

use super::abstract_texture::AbstractTexture;
use super::texture_atlas_sprite::TextureAtlasSprite;

pub const LOCAL_ASSETS_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/assets");

pub struct TextureAtlas {
    pub inner: AbstractTexture,
    pub sprites: HashMap<String, TextureAtlasSprite>, // texturesByName
}

impl Deref for TextureAtlas {
    type Target = AbstractTexture;
    fn deref(&self) -> &AbstractTexture {
        &self.inner
    }
}

impl TextureAtlas {
    fn load_texture_rgba(
        texture_path: &str,
        archive: &mut Option<zip::ZipArchive<std::fs::File>>,
    ) -> Option<Vec<u8>> {
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
                if let Some(archive) = archive.as_mut() {
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

    pub fn build(texture_names: &[&str], assets: &mut Assets<Image>) -> Self {
        let mut archive = Self::open_jar_archive();
        let tex_count = texture_names.len();

        let mut loaded: Vec<Vec<u8>> = Vec::with_capacity(tex_count);
        for texture_path in texture_names {
            if texture_path.is_empty() {
                loaded.push(vec![0u8; 16 * 16 * 4]);
                continue;
            }
            match Self::load_texture_rgba(texture_path, &mut archive) {
                Some(data) => loaded.push(data),
                None => {
                    tracing::warn!("Texture not found: {texture_path}, using white fallback");
                    loaded.push(vec![255u8; 16 * 16 * 4]);
                }
            }
        }

        let tex_size: usize = 16;
        let per_row = (tex_count as f32).sqrt().ceil() as usize;
        let atlas_width = tex_size * per_row;
        let row_count = (tex_count + per_row - 1) / per_row;
        let atlas_height = tex_size * row_count;

        let atlas_data_len = atlas_width * atlas_height * 4;
        let mut atlas_data = vec![0u8; atlas_data_len];
        let mut sprites = HashMap::new();

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

            let u0 = x as f32 / atlas_width as f32;
            let v0 = y as f32 / atlas_height as f32;
            let u1 = (x + tex_size) as f32 / atlas_width as f32;
            let v1 = (y + tex_size) as f32 / atlas_height as f32;

            sprites.insert(
                texture_names[i].to_string(),
                TextureAtlasSprite {
                    name: texture_names[i].to_string(),
                    u0,
                    v0,
                    u1,
                    v1,
                },
            );
        }

        let image = bevy::render::texture::Image {
            data: atlas_data,
            texture_descriptor: bevy::render::render_resource::TextureDescriptor {
                label: Some("block_texture_atlas"),
                size: bevy::render::render_resource::Extent3d {
                    width: atlas_width as u32,
                    height: atlas_height as u32,
                    depth_or_array_layers: 1,
                },
                dimension: bevy::render::render_resource::TextureDimension::D2,
                format: bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                mip_level_count: 1,
                sample_count: 1,
                usage: bevy::render::render_resource::TextureUsages::TEXTURE_BINDING
                    | bevy::render::render_resource::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            sampler: bevy::render::texture::ImageSampler::Descriptor(
                bevy::render::render_resource::SamplerDescriptor {
                    label: Some("block_texture_atlas_sampler"),
                    mag_filter: bevy::render::render_resource::FilterMode::Nearest,
                    min_filter: bevy::render::render_resource::FilterMode::Nearest,
                    mipmap_filter: bevy::render::render_resource::FilterMode::Nearest,
                    address_mode_u: bevy::render::render_resource::AddressMode::Repeat,
                    address_mode_v: bevy::render::render_resource::AddressMode::Repeat,
                    address_mode_w: bevy::render::render_resource::AddressMode::Repeat,
                    ..Default::default()
                }
                .into(),
            ),
            ..Default::default()
        };

        let texture = assets.add(image);

        TextureAtlas {
            inner: AbstractTexture { texture },
            sprites,
        }
    }
}
