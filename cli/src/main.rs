use clap::{Parser, Subcommand};

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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt().init();
    let cli = Cli::parse();
    match cli.command {
        Command::Ping { address } => {
            println!("Pinging {address}...");
        }
        Command::Status { address } => {
            println!("Querying {address}...");
        }
    }
    Ok(())
}
