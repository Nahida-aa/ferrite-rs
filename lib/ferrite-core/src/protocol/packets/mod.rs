pub mod handshake;
pub mod login;
pub mod config;
pub mod play;

/// Minecraft protocol version for 1.21.2–1.21.4
pub const PROTOCOL_VERSION: i32 = 766;
/// Change to 767 for 1.21.5, 768 for 1.21.6+.
/// FerrumC 1.21.8 = 772.

pub const FERRUMC_PROTOCOL: i32 = 772;
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ConnectionState {
    Handshake,
    Status,
    Login,
    Config,
    Play,
}
