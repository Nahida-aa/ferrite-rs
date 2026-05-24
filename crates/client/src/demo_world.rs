use ferrite_core::block::BlockState;
use ferrite_core::chunk::{CHUNK_WIDTH, Chunk, ChunkSection, SECTION_HEIGHT};

const W: usize = CHUNK_WIDTH;

fn solid_plane(_y: usize, id: u16) -> ChunkSection {
    let b = vec![BlockState::from_raw(id); SECTION_HEIGHT * W * W];
    ChunkSection { blocks: b }
}

fn air_section() -> ChunkSection {
    solid_plane(0, 0)
}

/// Fill a contiguous rectangle at (x0,z0)-(x1,z1) within a section with a block.
fn fill_rect(s: &mut [BlockState], x0: usize, x1: usize, z0: usize, z1: usize, y: usize, id: u16) {
    for z in z0..=z1 {
        for x in x0..=x1 {
            let idx = y * W * W + z * W + x;
            s[idx] = BlockState::from_raw(id);
        }
    }
}

/// Fill a 2D rectangle at a given Y layer within a section.
fn fill_rect_y(
    section: &mut ChunkSection,
    y: usize,
    x0: usize,
    z0: usize,
    x1: usize,
    z1: usize,
    id: u16,
) {
    assert!(y < SECTION_HEIGHT);
    fill_rect(&mut section.blocks, x0, x1, z0, z1, y, id);
}

fn fill_column(section: &mut ChunkSection, x: usize, z: usize, y0: usize, y1: usize, id: u16) {
    for y in y0..=y1 {
        let idx = y * W * W + z * W + x;
        section.blocks[idx] = BlockState::from_raw(id);
    }
}

/// Generates a demo chunk for the given chunk coordinates.
pub fn generate_demo_chunk(cx: i32, cz: i32) -> Option<Chunk> {
    if cx < -1 || cx > 1 || cz < -1 || cz > 1 {
        return None;
    }

    let mut sections: Vec<ChunkSection> = Vec::new();

    // We'll create sections from y=0 to y=80 (5 sections)
    // Ground sits at y=64-80 (section index 4)

    let mut s;

    // Section 0-3: y=0..64 — all stone (for depth)
    for _ in 0..4 {
        sections.push(solid_plane(0, 1)); // stone
    }

    // Section 4: y=64..80 — surface level
    s = air_section();
    // Ground layer: grass on top
    // y=0 in this section = global y=64
    // y=1 in this section = global y=65

    // Layer 0 (y=64): bedrock
    for z in 0..W {
        for x in 0..W {
            let idx = 0 * W * W + z * W + x;
            s.blocks[idx] = BlockState::from_raw(85); // bedrock
        }
    }
    // Layer 1-3 (y=65-67): stone
    for y in 1..4 {
        for z in 0..W {
            for x in 0..W {
                let idx = y * W * W + z * W + x;
                s.blocks[idx] = BlockState::from_raw(1); // stone
            }
        }
    }
    // Layer 4-5 (y=68-69): dirt
    for y in 4..6 {
        for z in 0..W {
            for x in 0..W {
                let idx = y * W * W + z * W + x;
                s.blocks[idx] = BlockState::from_raw(10); // dirt
            }
        }
    }
    // Layer 6 (y=70): grass block on top
    for z in 0..W {
        for x in 0..W {
            let idx = 6 * W * W + z * W + x;
            s.blocks[idx] = BlockState::from_raw(9); // grass_block (snowy=false)
        }
    }

    // Features on the surface (y=7 corresponds to y=71, one above grass)
    // Demarcation path: center lines
    if cx == 0 || cz == 0 {
        // Path blocks at the chunk boundaries — show cobblestone path
        if cx == 0 && cz == 0 {
            // Center chunk: path at x=0 and z=0 lines
            for y in 7..=7 {
                for z in 0..W {
                    fill_rect_y(&mut s, y, 0, z, 0, z, 14); // cobblestone border
                }
                for x in 0..W {
                    fill_rect_y(&mut s, y, x, 0, x, 0, 14);
                }
            }
        }
    }

    // Demo blocks: columns of different types at specific positions
    // Only in center chunk (0,0) to show variety
    if cx == 0 && cz == 0 {
        let pillar_positions: [(usize, usize, u16, &str); 24] = [
            (2, 2, 1, "stone"),
            (2, 5, 2139, "bricks"),
            (2, 8, 15, "oak_planks"),
            (2, 11, 16, "spruce_planks"),
            (2, 14, 2137, "gold_block"),
            (5, 2, 4340, "diamond_block"),
            (5, 5, 2107, "red_wool"),
            (5, 8, 2104, "blue_wool"),
            (5, 11, 2100, "gray_wool"),
            (5, 14, 562, "glass"),
            (8, 2, 136, "oak_log"),
            (8, 5, 139, "spruce_log"),
            (8, 8, 148, "acacia_log"),
            (8, 11, 151, "cherry_log"),
            (8, 14, 160, "mangrove_log"),
            (11, 2, 129, "gold_ore"),
            (11, 5, 131, "iron_ore"),
            (11, 8, 133, "coal_ore"),
            (11, 11, 23970, "copper_ore"),
            (11, 14, 563, "lapis_ore"),
            (14, 2, 6780, "stone_bricks"),
            (14, 5, 5958, "ice"),
            (14, 8, 6028, "netherrack"),
            (14, 11, 2093, "white_wool"),
        ];
        for (x, z, id, _name) in pillar_positions.iter() {
            fill_column(&mut s, *x, *z, 7, 9, *id); // 3 blocks tall
        }

        // Some terracotta colors on the surface
        let color_positions: [(usize, usize, u16); 1] = [
            (14, 14, 10165), // white terracotta
        ];
        for (x, z, id) in color_positions.iter() {
            fill_rect_y(&mut s, 7, *x, *z, *x + 1, *z + 1, *id);
        }
    }

    sections.push(s);

    // Section 5: y=80..96 — nothing (air) except maybe tall structures
    sections.push(air_section());

    Some(Chunk { sections })
}
