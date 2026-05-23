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
