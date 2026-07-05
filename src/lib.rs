use clap::Parser;

mod analysis;
mod cli;
mod commands;
mod model;
mod storage;
mod views;

use cli::Commands;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = cli::Cli::parse();

    match cli.command {
        Commands::Init => {
            commands::init::run()?;
            Ok(())
        }

        Commands::Check => {
            commands::check::run()?;
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

        Commands::Matrix { action } => {
            commands::matrix::run(action)?;
            Ok(())
        }

        _ => {
            eprintln!("not implemented yet");
            Ok(())
        }
    }
}
