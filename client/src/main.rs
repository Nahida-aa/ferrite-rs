mod game;
mod net_plugin;
mod network;
mod player;
mod server;
mod ui;

use std::fs::OpenOptions;

use bevy::prelude::*;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("ferrite.log")?;

    let mut filter = EnvFilter::builder()
        .with_env_var("FERRITE_LOG")
        .from_env_lossy();
    filter = filter.add_directive("ferrite_client=debug".parse().unwrap());
    tracing_subscriber::fmt()
        .with_target(false)
        .with_writer(log_file)
        .with_ansi(false)
        .with_env_filter(filter)
        .init();

    let auto_connect = std::env::args().any(|a| a == "--auto-connect");

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.05)));
    app.add_plugins(DefaultPlugins);
    app.add_plugins(game::GamePlugin);

    if auto_connect {
        app.world_mut()
            .resource_mut::<game::PendingConnect>()
            .0
            .push(("localhost:25565".to_string(), true));
    }

    app.run();
    Ok(())
}
