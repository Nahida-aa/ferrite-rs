use anyhow::{Context, Result};
use std::net::TcpStream;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

pub struct ServerHandle {
    child: Child,
}

impl ServerHandle {
    pub fn spawn() -> Result<Self> {
        let path = find_ferrumc().context(
            "FerrumC binary not found. Download it from https://github.com/sweattypalms/ferrumc\n\
             Place the `ferrumc` binary in the current directory or add it to PATH.\n\
             Or set FERRUMC_PATH environment variable.",
        )?;

        tracing::info!("Starting FerrumC from: {}", path.display());
        let child = Command::new(&path)
            .arg("run")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| format!("Failed to spawn FerrumC at {}", path.display()))?;

        tracing::info!("FerrumC started from {}", path.display());

        let handle = Self { child };
        handle.wait_for_port(Duration::from_secs(15))?;

        Ok(handle)
    }

    fn wait_for_port(&self, timeout: Duration) -> Result<()> {
        let start = Instant::now();
        loop {
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
    for name in &["ferrumc", "ferrumc.exe"] {
        let p = PathBuf::from(name);
        if p.is_file() {
            // Prepend ./ so Command::new treats it as a path (not PATH lookup)
            return Some(PathBuf::from("./").join(name));
        }
        let p = PathBuf::from("bin").join(name);
        if p.is_file() {
            return Some(p);
        }
    }

    // 3. PATH
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
