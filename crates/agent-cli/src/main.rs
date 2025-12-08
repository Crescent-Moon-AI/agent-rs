//! Command-line interface for agent-rs

use clap::Parser;
use tracing::info;

#[derive(Parser, Debug)]
#[command(name = "agent-cli")]
#[command(about = "CLI for agent-rs framework", long_about = None)]
struct Args {
    /// Command to run
    #[arg(short, long)]
    command: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    agent_utils::init_tracing();

    let args = Args::parse();

    info!("Starting agent-cli");

    if let Some(command) = args.command {
        info!("Running command: {}", command);
        // TODO: Implement command execution
    } else {
        println!("Welcome to agent-rs CLI!");
        println!("Use --command to specify a command to run");
    }

    Ok(())
}
