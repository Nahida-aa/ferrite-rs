use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};

pub fn chunk_to_mesh(chunk: &ferrite_core::chunk::Chunk, chunk_x: i32, chunk_z: i32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let blocks_per_section = ferrite_core::chunk::SECTION_HEIGHT
        * ferrite_core::chunk::CHUNK_WIDTH
        * ferrite_core::chunk::CHUNK_WIDTH;

    let base_x = chunk_x as f32 * 16.0;
    let base_z = chunk_z as f32 * 16.0;

    // Cube face definitions (4 verts per face)
    let face_verts: [[[f32; 3]; 4]; 6] = [
        // +X
        [
            [1.0, 0.0, 0.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 1.0, 0.0],
        ],
        // -X
        [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 1.0, 1.0],
        ],
        // +Y
        [
            [0.0, 1.0, 0.0],
            [1.0, 1.0, 0.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ],
        // -Y
        [
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
        // +Z
        [
            [0.0, 0.0, 1.0],
            [0.0, 1.0, 1.0],
            [1.0, 1.0, 1.0],
            [1.0, 0.0, 1.0],
        ],
        // -Z
        [
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0],
        ],
    ];

    let face_normals: [[f32; 3]; 6] = [
        [1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, -1.0, 0.0],
        [0.0, 0.0, 1.0],
        [0.0, 0.0, -1.0],
    ];

    let mut idx_offset: u32 = 0;

    for (si, section) in chunk.sections.iter().enumerate() {
        for local_y in 0..ferrite_core::chunk::SECTION_HEIGHT {
            for cz in 0..ferrite_core::chunk::CHUNK_WIDTH {
                for cx in 0..ferrite_core::chunk::CHUNK_WIDTH {
                    let idx = local_y
                        * ferrite_core::chunk::CHUNK_WIDTH
                        * ferrite_core::chunk::CHUNK_WIDTH
                        + cz * ferrite_core::chunk::CHUNK_WIDTH
                        + cx;
                    let block = section.blocks[idx];
                    if block == ferrite_core::block::BlockState::AIR {
                        continue;
                    }

                    let world_x = base_x + cx as f32;
                    let world_y =
                        si as f32 * ferrite_core::chunk::SECTION_HEIGHT as f32 + local_y as f32;
                    let world_z = base_z + cz as f32;

                    // For simplicity, add all faces (no face culling)
                    for f in 0..6usize {
                        for v in 0..4usize {
                            let pv = face_verts[f][v];
                            positions.push([world_x + pv[0], world_y + pv[1], world_z + pv[2]]);
                            normals.push(face_normals[f]);
                            uvs.push([0.0, 0.0]);
                        }
                        indices.push(idx_offset + 0);
                        indices.push(idx_offset + 1);
                        indices.push(idx_offset + 2);
                        indices.push(idx_offset + 0);
                        indices.push(idx_offset + 2);
                        indices.push(idx_offset + 3);
                        idx_offset += 4;
                    }
                }
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
