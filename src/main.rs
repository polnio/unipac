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

use anyhow::Result;

fn main() -> Result<()> {
    Args::init(Args::parse());
    let args = Args::get();
    Config::init(Config::from_opt_file(args.config_path.as_deref())?);

    match &args.command {
        args::Command::List { updates: true } => commands::list_updates(),
        args::Command::List { .. } => commands::list_packages(),
        args::Command::Search { query } => commands::search(query),
        args::Command::Info { pname } => commands::info(pname),
        args::Command::Install { pname } => commands::install(pname),
        args::Command::Remove { pname } => commands::remove(pname),
        args::Command::Update { list: true } => commands::list_updates(),
        args::Command::Update { .. } => commands::update(),
    }

    Ok(())
}
