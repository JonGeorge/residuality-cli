use clap::Parser;

mod cli;
mod commands;
mod model;
mod storage;

use cli::Commands;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Init => {
            println!("Initialized...");
            Ok(())
        }

        Commands::Component { action } => {
            commands::component::run(action)?;
            Ok(())
        }

        Commands::Stressor { action } => {
            commands::stressor::run(action)?;
            Ok(())
        }

        Commands::Matrix => {
            commands::matrix::run()?;
            Ok(())
        }

        _ => {
            eprintln!("not implemented yet");
            Ok(())
        }
    }
}
