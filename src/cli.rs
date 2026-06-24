use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "residuality")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Init,
    Component {
        #[command(subcommand)]
        action: ComponentAction,
    },
    Stressor {
        #[command(subcommand)]
        action: StressorAction,
    },
    Matrix,
    Triggers,
    Test {
        file: String,
    },
}

#[derive(Subcommand)]
pub enum ComponentAction {
    Add { id: String, name: String },
}

#[derive(Subcommand)]
pub enum StressorAction {
    // Placeholder for now; we'll grow this when the Stressor struct comes back.
    Add { id: String, name: String },
}
