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
    let spinners = Spinners::new();

    let handles = config
        .general
        .plugins
        .into_iter()
        .map(|plugin| {
            // DEBUG: Simulate different speeds
            // std::thread::sleep(std::time::Duration::from_secs(2));
            let (progress_sender, progress_receiver) = std::sync::mpsc::channel();
            let plugin = Plugin::new(plugin, progress_sender);
            let id = plugin.get_id().context("Failed to get id")?;
            let name = plugin.get_name().context("Failed to get name")?;
            let spinner = spinners.add(name.clone());
            let pbh = std::thread::spawn(move || {
                while let Ok(progress) = progress_receiver.recv() {
                    spinner.set(progress);
                    if progress == 100 {
                        spinner.finish();
                        break;
                    }
                }
            });
            let ph = std::thread::spawn(move || {
                plugin.list_packages().context("Failed to list packages")
            });
            anyhow::Ok((id, name, pbh, ph))
        })
        .filter_map(|r| match r {
            Ok(r) => Some(r),
            Err(err) => {
                spinners.clear().unwrap();
                eprintln!("Error: {:#}", err);
                None
            }
        })
        .collect::<Vec<_>>()
        .into_iter()
        .filter_map(|(id, name, pbh, ph)| {
            let packages = ph.join().unwrap();
            pbh.join().unwrap();
            match packages {
                Ok(packages) => Some((id, name, packages)),
                Err(err) => {
                    spinners.clear().unwrap();
                    eprintln!("[{}] Error: {:#}", name, err);
                    None
                }
            }
        })
        .collect::<Vec<_>>();

    spinners.clear().unwrap();

    for (id, name, packages) in handles {
        println!(
            "------\nid: {}\nname: {}\npackages: {:?}",
            id, name, packages
        );
    }

    Ok(())
}
