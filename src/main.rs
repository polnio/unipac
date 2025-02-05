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
use indicatif::ProgressBar;

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
        let (progress_sender, progress_receiver) = std::sync::mpsc::channel();
        let plugin = Plugin::new(plugin, progress_sender);
        let id = plugin.get_id().context("Failed to get id")?;
        let name = plugin.get_name().context("Failed to get name")?;
        let pbh = std::thread::spawn(move || {
            let pb = ProgressBar::new(100);
            while let Ok(progress) = progress_receiver.recv() {
                pb.set_position(progress as u64);
                if progress == 100 {
                    pb.finish();
                    break;
                }
            }
        });
        let packages = plugin.list_packages().context("Failed to list packages")?;
        pbh.join().unwrap();
        println!(
            "------\nid: {}\nname: {}\npackages: {:?}",
            id, name, packages
        );
    }

    Ok(())
}
