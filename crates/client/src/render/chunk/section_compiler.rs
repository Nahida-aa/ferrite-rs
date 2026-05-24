use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

// use crate::block::block_model_set::BlockModelSet;
// use crate::texture::texture_atlas::TextureAtlas;
use crate::render::{
    block::{block_model_set::BlockModelSet, dispatch::block_state_model::BlockStateModel},
    texture::texture_atlas::TextureAtlas,
};
use ferrite_core::{
    block::BlockState,
    chunk,
    direction::{DOWN, EAST, NORTH, SOUTH, UP, WEST},
};

/// Java 对照: net.minecraft.client.renderer.chunk.SectionCompiler
pub struct SectionCompiler<'a> {
    pub registry: &'a BlockModelSet,
    pub atlas: &'a TextureAtlas,
}

impl<'a> SectionCompiler<'a> {
    pub fn new(registry: &'a BlockModelSet, atlas: &'a TextureAtlas) -> Self {
        Self { registry, atlas }
    }

    /// Java 对照: SectionCompiler.compile
    pub fn compile(&self, chunk: &chunk::Chunk, chunk_x: i32, chunk_z: i32) -> Mesh {
        chunk_to_mesh(chunk, chunk_x, chunk_z, self.registry, self.atlas)
    }
}

fn face_for_axis(axis: usize, pos_face: bool) -> usize {
    match (axis, pos_face) {
        (0, true) => EAST,
        (0, false) => WEST,
        (1, true) => UP,
        (1, false) => DOWN,
        (2, true) => SOUTH,
        (2, false) => NORTH,
        _ => unreachable!(),
    }
}

