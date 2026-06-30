use clap::Parser;

mod cli;
mod commands;
mod model;
mod storage;

use cli::Commands;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
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
