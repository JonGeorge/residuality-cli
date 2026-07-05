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
    Check,
    Component {
        #[command(subcommand)]
        action: ComponentAction,
    },
    Stressor {
        #[command(subcommand)]
        action: StressorAction,
    },
    Matrix {
        #[command(subcommand)]
        action: MatrixAction,
    },
    Triggers,
    Test {
        file: String,
    },
}

#[derive(Subcommand)]
pub enum ComponentAction {
    Add {
        id: String,
        name: Option<String>,
    },

    #[command(alias = "ls")]
    List,
}

#[derive(Subcommand)]
pub enum StressorAction {
    Add {
        #[arg(long)]
        id: Option<String>,

        #[arg(long, requires = "id")]
        name: Option<String>,

        #[arg(long, requires = "id")]
        detection: Option<String>,

        #[arg(long, requires = "id")]
        attractor: Option<String>,

        #[arg(long, requires = "id")]
        business_reaction: Option<String>,

        #[arg(long, requires = "id")]
        technical_change: Option<String>,

        #[arg(long = "affects", requires = "id", value_delimiter = ';')]
        affected_components: Vec<String>,
    },

    #[command(alias = "ls")]
    List,
}

#[derive(Subcommand)]
pub enum MatrixAction {
    Export,
    Print,
}
