mod connection;

use anyhow::Result;
use tokio::runtime::Handle;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum NetworkEvent {
    Connected,
    Disconnected(String),
}

pub struct Network {
    rx: mpsc::Receiver<NetworkEvent>,
}

impl Network {
    pub fn connect(
        handle: &Handle,
        addr: &str,
        username: &str,
    ) -> (Self, tokio::task::JoinHandle<()>) {
        let (tx, rx) = mpsc::channel(256);
        let addr = addr.to_string();
        let username = username.to_string();

        let join = handle.spawn(async move {
            if let Err(e) = connection::run(&addr, &username, &tx).await {
                let _ = tx.send(NetworkEvent::Disconnected(e.to_string())).await;
            }
        });

        (Self { rx }, join)
    }

    pub fn try_recv(&mut self) -> Result<Option<NetworkEvent>, mpsc::error::TryRecvError> {
        match self.rx.try_recv() {
            Ok(event) => Ok(Some(event)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
