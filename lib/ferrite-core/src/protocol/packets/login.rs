use bytes::BytesMut;

use crate::protocol::codec::{read_string, read_uuid, write_string, write_uuid};
use uuid::Uuid;

/// C→S, Login state, ID 0x00
pub struct LoginStart {
    pub username: String,
    pub uuid: Uuid,
}

impl LoginStart {
    pub const ID: i32 = 0x00;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        write_string(&mut buf, &self.username);
        write_uuid(&mut buf, self.uuid);
        buf
    }
}

/// S→C, Login state, ID 0x02
pub struct LoginSuccess {
    pub uuid: Uuid,
    pub username: String,
    pub properties: Vec<(String, String, Option<String>)>,
}

impl LoginSuccess {
    pub const ID: i32 = 0x02;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        let uuid = read_uuid(buf)?;
        let username = read_string(buf, 16)?;

        let prop_count = crate::protocol::codec::read_var_int(buf)? as usize;
        let mut properties = Vec::with_capacity(prop_count);
        for _ in 0..prop_count {
            let name = read_string(buf, 32767)?;
            let value = read_string(buf, 32767)?;
            let signature = if buf.len() > 0 {
                // Check if there's a boolean for has_signature
                // This is simplified - in practice need to parse properly
                None
            } else {
                None
            };
            properties.push((name, value, signature));
        }

        Some(Self {
            uuid,
            username,
            properties,
        })
    }
}

/// C→S, Login state, ID 0x03
pub struct LoginAcknowledged;

impl LoginAcknowledged {
    pub const ID: i32 = 0x03;

    pub fn encode(&self) -> BytesMut {
        BytesMut::new()
    }
}

/// S→C, Login state, ID 0x00
pub struct LoginDisconnect {
    pub reason: String,
}

impl LoginDisconnect {
    pub const ID: i32 = 0x00;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        let reason = read_string(buf, 32767)?;
        Some(Self { reason })
    }
}
