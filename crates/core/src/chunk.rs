use crate::block::BlockState;

pub const SECTION_HEIGHT: usize = 16;
pub const CHUNK_WIDTH: usize = 16;

#[derive(Debug, Clone)]
pub struct Chunk {
    pub sections: Vec<ChunkSection>,
}

#[derive(Debug, Clone)]
pub struct ChunkSection {
    pub blocks: Vec<BlockState>,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            blocks: vec![BlockState::AIR; SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH],
        }
    }
}

fn read_paletted_container_u64s(
    buf: &mut bytes::BytesMut,
    bits_per_entry: u8,
    entry_count: usize,
) -> Option<Vec<u64>> {
    if bits_per_entry == 0 {
        return Some(Vec::new());
    }
    let long_count = (entry_count * bits_per_entry as usize + 63) / 64;
    let mut data = Vec::with_capacity(long_count);
    for _ in 0..long_count {
        if buf.len() < 8 {
            return None;
        }
        let mut le = [0u8; 8];
        le.copy_from_slice(&buf.split_to(8));
        data.push(u64::from_le_bytes(le));
    }
    Some(data)
}

fn unpack_blocks(
    data: &[u64],
    bits_per_entry: u8,
    entry_count: usize,
    palette: Option<&[u16]>,
    is_global: bool,
) -> Vec<BlockState> {
    let mut blocks = Vec::with_capacity(entry_count);
    if bits_per_entry == 0 {
        let raw = palette.and_then(|p| p.first()).copied().unwrap_or(0);
        for _ in 0..entry_count {
            blocks.push(BlockState::from_raw(raw));
        }
        return blocks;
    }
    let bits = bits_per_entry as usize;
    let mask = if bits < 64 {
        (1u64 << bits) - 1
    } else {
        u64::MAX
    };
    for entry_idx in 0..entry_count {
        let bit_index = entry_idx * bits;
        let start_long = bit_index / 64;
        let start_offset = bit_index % 64;
        let value = if start_long >= data.len() {
            0
        } else if start_offset + bits <= 64 {
            (data[start_long] >> start_offset) & mask
        } else {
            let low = data[start_long] >> start_offset;
            let remaining = bits - (64 - start_offset);
            let high = if start_long + 1 < data.len() {
                data[start_long + 1] & ((1u64 << remaining) - 1)
            } else {
                0
            };
            (high << (64 - start_offset)) | low
        };
        let raw = if is_global {
            value as u16
        } else if let Some(pal) = palette {
            let idx = value as usize;
            if idx < pal.len() {
                pal[idx]
            } else {
                0
            }
        } else {
            0
        };
        blocks.push(BlockState::from_raw(raw));
    }
    blocks
}

