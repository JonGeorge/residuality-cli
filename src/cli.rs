use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version)]
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
    Add { id: String, name: Option<String> },

    #[command(alias = "ls")]
    List,
}

#[derive(Subcommand)]
pub enum StressorAction {
    Add {
        #[arg(long)]
        id: String,

        #[arg(long)]
        name: Option<String>,

        #[arg(long)]
        detection: Option<String>,

        #[arg(long)]
        attractor: Option<String>,

        #[arg(long)]
        business_reaction: Option<String>,

        #[arg(long)]
        technical_change: Option<String>,

        #[arg(long = "affects", value_delimiter = ';')]
        affected_components: Vec<String>,
    },

    #[command(alias = "ls")]
    List,
}
