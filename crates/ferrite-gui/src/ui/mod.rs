use bevy::prelude::*;

use crate::{LanDiscoveryState, PauseMenuOpen, UiRes, UiScreenState};
use crate::lan_discovery::LanState;
use crate::worlds::{SelectedWorld, WorldManager};

pub mod hud;
pub mod menu;
pub mod pause;
pub mod server_list;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(UiRes { last_error: None })
            .insert_resource(PauseMenuOpen(false))
            .init_resource::<UiScreenState>()
            .insert_resource(LanDiscoveryState {
                lan: LanState::default(),
                servers: Vec::new(),
                initialized: false,
                generation: 0,
            })
            .init_resource::<WorldManager>()
            .init_resource::<SelectedWorld>();
    }
}
