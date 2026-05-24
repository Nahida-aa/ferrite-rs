use bevy::prelude::*;

use crate::{LanDiscoveryState, PauseMenuOpen, SelectedServer, UiFont, UiRes, UiScreenState};
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
            .init_resource::<SelectedWorld>()
            .init_resource::<SelectedServer>()
            .add_systems(Startup, |mut commands: Commands, server: Res<AssetServer>| {
                commands.insert_resource(UiFont(server.load("fonts/aaxlMonoSC-Regular.ttf")));
            });
    }
}
