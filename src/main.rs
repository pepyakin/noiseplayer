mod daemon;
mod player;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Spawn the deamon.
    Spawn,
    /// Kill the daemon.
    Kill,
}

/// Uses clap to parse the command line arguments.
fn parse_cli() -> Commands {
    let cli = Cli::parse();

    match cli.command {
        Some(command) => command,
        None => {
            println!("No command given. Use --help to see available commands.");
            std::process::exit(1);
        }
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    match parse_cli() {
        Commands::Spawn => daemon::spawn()?,
        Commands::Kill => daemon::kill()?,
    }

    Ok(())
}