impl Chunk {
    pub fn decode_from_play_payload(buf: &mut bytes::BytesMut) -> Option<Self> {
        let blocks_per_section = SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH;
        use crate::protocol::codec::read_var_int;
        use bytes::Buf;

        let mut sections = Vec::new();

        while buf.has_remaining() {
            // need at least 2 bytes for block_count
            if buf.len() < 2 {
                break;
            }

            let _block_count = buf.get_u16();

            // ---- Block states ----
            let block_bits = buf.get_u8();
            let (block_palette, is_global) = match block_bits {
                0 => {
                    let value = read_var_int(buf)? as u16;
                    (Some(vec![value]), false)
                }
                1..=8 => {
                    let len = read_var_int(buf)? as usize;
                    if len > 4096 {
                        return None;
                    }
                    let mut pal = Vec::with_capacity(len);
                    for _ in 0..len {
                        pal.push(read_var_int(buf)? as u16);
                    }
                    (Some(pal), false)
                }
                _ => {
                    (None, true)
                }
            };

            let block_data = read_paletted_container_u64s(buf, block_bits, blocks_per_section)?;
            let blocks = unpack_blocks(&block_data, block_bits, blocks_per_section, block_palette.as_deref(), is_global);

            // ---- Biomes ----
            let biome_bits = buf.get_u8();
            match biome_bits {
                0 => {
                    let _ = read_var_int(buf)?;
                }
                1..=15 => {
                    let len = read_var_int(buf)? as usize;
                    if len > 64 {
                        return None;
                    }
                    for _ in 0..len {
                        let _ = read_var_int(buf)?;
                    }
                }
                16 | _ => {}
            };

            let _biome_data =
                read_paletted_container_u64s(buf, biome_bits, 64)?;

            let mut sec = ChunkSection::new();
            sec.blocks = blocks;
            sections.push(sec);
        }

        if sections.is_empty() {
            None
        } else {
            Some(Chunk { sections })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};

    fn encode_section(
        buf: &mut BytesMut,
        block_count: u16,
        block_bits: u8,
        block_palette: &[u16],
        block_data: &[u64],
        biome_bits: u8,
        biome_palette: &[u16],
        biome_data: &[u64],
    ) {
        buf.put_u16(block_count);

        buf.put_u8(block_bits);
        match block_bits {
            0 => {
                crate::protocol::codec::write_var_int(buf, block_palette[0] as i32);
            }
            1..=15 => {
                crate::protocol::codec::write_var_int(buf, block_palette.len() as i32);
                for &p in block_palette {
                    crate::protocol::codec::write_var_int(buf, p as i32);
                }
            }
            _ => {}
        }
        for &v in block_data {
            buf.put_u64_le(v);
        }

        buf.put_u8(biome_bits);
        match biome_bits {
            0 => {
                crate::protocol::codec::write_var_int(buf, biome_palette[0] as i32);
            }
            1..=15 => {
                crate::protocol::codec::write_var_int(buf, biome_palette.len() as i32);
                for &p in biome_palette {
                    crate::protocol::codec::write_var_int(buf, p as i32);
                }
            }
            _ => {}
        }
        for &v in biome_data {
            buf.put_u64_le(v);
        }
    }

    #[test]
    fn decode_single_section_uniform() {
        let mut payload = BytesMut::new();

        // One section: uniform blocks of value 7, uniform biome of value 1
        let block_palette = vec![7u16];
        let biome_palette = vec![1u16];
        encode_section(
            &mut payload,
            4096,  // block_count
            0,     // block bits (uniform)
            &block_palette,
            &[],   // no block data for uniform
            0,     // biome bits (uniform)
            &biome_palette,
            &[],   // no biome data for uniform
        );

        let chunk = Chunk::decode_from_play_payload(&mut payload).expect("decode");
        assert_eq!(chunk.sections.len(), 1);
        for &b in chunk.sections[0].blocks.iter() {
            assert_eq!(b.raw(), 7u16);
        }
    }

    #[test]
    fn decode_two_sections_uniform() {
        let mut payload = BytesMut::new();

        encode_section(
            &mut payload,
            4096, 0, &[7u16], &[], 0, &[1u16], &[],
        );
        encode_section(
            &mut payload,
            4096, 0, &[42u16], &[], 0, &[1u16], &[],
        );

        let chunk = Chunk::decode_from_play_payload(&mut payload).expect("decode");
        assert_eq!(chunk.sections.len(), 2);
        assert_eq!(chunk.sections[0].blocks[0].raw(), 7u16);
        assert_eq!(chunk.sections[1].blocks[0].raw(), 42u16);
    }

    #[test]
    fn decode_palette_bitpacked() {
        let blocks_per_section = SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH;
        let bits_per_block = 4usize;
        let palette = vec![7u16, 42u16];

        // pack block indices: alternate 0, 1
        let mut indices = Vec::with_capacity(blocks_per_section);
        for i in 0..blocks_per_section {
            indices.push((i % 2) as u64);
        }
        let total_bits = blocks_per_section * bits_per_block;
        let long_count = (total_bits + 63) / 64;
        let mut data = vec![0u64; long_count];
        for (i, &idx) in indices.iter().enumerate() {
            let bit_pos = i * bits_per_block;
            let long_idx = bit_pos / 64;
            let offset = bit_pos % 64;
            data[long_idx] |= (idx as u64) << offset;
            if offset + bits_per_block > 64 {
                let overflow = offset + bits_per_block - 64;
                if long_idx + 1 < data.len() {
                    data[long_idx + 1] |= (idx as u64) >> (bits_per_block - overflow);
                }
            }
        }

        let mut payload = BytesMut::new();
        encode_section(
            &mut payload,
            2048,              // ~half non-air
            bits_per_block as u8,
            &palette,
            &data,
            0,                 // uniform biome
            &[1u16],
            &[],
        );

        let chunk = Chunk::decode_from_play_payload(&mut payload).expect("decode");
        assert_eq!(chunk.sections.len(), 1);
        for (i, &b) in chunk.sections[0].blocks.iter().enumerate() {
            let expected_raw = palette[(i % 2) as usize];
            assert_eq!(b.raw(), expected_raw);
        }
    }
}
