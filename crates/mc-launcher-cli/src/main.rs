#![allow(dead_code)]

mod manifest;
mod download;
mod library;
mod assets;
mod launch;

use std::path::PathBuf;
use anyhow::Result;
use clap::Parser;

const MANIFEST_URL: &str =
    "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Parser)]
#[command(name = "mc-launcher-cli", about = "Launch the vanilla Minecraft client")]
struct Cli {
    #[arg(short, long, default_value = "1.21.8")]
    version: String,

    #[arg(short, long)]
    server: Option<String>,

    #[arg(short = 'P', long, default_value = "25565")]
    port: u16,

    #[arg(short, long, default_value = "TestPlayer")]
    username: String,

    #[arg(short = 'J', long, default_value = "java")]
    java: String,

    #[arg(long)]
    no_assets: bool,

    #[arg(long)]
    cache: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    let cache_dir = cli
        .cache
        .unwrap_or_else(|| {
            dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("/tmp"))
                .join("mc-launcher-cli")
        });

    let cache = download::Cache::new(cache_dir);

    // Fetch version manifest
    tracing::info!("Fetching version manifest...");
    let manifest_path = cache.get(MANIFEST_URL, None).await?;
    let manifest_data = tokio::fs::read_to_string(&manifest_path).await?;
    let manifest: manifest::VersionManifest = serde_json::from_str(&manifest_data)?;

    let version_id = if cli.version == "latest" {
        manifest.latest.release.clone()
    } else {
        cli.version.clone()
    };

    let version_entry = manifest
        .versions
        .iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version_id))?;

    tracing::info!("Fetching metadata for {}...", version_id);
    let meta_path = cache.get(&version_entry.url, None).await?;
    let meta_data = tokio::fs::read_to_string(&meta_path).await?;
    let metadata: manifest::VersionMetadata = serde_json::from_str(&meta_data)?;

    tracing::info!("  Type: {}, Java: {}",
        metadata.kind, metadata.java_version.major_version);

    // Download client jar
    tracing::info!("Downloading client jar...");
    let client_jar = cache
        .get_jar(
            &metadata.downloads.client.url,
            &format!("{}.jar", version_id),
            Some(&metadata.downloads.client.sha1),
        )
        .await?;

    // Resolve and download libraries
    tracing::info!("Resolving libraries...");
    let (libraries, natives_dir) = library::resolve_libraries(&cache, &metadata).await?;
    tracing::info!("  {} libraries resolved", libraries.len());

    // Assets
    let game_dir = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from(".minecraft"))
        .join(".minecraft");
    let assets_dir = game_dir.join("assets");
    let assets_root = assets_dir.clone();

    if !cli.no_assets {
        tracing::info!("Downloading assets...");
        assets::ensure_assets(&cache, &metadata, &assets_dir).await?;
    } else {
        tracing::info!("Skipping assets (--no-assets)");
    }

    // Launch
    tracing::info!("Launching Minecraft {}...", version_id);

    let config = launch::LaunchConfig {
        java_path: PathBuf::from(&cli.java),
        version: version_id,
        server: cli.server,
        port: Some(cli.port),
        username: cli.username,
        uuid: "00000000-0000-0000-0000-000000000000".to_string(),
        access_token: "0".to_string(),
        user_type: "mojang".to_string(),
        game_dir,
        assets_dir,
        assets_root,
        libraries,
        natives_dir,
        metadata,
        client_jar,
        no_assets: cli.no_assets,
    };

    let mut child = launch::launch(config)?;
    tracing::info!("Minecraft started (PID: {})", child.id());
    tracing::info!("Waiting for game to exit...");

    let status = child.wait()?;
    tracing::info!("Minecraft exited with status: {}", status);

    Ok(())
}
