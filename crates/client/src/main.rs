mod chunk_mesh;
mod net_plugin;
mod server;

use std::fs::OpenOptions;

use bevy::prelude::*;
use tracing_subscriber::EnvFilter;

fn main() -> anyhow::Result<()> {
    std::fs::create_dir_all("logs").ok();
    let log_path = format!("logs/ferrite-{}.log", chrono_timestamp());
    let log_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)?;

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
    app.add_plugins((
        net_plugin::NetworkPlugin,
        ferrite_gui::player::PlayerPlugin,
        ferrite_gui::UIPlugin,
    ));

    if auto_connect {
        app.world_mut()
            .resource_mut::<net_plugin::PendingConnect>()
            .0
            .push(("localhost:25565".to_string(), true, Some("world".to_string())));
    }

    app.run();
    Ok(())
}

fn chrono_timestamp() -> String {
    chrono::Local::now().format("%Y%m%d-%H%M%S").to_string()
}
