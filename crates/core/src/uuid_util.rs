use md5::{Md5, Digest};
use uuid::Uuid;

const UUID_PREFIX_OFFLINE_PLAYER: &str = "OfflinePlayer:";

pub fn create_offline_player_uuid(player_name: &str) -> Uuid {
    let input = format!("{}{}", UUID_PREFIX_OFFLINE_PLAYER, player_name);
    let mut hasher = Md5::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    let mut bytes: [u8; 16] = hash.into();

    bytes[6] = (bytes[6] & 0x0f) | 0x30;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;

    Uuid::from_bytes(bytes)
}
