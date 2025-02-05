use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq, Eq, clap::Parser)]
pub struct Args {
    #[arg(short, long)]
    pub config_path: Option<PathBuf>,
}
