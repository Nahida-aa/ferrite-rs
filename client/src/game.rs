pub use crate::net_plugin::PendingConnect;

use bevy::app::{App, Plugin};

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            crate::net_plugin::NetworkPlugin,
            crate::player::PlayerPlugin,
            crate::ui::UIPlugin,
        ));
    }
}
