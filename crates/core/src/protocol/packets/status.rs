use bytes::{Buf, BufMut, BytesMut};

use crate::protocol::codec::read_string;

/// C→S, Status state, ID 0x00
pub struct StatusRequest;

impl StatusRequest {
    pub const ID: i32 = 0x00;

    pub fn encode(&self) -> BytesMut {
        BytesMut::new()
    }
}

/// S→C, Status state, ID 0x00
pub struct StatusResponse {
    pub json: String,
}

impl StatusResponse {
    pub const ID: i32 = 0x00;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        let json = read_string(buf, 32767)?;
        Some(Self { json })
    }
}

/// C→S, Status state, ID 0x01
pub struct PingRequest {
    pub payload: i64,
}

impl PingRequest {
    pub const ID: i32 = 0x01;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        buf.put_i64(self.payload);
        buf
    }
}

/// S→C, Status state, ID 0x01
pub struct PongResponse {
    pub payload: i64,
}

impl PongResponse {
    pub const ID: i32 = 0x01;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        if buf.len() < 8 {
            return None;
        }
        Some(Self {
            payload: buf.get_i64(),
        })
    }
}
