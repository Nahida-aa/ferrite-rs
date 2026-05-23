mod render;
mod network;
mod state;

use anyhow::Result;
use winit::event_loop::EventLoop;

fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let event_loop = EventLoop::new()?;
    let mut app = state::AppState::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