pub fn chunk_to_mesh(
    chunk: &chunk::Chunk,
    chunk_x: i32,
    chunk_z: i32,
    registry: &BlockModelSet,
    atlas: &TextureAtlas,
) -> Mesh {
    // Java 对照: SectionCompiler.compile -> ModelBlockRenderer.tesselateBlock
    let size_x = chunk::CHUNK_WIDTH as usize;
    let size_z = chunk::CHUNK_WIDTH as usize;
    let size_y = chunk.sections.len() * chunk::SECTION_HEIGHT;

    let base_x = chunk_x as f32 * 16.0;
    let base_z = chunk_z as f32 * 16.0;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let air = BlockState::AIR;
    let get_state = |x: isize, y: isize, z: isize| -> BlockState {
        if x < 0
            || x >= size_x as isize
            || y < 0
            || y >= size_y as isize
            || z < 0
            || z >= size_z as isize
        {
            return air;
        }
        let si = (y as usize) / chunk::SECTION_HEIGHT;
        let local_y = (y as usize) % chunk::SECTION_HEIGHT;
        let idx = local_y * size_z * size_x + (z as usize) * size_x + (x as usize);
        chunk.sections[si].blocks[idx]
    };

    let mut idx_offset: u32 = 0;

    for axis in 0..3usize {
        let (u_size, v_size, w_size) = match axis {
            0 => (size_z, size_y, size_x),
            1 => (size_x, size_z, size_y),
            2 => (size_x, size_y, size_z),
            _ => unreachable!(),
        };

        for w in 0..=w_size {
            let mut mask: Vec<Option<(BlockState, bool)>> = vec![None; u_size * v_size];

            for v in 0..v_size {
                for u in 0..u_size {
                    let (cx, cy, cz) = match axis {
                        0 => (w as isize - 1, v as isize, u as isize),
                        1 => (u as isize, w as isize - 1, v as isize),
                        2 => (u as isize, v as isize, w as isize - 1),
                        _ => unreachable!(),
                    };
                    let (nx, ny, nz) = match axis {
                        0 => (w as isize, v as isize, u as isize),
                        1 => (u as isize, w as isize, v as isize),
                        2 => (u as isize, v as isize, w as isize),
                        _ => unreachable!(),
                    };

                    let cur = get_state(nx, ny, nz);
                    let prev = get_state(cx, cy, cz);
                    let idx = v * u_size + u;
                    if prev != air && cur == air {
                        mask[idx] = Some((prev, true));
                    } else if prev == air && cur != air {
                        mask[idx] = Some((cur, false));
                    }
                }
            }

            let mut u = 0usize;
            while u < u_size {
                let mut v = 0usize;
                while v < v_size {
                    let idx = v * u_size + u;
                    if mask[idx].is_none() {
                        v += 1;
                        continue;
                    }

                    let (id, pos_face) = mask[idx].unwrap();

                    let mut width = 1usize;
                    while u + width < u_size
                        && mask[v * u_size + (u + width)] == Some((id, pos_face))
                    {
                        width += 1;
                    }

                    let mut height = 1usize;
                    'height_loop: while v + height < v_size {
                        for k in 0..width {
                            if mask[(v + height) * u_size + (u + k)] != Some((id, pos_face)) {
                                break 'height_loop;
                            }
                        }
                        height += 1;
                    }

                    for hv in 0..height {
                        for hu in 0..width {
                            mask[(v + hv) * u_size + (u + hu)] = None;
                        }
                    }

                    let (x0, y0, z0, x1, y1, z1) = match axis {
                        0 => (
                            w as f32,
                            v as f32,
                            u as f32,
                            w as f32,
                            (v + height) as f32,
                            (u + width) as f32,
                        ),
                        1 => (
                            u as f32,
                            w as f32,
                            v as f32,
                            (u + width) as f32,
                            w as f32,
                            (v + height) as f32,
                        ),
                        2 => (
                            u as f32,
                            v as f32,
                            w as f32,
                            (u + width) as f32,
                            (v + height) as f32,
                            w as f32,
                        ),
                        _ => unreachable!(),
                    };

                    let to_world =
                        |x: f32, y: f32, z: f32| -> [f32; 3] { [base_x + x, y, base_z + z] };

                    let face_idx = face_for_axis(axis, pos_face);
                    let tex_idx = match registry.get(id).model {
                        BlockStateModel::SingleVariant(ref m) => m.faces[face_idx].texture,
                    };
                    let tex_name = registry.textures().get(tex_idx).copied().unwrap_or("");
                    let sprite = atlas.sprites.get(tex_name);
                    let (u_min, v_min, u_max, v_max) = sprite
                        .map(|s| (s.u0, s.v0, s.u1, s.v1))
                        .unwrap_or((0.0, 0.0, 1.0, 1.0));

                    let (v0, v1, v2, v3, normal, q_uv) = match (axis, pos_face) {
                        (0, true) => (
                            to_world(x0, y0, z0),
                            to_world(x0, y1, z0),
                            to_world(x0, y1, z1),
                            to_world(x0, y0, z1),
                            [1.0, 0.0, 0.0],
                            [u_min, v_min, u_min, v_max, u_max, v_max, u_max, v_min],
                        ),
                        (0, false) => (
                            to_world(x0, y0, z0),
                            to_world(x0, y0, z1),
                            to_world(x0, y1, z1),
                            to_world(x0, y1, z0),
                            [-1.0, 0.0, 0.0],
                            [u_min, v_min, u_max, v_min, u_max, v_max, u_min, v_max],
                        ),
                        (1, true) => (
                            to_world(x0, y0, z0),
                            to_world(x0, y0, z1),
                            to_world(x1, y0, z1),
                            to_world(x1, y0, z0),
                            [0.0, 1.0, 0.0],
                            [u_min, v_min, u_min, v_max, u_max, v_max, u_max, v_min],
                        ),
                        (1, false) => (
                            to_world(x0, y0, z0),
                            to_world(x1, y0, z0),
                            to_world(x1, y0, z1),
                            to_world(x0, y0, z1),
                            [0.0, -1.0, 0.0],
                            [u_min, v_min, u_max, v_min, u_max, v_max, u_min, v_max],
                        ),
                        (2, true) => (
                            to_world(x0, y0, z0),
                            to_world(x1, y0, z0),
                            to_world(x1, y1, z0),
                            to_world(x0, y1, z0),
                            [0.0, 0.0, 1.0],
                            [u_min, v_min, u_max, v_min, u_max, v_max, u_min, v_max],
                        ),
                        (2, false) => (
                            to_world(x0, y0, z0),
                            to_world(x0, y1, z0),
                            to_world(x1, y1, z0),
                            to_world(x1, y0, z0),
                            [0.0, 0.0, -1.0],
                            [u_min, v_min, u_min, v_max, u_max, v_max, u_max, v_min],
                        ),
                        _ => unreachable!(),
                    };

                    positions.push(v0);
                    positions.push(v1);
                    positions.push(v2);
                    positions.push(v3);
                    for _ in 0..4 {
                        normals.push(normal);
                    }
                    uvs.push([q_uv[0], q_uv[1]]);
                    uvs.push([q_uv[2], q_uv[3]]);
                    uvs.push([q_uv[4], q_uv[5]]);
                    uvs.push([q_uv[6], q_uv[7]]);
                    indices.push(idx_offset + 0);
                    indices.push(idx_offset + 1);
                    indices.push(idx_offset + 2);
                    indices.push(idx_offset + 0);
                    indices.push(idx_offset + 2);
                    indices.push(idx_offset + 3);
                    idx_offset += 4;

                    v += 1;
                }
                u += 1;
            }
        }
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, Default::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
