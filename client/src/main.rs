mod render;
mod network;
mod state;
mod server;

use std::fs::OpenOptions;
use winit::event_loop::EventLoop;

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

    let event_loop = EventLoop::new()?;
    let mut app = state::AppState::new()?;
    if auto_connect {
        app.queue_connect("localhost:25565".to_owned(), true);
    }
    event_loop.run(move |event, target| {
        app.handle_event(event, target);
    })?;
    Ok(())
}
