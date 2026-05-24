use std::sync::{Arc, Mutex};
use std::thread;

/// Shared state between the LAN listener thread and the Bevy main thread.
/// Call `init()` once at startup to bind the socket and start listening.
#[derive(Clone, Default)]
pub struct LanState {
    pub servers: Arc<Mutex<Vec<ferrite_net::lan::DiscoveredServer>>>,
    pub available: Arc<Mutex<bool>>,
}

impl LanState {
    pub fn init(&self) {
        let mut avail = self.available.lock().unwrap();
        if *avail {
            return;
        }

        let servers = self.servers.clone();

        let socket = match ferrite_net::lan::create_lan_socket() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[Ferrite] LAN discovery unavailable (port 4445): {e}");
                return;
            }
        };

        *avail = true;
        drop(avail);

        thread::spawn(move || {
            let mut buf = [0u8; 2048];
            loop {
                match socket.recv_from(&mut buf) {
                    Ok((len, src)) => {
                        if let Some(server) = ferrite_net::lan::parse_lan_packet(&buf[..len], src) {
                            let mut list = servers.lock().unwrap();
                            if let Some(pos) =
                                list.iter().position(|s| s.address == server.address)
                            {
                                list[pos] = server;
                            } else {
                                list.push(server);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                    Err(_) => break,
                }
            }
        });
    }

    pub fn is_available(&self) -> bool {
        *self.available.lock().unwrap()
    }

    pub fn take_servers(&self) -> Vec<ferrite_net::lan::DiscoveredServer> {
        std::mem::take(&mut *self.servers.lock().unwrap())
    }
}
