use std::collections::HashSet;
use std::time::{Duration, Instant};

use network::lan::create_lan_socket;

pub fn run(duration_secs: u64) {
    let socket = match create_lan_socket() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("LAN discovery unavailable (port 4445): {e}");
            return;
        }
    };

    println!("Scanning for Minecraft LAN games on 224.0.2.60:4445 ...");
    println!("(scanning for {duration_secs} seconds)");
    println!();

    let start = Instant::now();
    let total = Duration::from_secs(duration_secs);
    let mut seen = HashSet::new();
    let mut buf = [0u8; 2048];

    loop {
        if start.elapsed() >= total {
            break;
        }

        match socket.recv_from(&mut buf) {
            Ok((len, src)) => {
                if let Some(server) = network::lan::parse_lan_packet(&buf[..len], src) {
                    if seen.insert(server.address.clone()) {
                        println!(
                            "  {} | {} | {}/{} players | {}",
                            server.motd,
                            server.address,
                            server.player_count,
                            server.max_players,
                            if server.version.is_empty() {
                                "?"
                            } else {
                                &server.version
                            },
                        );
                    }
                }
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(_) => break,
        }
    }

    println!();
    if seen.is_empty() {
        println!("No servers found.");
    } else {
        println!("Found {} server(s):", seen.len());
        for addr in &seen {
            println!("  {addr}");
        }
    }
}
