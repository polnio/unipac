use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, clap::Subcommand)]
pub enum Command {
    List,
    Search { query: String },
}
