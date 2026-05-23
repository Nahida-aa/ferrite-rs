use bytes::{Buf, BufMut, BytesMut};

/// S→C, Play state, ID 0x26
pub struct KeepAliveS2C {
    pub id: i64,
}

impl KeepAliveS2C {
    pub const ID: i32 = 0x26;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        let id = buf.get_i64();
        Some(Self { id })
    }
}

/// C→S, Play state, ID 0x17
pub struct KeepAliveC2S {
    pub id: i64,
}

impl KeepAliveC2S {
    pub const ID: i32 = 0x1B;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_i64(self.id);
        buf
    }
}
