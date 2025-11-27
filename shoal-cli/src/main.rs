use anyhow::Result;
use clap::{Parser, Subcommand};
use shoal_core::{self, create_shoal_manager};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(name = "shoal")]
#[command(version, about = "Local stack orchestrator")]
struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Up { stack_name: String },
    Down { stack_name: String },
}

fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new("debug"))
        .init();

    let args = Args::parse();
    let shoal_manager = create_shoal_manager().unwrap();

    match args.command {
        Commands::Up { stack_name } => shoal_manager.up(stack_name),
        Commands::Down { stack_name } => shoal_manager.down(stack_name),
    }
}
