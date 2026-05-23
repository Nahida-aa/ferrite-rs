use bytes::BytesMut;

use crate::protocol::codec::{read_string, write_string};

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

/// S→C, Config state, ID 0x03
pub struct FinishConfiguration;

impl FinishConfiguration {
    pub const ID: i32 = 0x03;
}

/// C→S, Config state, ID 0x03
pub struct FinishConfigurationAcknowledged;

impl FinishConfigurationAcknowledged {
    pub const ID: i32 = 0x03;

    pub fn encode(&self) -> BytesMut {
        BytesMut::new()
    }
}

/// C→S, Config state, ID 0x00
pub struct ClientInformation {
    pub locale: String,
    pub view_distance: i8,
    pub chat_mode: i32,
    pub chat_colors: bool,
    pub displayed_skin_parts: u8,
    pub main_hand: i32,
    pub enable_text_filtering: bool,
    pub allow_server_listings: bool,
    pub particle_status: i32,
}

impl ClientInformation {
    pub const ID: i32 = 0x00;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        write_string(&mut buf, &self.locale);
        buf.extend_from_slice(&[self.view_distance as u8]);
        crate::protocol::codec::write_var_int(&mut buf, self.chat_mode);
        buf.extend_from_slice(&[self.chat_colors as u8]);
        buf.extend_from_slice(&[self.displayed_skin_parts]);
        crate::protocol::codec::write_var_int(&mut buf, self.main_hand);
        buf.extend_from_slice(&[self.enable_text_filtering as u8]);
        buf.extend_from_slice(&[self.allow_server_listings as u8]);
        crate::protocol::codec::write_var_int(&mut buf, self.particle_status);
        buf
    }
}

/// S→C, Config state, ID 0x0E
pub struct ClientBoundKnownPacks;

impl ClientBoundKnownPacks {
    pub const ID: i32 = 0x0E;
}

/// C→S, Config state, ID 0x07
pub struct ServerBoundKnownPacks;

impl ServerBoundKnownPacks {
    pub const ID: i32 = 0x07;

    pub fn encode(&self) -> BytesMut {
        let mut buf = BytesMut::new();
        // VarInt count = 0 (empty list - accept server's packs)
        crate::protocol::codec::write_var_int(&mut buf, 0);
        buf
    }
}

/// S→C, Config state, ID 0x01 (plugin message)
pub struct ClientBoundPluginMessage {
    pub channel: String,
    pub data: bytes::BytesMut,
}

impl ClientBoundPluginMessage {
    pub const ID: i32 = 0x01;

    pub fn decode(buf: &mut BytesMut) -> Option<Self> {
        let channel = read_string(buf, 32767)?;
        let data = buf.clone();
        Some(Self { channel, data })
    }
}
