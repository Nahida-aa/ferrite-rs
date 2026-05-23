use bytes::BytesMut;

use crate::protocol::codec::read_string;

/// S→C, Config state, ID 0x07
pub struct RegistryData {
    pub data: String,
}

impl RegistryData {
    pub const ID: i32 = 0x07;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        let data = read_string(buf, usize::MAX)?;
        Some(Self { data })
    }
}

/// S→C, Config state, ID 0x12
pub struct FinishConfiguration;

impl FinishConfiguration {
    pub const ID: i32 = 0x12;
}

/// C→S, Config state, ID 0x02
pub struct FinishConfigurationAcknowledged;

impl FinishConfigurationAcknowledged {
    pub const ID: i32 = 0x02;

    pub fn encode(&self) -> BytesMut {
        BytesMut::new()
    }
}
