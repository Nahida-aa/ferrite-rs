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

impl Chunk {
    /// Decode a simplified Chunk from a play-payload slice.
    ///
    /// This implements a pragmatic, testable subset of the real ChunkData decoding:
    /// - It expects that block states for present sections are encoded as contiguous big-endian `u16` per block
    ///   (blocks_per_section * 2 bytes per present section). This is NOT the full Minecraft format,
    ///   but useful for unit tests and for a simple end-to-end rendering pipeline.
    pub fn decode_from_play_payload(
        buf: &mut bytes::BytesMut,
        primary_bit_mask: u32,
    ) -> Option<Self> {
        let blocks_per_section = SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH;
        let mut sections = Vec::with_capacity(16);

        for section_index in 0..16usize {
            if (primary_bit_mask >> section_index) & 1 == 1 {
                // section present: try to read blocks_per_section u16 values
                if buf.len() < blocks_per_section * 2 {
                    return None;
                }
                let mut sec = ChunkSection::new();
            } else {
                sections.push(ChunkSection::new());
            }
        }

        use crate::protocol::codec::{read_var_int, write_var_int};
        use bytes::Buf;

        let present_sections = (0..16)
            .filter(|i| ((primary_bit_mask >> i) & 1) == 1)
            .count();

        // Strategy 1: simple u16-per-block per present section
        let expected_bytes = present_sections * blocks_per_section * 2;
        if buf.len() == expected_bytes {
            let mut sections = Vec::with_capacity(16);
            for section_index in 0..16usize {
                if (primary_bit_mask >> section_index) & 1 == 1 {
                    let mut sec = ChunkSection::new();
                    for i in 0..blocks_per_section {
                        let hi = buf.split_to(1)[0];
                        let lo = buf.split_to(1)[0];
                        let raw = u16::from_be_bytes([hi, lo]);
                        sec.blocks[i] = BlockState::from_raw(raw);
                    }
                    sections.push(sec);
                } else {
                    sections.push(ChunkSection::new());
                }
            }
            return Some(Chunk { sections });
        }

        // Strategy 2: attempt palette + bit-packed long array per section (common modern format)
        let mut sections = Vec::with_capacity(16);
        for section_index in 0..16usize {
            if (primary_bit_mask >> section_index) & 1 == 1 {
                // read palette length
                let palette_len = read_var_int(buf)? as usize;
                let mut palette = Vec::with_capacity(palette_len);
                for _ in 0..palette_len {
                    let entry = read_var_int(buf)? as u16;
                    palette.push(entry);
                }

                // read data array length (number of longs)
                let long_count = read_var_int(buf)? as usize;
                if buf.len() < long_count * 8 {
                    return None;
                }
                let mut data = Vec::with_capacity(long_count);
                for _ in 0..long_count {
                    // read little-endian u64
                    let mut le = [0u8; 8];
                    for i in 0..8 {
                        le[i] = buf.split_to(1)[0];
                    }
                    data.push(u64::from_le_bytes(le));
                }

                // determine bits per block
                let bits_per_block = (palette_len.max(1) as f32).log2().ceil() as usize;
                let bits_per_block = bits_per_block.max(4);

                // unpack blocks
                let mut sec = ChunkSection::new();
                let mut bit_index = 0usize;
                for block_idx in 0..blocks_per_section {
                    let start_long = bit_index / 64;
                    let start_offset = bit_index % 64;
                    let mut value: u64;
                    if start_long >= data.len() {
                        return None;
                    }
                    if start_offset + bits_per_block <= 64 {
                        value = (data[start_long] >> start_offset) & ((1u64 << bits_per_block) - 1);
                    } else {
                        // spans two longs
                        let low = data[start_long] >> start_offset;
                        let high = if start_long + 1 < data.len() {
                            data[start_long + 1]
                                & ((1u64 << (bits_per_block - (64 - start_offset))) - 1)
                        } else {
                            0
                        };
                        value = (high << (64 - start_offset)) | low;
                        value &= (1u64 << bits_per_block) - 1;
                    }
                    bit_index += bits_per_block;

                    let palette_index = value as usize;
                    let raw = if palette_index < palette.len() {
                        palette[palette_index]
                    } else {
                        0u16
                    };
                    sec.blocks[block_idx] = BlockState::from_raw(raw);
                }
                sections.push(sec);
            } else {
                sections.push(ChunkSection::new());
            }
        }

        Some(Chunk { sections })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;

    #[test]
    fn decode_simple_chunk() {
        // build payload for only section 0 present, fill with raw value 7
        let blocks_per_section = SECTION_HEIGHT * CHUNK_WIDTH * CHUNK_WIDTH;
        let mut payload = BytesMut::new();
        for _ in 0..blocks_per_section {
            payload.extend_from_slice(&7u16.to_be_bytes());
        }
        let mask = 1u32; // only section 0 present
        let chunk = Chunk::decode_from_play_payload(&mut payload, mask).expect("decode");
        assert_eq!(chunk.sections.len(), 16);
        // section 0 should be filled with 7
        for &b in chunk.sections[0].blocks.iter() {
            assert_eq!(b.raw(), 7u16);
        }
        // other sections should be air
        for sec in chunk.sections.iter().skip(1) {
            for &b in sec.blocks.iter() {
                assert_eq!(b.raw(), BlockState::AIR.raw());
            }
        }
    }
}
