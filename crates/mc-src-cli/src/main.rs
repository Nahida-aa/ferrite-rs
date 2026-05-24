use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;

use clap::{Parser, Subcommand};
use serde::Deserialize;

const CFR_URL: &str =
    "https://repo1.maven.org/maven2/org/benf/cfr/0.152/cfr-0.152.jar";
const MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

const CRATE_DIR: &str = env!("CARGO_MANIFEST_DIR");

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct Config {
    #[serde(default = "default_versions")]
    versions: Vec<String>,
}

fn default_versions() -> Vec<String> {
    vec!["1.21.8".to_string()]
}

impl Default for Config {
    fn default() -> Self {
        Self { versions: default_versions() }
    }
}

// ---------------------------------------------------------------------------
// CLI
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(name = "mc-src-cli")]
struct Cli {
    #[command(subcommand)]
    command: Sub,
}

#[derive(Subcommand)]
enum Sub {
    /// Decompile Minecraft jar to readable Java source
    Decompile {
        #[arg(long)]
        versions: Vec<String>,

        #[arg(long, default_value = ".minecraft")]
        minecraft_dir: PathBuf,

        #[arg(long, default_value = "mc-src")]
        output_dir: PathBuf,
    },

    /// Extract assets (textures/sounds) and data (blockstates/tags) from jar
    Export {
        #[arg(long, default_value = "1.21.8")]
        version: String,

        #[arg(long, default_value = ".minecraft")]
        minecraft_dir: PathBuf,

        /// Asset output directory (<output>/minecraft/...)
        #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../client/assets"))]
        assets_dir: PathBuf,

        /// Data output directory (<output>/minecraft/...)
        #[arg(long, default_value = concat!(env!("CARGO_MANIFEST_DIR"), "/../client/data"))]
        data_dir: PathBuf,

        /// What to export: "assets", "data", or "all"
        #[arg(long, default_value = "all")]
        types: String,
    },
}

// ---------------------------------------------------------------------------
// Manifest types
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
struct Manifest {
    latest: Latest,
    versions: Vec<VersionEntry>,
}

#[derive(Deserialize)]
struct Latest {
    release: String,
    snapshot: String,
}

#[derive(Deserialize)]
struct VersionEntry {
    id: String,
    url: String,
}

#[derive(Deserialize)]
struct VersionMeta {
    downloads: Downloads,
}

#[derive(Deserialize)]
struct Downloads {
    client: Artifact,
    #[serde(default)]
    client_mappings: Option<Artifact>,
}

#[derive(Deserialize)]
struct Artifact {
    url: String,
    #[serde(default)]
    sha1: String,
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn cache_path(name: &str) -> PathBuf {
    PathBuf::from(CRATE_DIR).join(".cache").join(name)
}

fn cfr_path() -> PathBuf {
    cache_path("tools/cfr.jar")
}

fn count_java_files(dir: &std::path::Path) -> usize {
    if !dir.exists() {
        return 0;
    }
    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                count += count_java_files(&path);
            } else if path.extension().is_some_and(|e| e == "java") {
                count += 1;
            }
        }
    }
    count
}

async fn fetch_url(url: &str, dest: &std::path::Path) -> anyhow::Result<PathBuf> {
    if dest.exists() {
        return Ok(dest.to_path_buf());
    }
    tracing::info!("Downloading {url}");
    tokio::fs::create_dir_all(dest.parent().unwrap()).await?;
    let response = reqwest::get(url).await?.bytes().await?;
    tokio::fs::write(dest, &response).await?;
    Ok(dest.to_path_buf())
}

async fn fetch_manifest() -> anyhow::Result<Manifest> {
    let path = fetch_url(MANIFEST_URL, &cache_path("manifest.json")).await?;
    let data = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&data)?)
}

async fn fetch_version_meta(version: &str) -> anyhow::Result<VersionMeta> {
    let manifest = fetch_manifest().await?;
    let entry = manifest
        .versions
        .iter()
        .find(|e| e.id == version)
        .ok_or_else(|| anyhow::anyhow!("Version {version} not found in manifest"))?;
    let path = fetch_url(&entry.url, &cache_path(&format!("{version}.json"))).await?;
    let data = tokio::fs::read_to_string(&path).await?;
    Ok(serde_json::from_str(&data)?)
}

// ---------------------------------------------------------------------------
// Decompile
// ---------------------------------------------------------------------------

async fn ensure_cfr() -> anyhow::Result<PathBuf> {
    let path = cfr_path();
    if path.exists() {
        return Ok(path);
    }
    tracing::info!("Downloading CFR decompiler...");
    tokio::fs::create_dir_all(path.parent().unwrap()).await?;
    let response = reqwest::get(CFR_URL).await?.bytes().await?;
    tokio::fs::write(&path, &response).await?;
    Ok(path)
}

async fn ensure_jar(version: &str, minecraft_dir: &Path) -> anyhow::Result<PathBuf> {
    let jar = minecraft_dir
        .join("versions")
        .join(version)
        .join(format!("{version}.jar"));
    if jar.exists() {
        tracing::info!("Jar exists: {}", jar.display());
        return Ok(jar);
    }

    tracing::info!("Jar not found. Running mc-launcher-cli to download...");
    let status = StdCommand::new("cargo")
        .args([
            "run",
            "--package",
            "mc-launcher-cli",
            "--",
            "--version",
            version,
            "--no-assets",
            "--no-launch",
        ])
        .status()?;

    if !status.success() || !jar.exists() {
        tracing::warn!("mc-launcher-cli failed, downloading directly...");
        let meta = fetch_version_meta(version).await?;
        let jar_url = &meta.downloads.client.url;
        tokio::fs::create_dir_all(jar.parent().unwrap()).await?;
        tracing::info!("Downloading {version}.jar...");
        let response = reqwest::get(jar_url).await?.bytes().await?;
        tokio::fs::write(&jar, &response).await?;
    }

    if !jar.exists() {
        anyhow::bail!("Failed to obtain jar for {version}");
    }
    Ok(jar)
}

