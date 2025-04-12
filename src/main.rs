mod args;
mod commands;
mod config;
mod dirs;
mod package;
mod plugin;
mod spinners;

pub use args::Args;
pub use config::Config;
pub use dirs::PROJECT_DIRS;
pub use package::Package;
pub use plugin::Plugin;

use crate::spinners::Spinners;
use anyhow::Result;
use clap::Parser as _;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_opt_file(args.config_path.as_deref())?;

    match args.command {
        args::Command::List => commands::list_packages(args, config),
    }

    Ok(())
}
