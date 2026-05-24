use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

fn block_raw_to_color(raw: u16) -> [f32; 4] {
    if raw == 0 {
        return [0.0, 0.0, 0.0, 1.0];
    }
    let h = (raw as u32).wrapping_mul(0x9E3779B9);
    let r = ((h >> 16) & 0xFF) as f32 / 255.0;
    let g = ((h >> 8) & 0xFF) as f32 / 255.0;
    let b = (h & 0xFF) as f32 / 255.0;
    [
        r * 0.5 + 0.25,
        g * 0.5 + 0.25,
        b * 0.5 + 0.25,
        1.0,
    ]
}

// Greedy meshing for axis-aligned block chunks.
pub fn chunk_to_mesh(chunk: &ferrite_core::chunk::Chunk, chunk_x: i32, chunk_z: i32) -> Mesh {
    let size_x = ferrite_core::chunk::CHUNK_WIDTH as usize;
    let size_z = ferrite_core::chunk::CHUNK_WIDTH as usize;
    let size_y = chunk.sections.len() * ferrite_core::chunk::SECTION_HEIGHT;

    let base_x = chunk_x as f32 * 16.0;
    let base_z = chunk_z as f32 * 16.0;

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let air_raw = ferrite_core::block::BlockState::AIR.raw();
    let get_raw = |x: isize, y: isize, z: isize| -> u16 {
        if x < 0
            || x >= size_x as isize
            || y < 0
            || y >= size_y as isize
            || z < 0
            || z >= size_z as isize
        {
            return air_raw;
        }
        let si = (y as usize) / ferrite_core::chunk::SECTION_HEIGHT;
        let local_y = (y as usize) % ferrite_core::chunk::SECTION_HEIGHT;
        let idx = local_y * size_z * size_x + (z as usize) * size_x + (x as usize);
        chunk.sections[si].blocks[idx].raw()
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
            let mut mask: Vec<Option<(u16, bool)>> = vec![None; u_size * v_size];

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

                    let cur = get_raw(nx, ny, nz);
                    let prev = get_raw(cx, cy, cz);
                    let idx = v * u_size + u;
                    if prev != air_raw && cur == air_raw {
                        mask[idx] = Some((prev, true));
                    } else if prev == air_raw && cur != air_raw {
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

                    let (v0, v1, v2, v3, normal) = match axis {
                        0 => {
                            if pos_face {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x1, y0, z0),
                                    to_world(x1, y1, z1),
                                    to_world(x0, y1, z1),
                                    [1.0, 0.0, 0.0],
                                )
                            } else {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x0, y1, z1),
                                    to_world(x1, y1, z1),
                                    to_world(x1, y0, z0),
                                    [-1.0, 0.0, 0.0],
                                )
                            }
                        }
                        1 => {
                            if pos_face {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x1, y0, z0),
                                    to_world(x1, y1, z1),
                                    to_world(x0, y1, z1),
                                    [0.0, 1.0, 0.0],
                                )
                            } else {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x0, y1, z1),
                                    to_world(x1, y1, z1),
                                    to_world(x1, y0, z0),
                                    [0.0, -1.0, 0.0],
                                )
                            }
                        }
                        2 => {
                            if pos_face {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x0, y1, z1),
                                    to_world(x1, y1, z1),
                                    to_world(x1, y0, z0),
                                    [0.0, 0.0, 1.0],
                                )
                            } else {
                                (
                                    to_world(x0, y0, z0),
                                    to_world(x1, y0, z0),
                                    to_world(x1, y1, z1),
                                    to_world(x0, y1, z1),
                                    [0.0, 0.0, -1.0],
                                )
                            }
                        }
                        _ => unreachable!(),
                    };

                    let block_color = block_raw_to_color(id);
                    positions.push(v0);
                    positions.push(v1);
                    positions.push(v2);
                    positions.push(v3);
                    for _ in 0..4 {
                        normals.push(normal);
                        colors.push(block_color);
                        uvs.push([0.0, 0.0]);
                    }
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
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}