async fn decompile(version: &str, jar_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    let src_dir = output_dir.join(version);
    if src_dir.join("net/minecraft").exists() {
        let file_count = count_java_files(&src_dir);
        tracing::info!("Already decompiled ({file_count} files), skipping");
        return Ok(());
    }

    let meta = fetch_version_meta(version).await?;
    let cfr = ensure_cfr().await?;

    let has_mappings = meta.downloads.client_mappings.is_some();
    let mut cmd = StdCommand::new("java");
    cmd.arg("-jar")
        .arg(&cfr)
        .arg(jar_path)
        .arg("--outputdir")
        .arg(&src_dir)
        .arg("--silent")
        .arg("true");

    if let Some(mappings) = &meta.downloads.client_mappings {
        let mappings_path = cache_path(&format!("jars/{version}-mappings.txt"));
        if !mappings_path.exists() {
            tracing::info!("Downloading mappings...");
            tokio::fs::create_dir_all(mappings_path.parent().unwrap()).await?;
            let response = reqwest::get(&mappings.url).await?.bytes().await?;
            tokio::fs::write(&mappings_path, &response).await?;
        }
        cmd.arg("--obfuscationpath").arg(&mappings_path);
    }

    let action = if has_mappings {
        "obfuscated + mappings"
    } else {
        "unobfuscated"
    };
    tracing::info!("Status: {action}, decompiling...");

    let output = cmd.output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("CFR failed: {stderr}");
    }

    let file_count = count_java_files(&src_dir);
    tracing::info!("Done! {file_count} source files -> {}", src_dir.display());
    Ok(())
}

// ---------------------------------------------------------------------------
// Export
// ---------------------------------------------------------------------------

fn export(version: &str, minecraft_dir: &Path, assets_dir: &Path, data_dir: &Path, types: &str) -> anyhow::Result<()> {
    let jar = minecraft_dir
        .join("versions")
        .join(version)
        .join(format!("{version}.jar"));
    if !jar.exists() {
        anyhow::bail!("Jar not found: {} (run 'mc-src-cli decompile' first)", jar.display());
    }

    let file = std::fs::File::open(&jar)?;
    let mut archive = zip::ZipArchive::new(file)?;

    let do_assets = types == "all" || types == "assets";
    let do_data = types == "all" || types == "data";

    let prefixes: Vec<(&str, &Path)> = match (do_assets, do_data) {
        (true, true) => vec![("assets/", assets_dir), ("data/", data_dir)],
        (true, false) => vec![("assets/", assets_dir)],
        (false, true) => vec![("data/", data_dir)],
        (false, false) => return Ok(()),
    };

    for &(prefix, output) in &prefixes {
        let mut count = 0;

        for i in 0..archive.len() {
            let mut entry = match archive.by_index(i) {
                Ok(e) => e,
                Err(_) => continue,
            };

            let Some(zip_path) = entry.enclosed_name().map(|p| p.to_string_lossy().to_string()) else {
                continue;
            };

            if !zip_path.starts_with(prefix) {
                continue;
            }

            let relative = &zip_path[prefix.len()..];
            let dest = output.join(&relative);

            if entry.is_dir() {
                std::fs::create_dir_all(&dest).ok();
                continue;
            }

            std::fs::create_dir_all(dest.parent().unwrap())?;

            let mut data = Vec::with_capacity(entry.size() as usize);
            std::io::Read::read_to_end(&mut entry, &mut data)?;
            std::fs::write(&dest, &data)?;
            count += 1;
        }

        tracing::info!("Exported {count} files to {}", output.display());
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Main
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Sub::Decompile { versions, minecraft_dir, output_dir } => {
            let versions = if versions.is_empty() {
                let config_path = PathBuf::from(CRATE_DIR).join("config.json");
                let cfg: Config = if config_path.exists() {
                    let data = tokio::fs::read_to_string(&config_path).await?;
                    serde_json::from_str(&data)?
                } else {
                    Config::default()
                };
                cfg.versions
            } else {
                versions
            };

            let manifest = fetch_manifest().await?;
            tracing::info!("Latest release: {}", manifest.latest.release);

            for mut v in versions {
                if v == "latest" {
                    v = manifest.latest.release.clone();
                }

                tracing::info!("{:=<60}", "");
                tracing::info!("  Version: {}", v);
                tracing::info!("{:=<60}", "");

                let jar = ensure_jar(&v, &minecraft_dir).await?;

                if let Err(e) = decompile(&v, &jar, &output_dir).await {
                    tracing::error!("  Failed: {e}");
                }
            }

            tracing::info!("Done. Output: {}", output_dir.display());
        }
        Sub::Export { version, minecraft_dir, assets_dir, data_dir, types } => {
            if let Err(e) = export(&version, &minecraft_dir, &assets_dir, &data_dir, &types) {
                tracing::error!("Export failed: {e}");
            }
        }
    }

    Ok(())
}
