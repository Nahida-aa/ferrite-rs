mod connection;

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum NetworkEvent {
    Connected,
    Disconnected(String),
    PlayerPosition(f64, f64, f64),
    LoginPlay {
        entity_id: i32,
        game_mode: u8,
    },
    ChunkData {
        x: i32,
        z: i32,
        chunk: core::chunk::Chunk,
    },
}

#[derive(Debug)]
pub enum NetworkCommand {
    SetPosition {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        on_ground: bool,
    },
}

pub struct Network {
    rx: mpsc::Receiver<NetworkEvent>,
    cmd_tx: mpsc::Sender<NetworkCommand>,
}

impl Network {
    pub fn connect(
        handle: &Handle,
        addr: &str,
        username: &str,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(256);
        let (cmd_tx, cmd_rx) = mpsc::channel(256);
        let addr = addr.to_string();
        let username = username.to_string();

        let join = handle.spawn(async move {
            if let Err(e) = connection::run(&addr, &username, &tx, cmd_rx).await {
                let _ = tx.send(NetworkEvent::Disconnected(e.to_string())).await;
            }
        });

        (Self { rx, cmd_tx }, join)
    }

    pub fn try_recv(&mut self) -> Result<Option<NetworkEvent>, mpsc::error::TryRecvError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn command_sender(&self) -> mpsc::Sender<NetworkCommand> {
        self.cmd_tx.clone()
    }
}
