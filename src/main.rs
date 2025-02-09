mod args;
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

use anyhow::{Context as _, Result};
use clap::Parser as _;
use spinners::Spinners;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_opt_file(args.config_path)?;
    let spinner = Spinners::new();

    for plugin in config.general.plugins {
        let (progress_sender, progress_receiver) = std::sync::mpsc::channel();
        let plugin = Plugin::new(plugin, progress_sender);
        let id = plugin.get_id().context("Failed to get id")?;
        let name = plugin.get_name().context("Failed to get name")?;
        let spinner = spinner.add(name.clone());
        let pbh = std::thread::spawn(move || {
            while let Ok(progress) = progress_receiver.recv() {
                spinner.set(progress);
                if progress == 100 {
                    spinner.finish();
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
