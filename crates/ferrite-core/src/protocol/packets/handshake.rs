use bytes::{BufMut, BytesMut};

use crate::protocol::codec::{write_string, write_var_int};

/// C→S, Handshake state, ID 0x00
pub struct Handshake {
    pub protocol_version: i32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: i32, // 1=status, 2=login
}

impl Handshake {
    pub const ID: i32 = 0x00;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        write_var_int(&mut buf, self.protocol_version);
        write_string(&mut buf, &self.server_address);
        buf.put_u16(self.server_port);
        write_var_int(&mut buf, self.next_state);
        buf
    }
}
