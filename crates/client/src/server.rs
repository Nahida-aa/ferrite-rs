use anyhow::{Context, Result};
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct ServerHandle {
    child: Child,
}

impl ServerHandle {
    pub fn spawn(db_path: &str) -> Result<Self> {
        // Kill any existing ferrumc before starting a new one
        Self::kill_existing();

        let found = find_ferrumc().context(
            "FerrumC binary not found. Download it from https://github.com/sweattypalms/ferrumc\n\
             Place the `ferrumc` binary in the current directory or add it to PATH.\n\
             Or set FERRUMC_PATH environment variable.",
        )?;

        // Copy to project root so ferrumc's get_root_path() / current_exe().parent()
        // resolves to the project root, keeping all paths consistent with the CWD.
        let binary = ensure_local_binary(&found)?;

        tracing::info!("Starting FerrumC from: {}", binary.display());

        gui::worlds::write_server_config(Path::new("."), db_path)?;

        // Import Anvil → LMDB if the world hasn't been imported yet.
        let world_dir = Path::new(db_path);
        if gui::worlds::WorldManager::needs_import(world_dir) {
            tracing::info!("Importing world from {}...", world_dir.display());
            let mut import_child = Command::new(&binary)
                .arg("import")
                .arg("--import-path")
                .arg(world_dir)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .with_context(|| format!("Failed to spawn import for {}", world_dir.display()))?;

            if let Some(stdout) = import_child.stdout.take() {
                std::thread::spawn(|| {
                    let reader = BufReader::new(stdout);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            println!("[ferrumc] {line}");
                        }
                    }
                });
            }
            if let Some(stderr) = import_child.stderr.take() {
                std::thread::spawn(|| {
                    let reader = BufReader::new(stderr);
                    for line in reader.lines() {
                        if let Ok(line) = line {
                            eprintln!("[ferrumc] {line}");
                        }
                    }
                });
            }

            let status = import_child.wait()?;
            if !status.success() {
                anyhow::bail!("World import failed with status: {status}");
            }
            tracing::info!("Import complete.");
        }

        let mut child = Command::new(&binary)
            .arg("run")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to spawn FerrumC at {}", binary.display()))?;

        // Prefix ferrumc's stdout/stderr with [ferrumc] for visibility
        if let Some(stdout) = child.stdout.take() {
            std::thread::spawn(|| {
                let reader = BufReader::new(stdout);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        println!("[ferrumc] {line}");
                    }
                }
            });
        }
        if let Some(stderr) = child.stderr.take() {
            std::thread::spawn(|| {
                let reader = BufReader::new(stderr);
                for line in reader.lines() {
                    if let Ok(line) = line {
                        eprintln!("[ferrumc] {line}");
                    }
                }
            });
        }

        tracing::info!("FerrumC started from {}", binary.display());

        let mut handle = Self { child };
        handle.wait_for_port(Duration::from_secs(15))?;

        // Log effective server config from the written file
        if let Ok(mut f) = std::fs::File::open("configs/config.toml") {
            let mut buf = String::new();
            if f.read_to_string(&mut buf).is_ok() {
                for line in buf.lines() {
                    if line.starts_with("db_path")
                        || line.starts_with("online_mode")
                        || line.starts_with("encryption")
                        || line.starts_with("port")
                    {
                        tracing::info!("Server config: {line}");
                    }
                }
            }
        }

        Ok(handle)
    }

    fn kill_existing() {
        // Kill exact process names to avoid matching unrelated processes
        let _ = Command::new("pkill").args(["-x", "ferrumc"]).output();
        let _ = Command::new("pkill")
            .args(["-x", "ferrite-server"])
            .output();
        std::thread::sleep(std::time::Duration::from_millis(500));
    }

    fn wait_for_port(&mut self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
            if self.child.try_wait()?.is_some() {
                anyhow::bail!("FerrumC exited before binding to port 25565");
            }
            if TcpStream::connect_timeout(
                &"127.0.0.1:25565".parse().unwrap(),
                Duration::from_millis(500),
            )
            .is_ok()
            {
                tracing::info!("Local server ready on 127.0.0.1:25565");
                return Ok(());
            }
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timed out waiting for local server on port 25565 ({}s)",
                    timeout.as_secs()
                );
            }
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

impl Drop for ServerHandle {
    fn drop(&mut self) {
        tracing::info!("Shutting down local server...");
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn find_ferrumc() -> Option<PathBuf> {
    // 1. FERRUMC_PATH env var
    if let Ok(path) = std::env::var("FERRUMC_PATH") {
        let p = PathBuf::from(&path);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. Current directory
    // NOTE: `ferrumc/` is the git submodule directory, NOT a binary.
    // The copied local binary lives as `ferrite-server` to avoid the name clash.
    for name in &[
        "ferrite-server",
        "ferrite-server.exe",
        "ferrumc",
        "ferrumc.exe",
    ] {
        let p = PathBuf::from(name);
        if p.is_file() {
            return Some(PathBuf::from("./").join(name));
        }
        let p = PathBuf::from("bin").join(name);
        if p.is_file() {
            return Some(p);
        }
    }

    // 3. ferrumc submodule build output
    for name in &["ferrumc", "ferrumc.exe"] {
        let p = PathBuf::from("ferrumc/target/release").join(name);
        if p.is_file() {
            return Some(p);
        }
        let p = PathBuf::from("ferrumc/target/debug").join(name);
        if p.is_file() {
            return Some(p);
        }
    }

    // 4. PATH
    std::env::var_os("PATH").and_then(|paths| {
        std::env::split_paths(&paths).find_map(|dir| {
            for name in &["ferrumc", "ferrumc.exe"] {
                let p = dir.join(name);
                if p.is_file() {
                    return Some(p);
                }
            }
            None
        })
    })
}

/// Copy the ferrumc binary to `./ferrite-server` so that `current_exe().parent()`
/// resolves to the project root, keeping all config / data paths consistent with CWD.
fn ensure_local_binary(found: &Path) -> Result<PathBuf> {
    let local = PathBuf::from("./ferrite-server");
    if local.is_file() {
        return Ok(local);
    }
    tracing::info!(
        "Copying FerrumC from {} to {}",
        found.display(),
        local.display()
    );
    std::fs::copy(found, &local)?;
    Ok(local)
}
