#![allow(dead_code)]

mod manifest;
mod download;
mod library;
mod assets;
mod launch;

use std::path::PathBuf;
use clap::Parser;
use tracing_subscriber::prelude::*;

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

    /// Only download files, don't launch the game
    #[arg(long)]
    no_launch: bool,

    #[arg(long, default_value = ".minecraft")]
    game_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let log_dir = PathBuf::from("logs/mc-launcher-cli");
    tokio::fs::create_dir_all(&log_dir).await?;
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let log_file = log_dir.join(format!("launcher-{}.log", ts));
    let file = std::fs::File::create(&log_file)?;
    let (non_blocking, _guard) = tracing_appender::non_blocking(file);

    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info".into());

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(filter.clone()),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_filter(filter),
        )
        .init();

    let cli = Cli::parse();
    let game_dir = cli.game_dir.canonicalize().unwrap_or(cli.game_dir.clone());
    let cache = download::Cache::new(game_dir.join(".cache"));

    // Resolve version ID
    let version_id = if cli.version == "latest" {
        // Need manifest for "latest"
        let manifest_data = tokio::fs::read_to_string(
            &cache.get(MANIFEST_URL, None).await?,
        ).await?;
        let manifest: manifest::VersionManifest = serde_json::from_str(&manifest_data)?;
        manifest.latest.release.clone()
    } else {
        cli.version.clone()
    };

    let versions_dir = game_dir.join("versions").join(&version_id);
    let version_json = versions_dir.join(format!("{}.json", version_id));
    let client_jar = versions_dir.join(format!("{}.jar", version_id));

    let metadata = if version_json.exists() {
        // Reuse existing version metadata
        tracing::info!("Using existing metadata: {}", version_json.display());
        let data = tokio::fs::read_to_string(&version_json).await?;
        serde_json::from_str(&data)?
    } else {
        // Fetch from manifest
        let manifest_data = tokio::fs::read_to_string(
            &cache.get(MANIFEST_URL, None).await?,
        ).await?;
        let manifest: manifest::VersionManifest = serde_json::from_str(&manifest_data)?;

        let entry = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| anyhow::anyhow!("Version '{}' not found", version_id))?;

        tracing::info!("Fetching metadata for {}...", version_id);
        let meta_data = tokio::fs::read_to_string(
            &cache.get(&entry.url, None).await?,
        ).await?;
        let meta: manifest::VersionMetadata = serde_json::from_str(&meta_data)?;

        // Save for next time
        tokio::fs::create_dir_all(&versions_dir).await?;
        tokio::fs::write(&version_json, &meta_data).await?;

        meta
    };

    tracing::info!("  Type: {}, Java: {}",
        metadata.kind, metadata.java_version.major_version);

    // Download client jar if missing
    if !client_jar.exists() {
        tracing::info!("Downloading client jar...");
        cache
            .download_to(
                &metadata.downloads.client.url,
                &client_jar,
                Some(&metadata.downloads.client.sha1),
            )
            .await?;
    } else {
        tracing::info!("Client jar exists: {}", client_jar.display());
    }

    // Resolve libraries + natives
    tracing::info!("Resolving libraries...");
    let natives_arch = format!("natives-{}-{}", std::env::consts::OS, std::env::consts::ARCH);
    let natives_dir = versions_dir.join(&natives_arch);
    // Also check HMCL-style path (natives-linux-x86_64) for existing installations
    let hmcl_natives = versions_dir.join("natives-linux-x86_64");
    let natives_dir = if hmcl_natives.exists() { hmcl_natives } else { natives_dir };
    let (libraries, _) = library::resolve_libraries(
        &cache, &metadata, &game_dir.join("libraries"), &natives_dir,
    ).await?;
    tracing::info!("  {} libraries", libraries.len());

    // Assets
    let assets_root = game_dir.join("assets");

    if !cli.no_assets {
        tracing::info!("Downloading assets...");
        assets::ensure_assets(&cache, &metadata, &assets_root).await?;
    } else {
        tracing::info!("Skipping assets (--no-assets)");
    }

    // Launch (unless --no-launch)
    if cli.no_launch {
        tracing::info!("Download complete (--no-launch). Skipping game launch.");
        return Ok(());
    }

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
