use bevy::prelude::*;

pub mod lan_discovery;
pub mod player;
pub mod ui;
pub mod worlds;

pub use ferrite_net::lan::DiscoveredServer;
pub use ui::UIPlugin;


// ── UI Resources ──

#[derive(Resource)]
pub struct UiRes {
    pub last_error: Option<String>,
}

#[derive(Resource)]
pub struct UiFont(pub Handle<Font>);

#[derive(Resource)]
pub struct PauseMenuOpen(pub bool);

#[derive(Resource, Default)]
pub struct UiScreenState(pub UiScreen);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiScreen {
    MainMenu,
    WorldSelect,
    ServerList,
}

impl Default for UiScreen {
    fn default() -> Self {
        Self::MainMenu
    }
}

// ── UI Component markers ──

#[derive(Component)]
pub struct MainMenuUI;

#[derive(Component)]
pub struct WorldSelectUI;

#[derive(Component)]
pub struct ServerListUI;

#[derive(Component)]
pub struct PauseMenuUI;

#[derive(Component)]
pub struct HUDUI;

#[derive(Component)]
pub struct CoordText;

#[derive(Component)]
pub struct WorldEntryButton(pub String);

#[derive(Component)]
pub struct PlayWorldButton;

// ── Server List Resources ──

#[derive(Resource, Default)]
pub struct SelectedServer(pub Option<String>);

// ── Player Resources ──

#[derive(Resource)]
pub struct PlayerRes {
    pub position: Option<(f64, f64, f64)>,
}

#[derive(Resource)]
pub struct PlayerInfoRes {
    pub entity_id: Option<i32>,
    pub game_mode: Option<u8>,
}

impl Default for PlayerInfoRes {
    fn default() -> Self {
        Self {
            entity_id: None,
            game_mode: None,
        }
    }
}

#[derive(Resource)]
pub struct PlayerLookRes {
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for PlayerLookRes {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.3,
        }
    }
}

#[derive(Component)]
pub struct PlayerBlock;

#[derive(Resource)]
pub struct PlayerBlockEntity(pub Option<Entity>);

// ── Debug Overlay ──

#[derive(Resource, Default)]
pub struct ChunkCount(pub usize);

#[derive(Component)]
pub struct DebugOverlayUI;

#[derive(Resource)]
pub struct DebugOverlayVisible(pub bool);

// ── LanDiscoveryState ──

#[derive(Resource)]
pub struct LanDiscoveryState {
    pub lan: crate::lan_discovery::LanState,
    pub servers: Vec<DiscoveredServer>,
    pub initialized: bool,
    pub generation: u64,
}
