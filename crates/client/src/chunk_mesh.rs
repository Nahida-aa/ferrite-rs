use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

fn block_raw_to_color(raw: u16) -> [f32; 4] {
    let c = match raw {
        // Stone-type blocks
        1 => [0.50, 0.50, 0.50],        // stone
        2 | 3 => [0.72, 0.56, 0.56],    // granite
        4 | 5 => [0.82, 0.82, 0.80],    // diorite
        6 | 7 => [0.45, 0.45, 0.45],    // andesite / polished_andesite
        14 => [0.35, 0.33, 0.30],       // cobblestone
        85 => [0.15, 0.15, 0.15],       // bedrock
        129 => [0.48, 0.46, 0.38],      // gold_ore
        130 => [0.35, 0.33, 0.28],      // deepslate_gold_ore
        131 => [0.50, 0.46, 0.40],      // iron_ore
        132 => [0.37, 0.35, 0.30],      // deepslate_iron_ore
        133 => [0.42, 0.42, 0.42],      // coal_ore
        134 => [0.32, 0.32, 0.32],      // deepslate_coal_ore
        23970 => [0.55, 0.40, 0.30],    // copper_ore
        25967 => [0.30, 0.30, 0.32],    // cobbled_deepslate
        22109 => [0.40, 0.38, 0.36],    // tuff
        23344 => [0.85, 0.83, 0.78],    // calcite
        25796 => [0.55, 0.50, 0.40],    // dripstone_block
        2400 => [0.12, 0.08, 0.18],     // obsidian
        2399 => [0.35, 0.40, 0.30],     // mossy_cobblestone
        6780 => [0.50, 0.50, 0.50],     // stone_bricks
        2139 => [0.68, 0.32, 0.22],     // bricks
        2137 => [0.90, 0.78, 0.18],     // gold_block
        2138 => [0.78, 0.80, 0.82],     // iron_block
        4340 => [0.22, 0.78, 0.78],     // diamond_block
        8449 => [0.18, 0.75, 0.30],     // emerald_block
        565 => [0.15, 0.25, 0.65],      // lapis_block
        11634 => [0.12, 0.12, 0.12],    // coal_block
        20475 => [0.20, 0.18, 0.15],    // netherite_block
        13566 => [0.55, 0.20, 0.10],    // magma_block
        6028 => [0.55, 0.15, 0.15],     // netherrack
        6029 => [0.38, 0.28, 0.22],     // soul_sand

        // Dirt / ground
        8 => [0.85, 0.85, 0.88],        // grass_block (snowy)
        9 => [0.30, 0.65, 0.20],        // grass_block
        10 => [0.50, 0.35, 0.22],       // dirt
        11 => [0.40, 0.28, 0.18],       // coarse_dirt
        12 | 13 => [0.32, 0.22, 0.15],  // podzol
        118 => [0.85, 0.78, 0.55],      // sand
        119..=122 => [0.78, 0.70, 0.50], // suspicious_sand
        123 => [0.80, 0.55, 0.30],      // red_sand
        124 => [0.50, 0.45, 0.42],      // gravel
        125..=128 => [0.45, 0.40, 0.38], // suspicious_gravel
        5977 => [0.60, 0.62, 0.65],     // clay
        25903 => [0.25, 0.50, 0.18],    // moss_block
        5959 => [0.88, 0.90, 0.92],     // snow_block

        // Ice / water
        5958 => [0.60, 0.78, 0.88],     // ice
        11635 => [0.70, 0.82, 0.90],    // packed_ice
        13964 => [0.40, 0.60, 0.78],    // blue_ice
        86..=101 => [0.20, 0.40, 0.70], // water
        102..=117 => [0.80, 0.30, 0.05],// lava

        // Wood planks
        15 => [0.62, 0.50, 0.32],       // oak_planks
        16 => [0.38, 0.30, 0.20],       // spruce_planks
        17 => [0.78, 0.72, 0.58],       // birch_planks
        18 => [0.65, 0.48, 0.30],       // jungle_planks
        19 => [0.75, 0.55, 0.30],       // acacia_planks
        20 => [0.80, 0.55, 0.55],       // cherry_planks
        21 => [0.30, 0.22, 0.12],       // dark_oak_planks
        25 => [0.70, 0.68, 0.55],       // pale_oak_planks
        26 => [0.55, 0.35, 0.25],       // mangrove_planks
        27 | 28 => [0.65, 0.60, 0.30],  // bamboo_planks / bamboo_mosaic

        // Logs (3 axis variants each)
        136..=138 => [0.45, 0.35, 0.22], // oak_log
        139..=141 => [0.32, 0.27, 0.18], // spruce_log
        142..=144 => [0.70, 0.65, 0.55], // birch_log
        145..=147 => [0.55, 0.40, 0.25], // jungle_log
        148..=150 => [0.60, 0.42, 0.28], // acacia_log
        151..=153 => [0.65, 0.40, 0.35], // cherry_log
        154..=156 => [0.28, 0.22, 0.10], // dark_oak_log
        157..=159 => [0.60, 0.55, 0.42], // pale_oak_log
        160..=162 => [0.48, 0.30, 0.20], // mangrove_log
        163 | 164 => [0.45, 0.28, 0.15], // mangrove_roots
        165..=167 => [0.40, 0.30, 0.20], // muddy_mangrove_roots
        168..=170 => [0.58, 0.55, 0.30], // bamboo_block

        // Stripped logs
        192..=194 => [0.55, 0.45, 0.28], // stripped_oak_log
        171..=173 => [0.38, 0.34, 0.22], // stripped_spruce_log
        174..=176 => [0.75, 0.70, 0.60], // stripped_birch_log
        177..=179 => [0.60, 0.48, 0.32], // stripped_jungle_log
        180..=182 => [0.68, 0.50, 0.32], // stripped_acacia_log
        183..=185 => [0.70, 0.48, 0.42], // stripped_cherry_log
        186..=188 => [0.35, 0.28, 0.15], // stripped_dark_oak_log
        189..=191 => [0.65, 0.60, 0.48], // stripped_pale_oak_log
        195..=197 => [0.52, 0.38, 0.25], // stripped_mangrove_log
        198..=200 => [0.62, 0.58, 0.35], // stripped_bamboo_block

        // Leaves
        278 => [0.25, 0.55, 0.15],      // oak_leaves
        306 => [0.20, 0.48, 0.12],      // spruce_leaves
        334 => [0.25, 0.58, 0.15],      // birch_leaves
        362 => [0.30, 0.55, 0.15],      // jungle_leaves
        390 => [0.28, 0.52, 0.12],      // acacia_leaves
        418 => [0.50, 0.30, 0.35],      // cherry_leaves (pinkish)
        446 => [0.18, 0.40, 0.10],      // dark_oak_leaves
        502 => [0.22, 0.55, 0.15],      // mangrove_leaves

        // Saplings / small plants
        29..=44 => [0.25, 0.55, 0.10],  // saplings

        // Wool
        2093 => [0.85, 0.85, 0.85],     // white_wool
        2094 => [0.80, 0.50, 0.20],     // orange_wool
        2095 => [0.70, 0.30, 0.55],     // magenta_wool
        2096 => [0.40, 0.60, 0.75],     // light_blue_wool
        2097 => [0.85, 0.85, 0.20],     // yellow_wool
        2098 => [0.45, 0.75, 0.20],     // lime_wool
        2099 => [0.80, 0.50, 0.55],     // pink_wool
        2100 => [0.35, 0.35, 0.35],     // gray_wool
        2101 => [0.55, 0.55, 0.55],     // light_gray_wool
        2102 => [0.15, 0.45, 0.55],     // cyan_wool
        2103 => [0.55, 0.25, 0.55],     // purple_wool
        2104 => [0.20, 0.30, 0.60],     // blue_wool
        2105 => [0.35, 0.20, 0.12],     // brown_wool
        2106 => [0.25, 0.50, 0.15],     // green_wool
        2107 => [0.70, 0.20, 0.15],     // red_wool
        2108 => [0.10, 0.10, 0.10],     // black_wool

        // Other common blocks
        560 => [0.72, 0.72, 0.42],      // sponge (yellow-green)
        562 => [0.55, 0.70, 0.78],      // glass
        578 => [0.78, 0.72, 0.55],      // sandstone
        579 => [0.75, 0.70, 0.52],      // chiseled_sandstone
        580 => [0.78, 0.72, 0.55],      // cut_sandstone
        2142 => [0.50, 0.35, 0.22],     // bookshelf (brown with slightly different tone)
        4341 => [0.50, 0.38, 0.25],     // crafting_table
        10165 => [0.62, 0.55, 0.48],    // white_terracotta
        11633 => [0.70, 0.48, 0.32],    // terracotta

        // Fallback: hash-based color for unknown blocks
        _ => {
            let h = (raw as u32).wrapping_mul(0x9E3779B9);
            let r = ((h >> 16) & 0xFF) as f32 / 255.0;
            let g = ((h >> 8) & 0xFF) as f32 / 255.0;
            let b = (h & 0xFF) as f32 / 255.0;
            [r * 0.5 + 0.25, g * 0.5 + 0.25, b * 0.5 + 0.25]
        }
    };
    [c[0], c[1], c[2], 1.0]
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
