use clap::{Parser, Subcommand};

mod scan_lan;
mod status;

#[derive(Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Ping a Minecraft server
    Ping { address: String },
    /// Query server status
    Status { address: String },
    /// Scan local network for Minecraft LAN games
    ScanLan {
        #[arg(default_value = "5")]
        duration: u64,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();
    match cli.command {
        Command::Ping { address } => {
            let report = status::query_server_status(&address).await?;
            println!("Pong from {} in {} ms", address, report.latency_ms);
        }
        Command::Status { address } => {
            let report = status::query_server_status(&address).await?;
            println!("Address: {}", report.address);
            println!(
                "Version: {} (protocol {})",
                report.version_name, report.protocol_version
            );
            println!("Players: {}/{}", report.players_online, report.players_max);
            println!("Description: {}", report.description);
            println!("Latency: {} ms", report.latency_ms);
        }
        Command::ScanLan { duration } => {
            scan_lan::run(duration);
        }
    }
    Ok(())
}
