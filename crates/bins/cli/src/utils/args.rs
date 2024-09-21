use clap::{Parser, Subcommand};

#[derive(Debug, Clone, PartialEq, Eq, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Subcommand)]
pub enum Command {
    List,
}
