use bevy::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct WorldEntry {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Resource, Default)]
pub struct WorldManager {
    pub worlds: Vec<WorldEntry>,
}

#[derive(Resource, Default)]
pub struct SelectedWorld(pub Option<String>);

impl WorldManager {
    pub fn discover(worlds_dir: &Path) -> Self {
        let mut worlds = Vec::new();
        if let Ok(entries) = std::fs::read_dir(worlds_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                        if !name.starts_with('.') {
                            worlds.push(WorldEntry {
                                name: name.to_string(),
                                path,
                            });
                        }
                    }
                }
            }
        }
        worlds.sort_by(|a, b| a.name.cmp(&b.name));
        Self { worlds }
    }

    pub fn default_worlds_dir() -> PathBuf {
        PathBuf::from("saves")
    }

    pub fn needs_import(db_path: &Path) -> bool {
        let has_anvil = db_path.join("region").is_dir();
        let has_lmdb = db_path.join("data.mdb").is_file();
        has_anvil && !has_lmdb
    }
}

pub fn write_server_config(root: &Path, db_path: &str) -> anyhow::Result<()> {
    let config_dir = root.join("configs");
    std::fs::create_dir_all(&config_dir)?;

    use rand::Rng;
    let secret: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let online_mode = !cfg!(debug_assertions);
    let config = format!(
        r#"host = "0.0.0.0"
port = 25565
motd = ["Welcome to the best server ever!", "Rust", "Good luck, have fun!"]
max_players = 100
tps = 20
world = "ferrite"
whitelist = false
network_compression_threshold = 64

verify_decompressed_packets = true
encryption_enabled = {online_mode}
online_mode = {online_mode}
chunk_render_distance = 12

default_gamemode = "creative"

[database]
db_path = "{db_path}"
verify_chunk_data = true
map_size = 1_000
cache_ttl = 60
cache_capacity = 20_000

[performance]
chunks_per_tick = 0
chunks_per_tick_min = 16

[dashboard]
port = 9000
secret = "{secret}"
"#,
        db_path = db_path
    );
    std::fs::write(config_dir.join("config.toml"), config)?;
    tracing::info!("Wrote server config with db_path={} (root={})", db_path, root.display());
    Ok(())
}
