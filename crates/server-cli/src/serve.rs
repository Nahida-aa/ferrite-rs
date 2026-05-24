use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::process;
use tokio::signal;

pub struct ServeConfig {
    pub port: u16,
    pub online_mode: bool,
    pub rebuild: bool,
}

impl Default for ServeConfig {
    fn default() -> Self {
        Self {
            port: 25565,
            online_mode: false,
            rebuild: false,
        }
    }
}

pub async fn serve(config: ServeConfig) -> Result<()> {
    // Kill any existing ferrite-server / ferrumc processes
    let _ = Command::new("pkill")
        .args(["-x", "ferrite-server"])
        .output();
    let _ = Command::new("pkill")
        .args(["-x", "ferrumc"])
        .output();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let binary = ensure_server(config.rebuild)?;
    tracing::info!("Starting server from: {}", binary.display());

    let mut cmd = process::Command::new(&binary);
    cmd.arg("run")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.arg("--port").arg(config.port.to_string());
    if config.online_mode {
        cmd.arg("--online-mode");
    }

    let mut child = cmd
        .spawn()
        .with_context(|| format!("Failed to spawn {}", binary.display()))?;

    tracing::info!("Server started (PID: {})", child.id().unwrap_or(0));

    // Wait for ctrl+c or child exit
    tokio::select! {
        _ = signal::ctrl_c() => {
            tracing::info!("Shutting down server...");
            graceful_kill(&mut child).await;
        }
        status = child.wait() => {
            match status {
                Ok(s) => tracing::info!("Server exited with status: {s}"),
                Err(e) => tracing::error!("Server process error: {e}"),
            }
        }
    }

    Ok(())
}

async fn graceful_kill(child: &mut process::Child) {
    // Send SIGTERM, wait 3s, then SIGKILL
    let _ = child.kill().await;
    tokio::time::sleep(Duration::from_secs(3)).await;
    let _ = child.kill().await;
    let _ = child.wait().await;
}

/// Find the ferrumc/ferrite-server binary, building it if needed.
/// Always copies to `./ferrite-server` for consistent path resolution.
fn ensure_server(rebuild: bool) -> Result<PathBuf> {
    let local = PathBuf::from("./ferrite-server");

    // If `--rebuild` or no binary found, build from source
    if rebuild || !local.is_file() {
        // Search for existing binary first
        if !rebuild {
            if let Some(found) = find_server() {
                if found != local {
                    copy_if_different(&found, &local)?;
                }
                return Ok(local);
            }
        }

        // Build from ferrumc submodule
        tracing::info!("Building ferrumc server (release)...");
        let status = Command::new("cargo")
            .args(["build", "-p", "ferrumc", "--release"])
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run cargo build for ferrumc")?;

        if !status.success() {
            anyhow::bail!("cargo build -p ferrumc --release failed");
        }

        let built = PathBuf::from("ferrumc/target/release/ferrumc");
        if !built.is_file() {
            anyhow::bail!(
                "Build succeeded but {} not found",
                built.display()
            );
        }

        std::fs::copy(&built, &local)
            .with_context(|| format!("Failed to copy {} to ferrite-server", built.display()))?;
        tracing::info!("Copied ferrumc to ./ferrite-server");
    }

    if !local.is_file() {
        anyhow::bail!(
            "ferrite-server binary not found. Build it first or place ferrumc in PATH.\n\
             See https://github.com/sweattypalms/ferrumc"
        );
    }

    Ok(local)
}

/// Search for existing server binaries in order of preference.
fn find_server() -> Option<PathBuf> {
    // 1. FERRUMC_PATH env var
    if let Ok(path) = std::env::var("FERRUMC_PATH") {
        let p = PathBuf::from(&path);
        if p.is_file() {
            return Some(p);
        }
    }

    // 2. Current directory — prefer ferrite-server over ferrumc
    for name in &["ferrite-server", "ferrite-server.exe", "ferrumc", "ferrumc.exe"] {
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

fn copy_if_different(src: &Path, dst: &Path) -> Result<()> {
    if dst.is_file() {
        // Skip if same file (both point to ./ferrite-server)
        if same_file(src, dst) {
            return Ok(());
        }
        // Check if already up to date
        if let (Ok(src_m), Ok(dst_m)) = (src.metadata(), dst.metadata()) {
            if src_m.len() == dst_m.len()
                && src_m.modified().ok() == dst_m.modified().ok()
            {
                return Ok(());
            }
        }
    }
    tracing::info!("Copying {} → {}", src.display(), dst.display());
    std::fs::copy(src, dst)?;
    Ok(())
}

fn same_file(a: &Path, b: &Path) -> bool {
    if let (Ok(ma), Ok(mb)) = (a.canonicalize(), b.canonicalize()) {
        ma == mb
    } else {
        a == b
    }
}
