mod args;
mod config;
mod dirs;
mod package;
mod plugin;

pub use args::Args;
pub use config::Config;
pub use dirs::PROJECT_DIRS;
pub use package::Package;
pub use plugin::Plugin;

use anyhow::{Context as _, Result};
use clap::Parser as _;

fn main() -> Result<()> {
    let args = Args::parse();
    let is_default = args.config_path.is_none();
    let config_path = args
        .config_path
        .unwrap_or_else(|| PROJECT_DIRS.config_dir().join("config.toml"));

    let config = match (Config::from_file(&config_path), is_default) {
        (Ok(config), _) => config,
        (Err(_), true) => Config::default(),
        (Err(err), false) => return Err(err),
    };

    for plugin in config.general.plugins {
        let plugin = Plugin::new(plugin);
        let id = plugin.get_id().context("Failed to get id")?;
        let name = plugin.get_name().context("Failed to get name")?;
        let packages = plugin.list_packages().context("Failed to list packages")?;
        println!(
            "------\nid: {}\nname: {}\npackages: {:?}",
            id, name, packages
        );
    }

    Ok(())
}
