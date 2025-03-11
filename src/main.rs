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

use crate::spinners::Spinners;
use anyhow::{Context as _, Result};
use clap::Parser as _;
use plugin::Event;
use std::sync::mpsc;

fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_opt_file(args.config_path)?;
    let spinners = Spinners::new();

    let handles = config
        .general
        .plugins
        .into_iter()
        .map(|plugin| {
            let (event_sender, event_receiver) = mpsc::channel();
            let plugin = Plugin::builder()
                .path(plugin)
                .event_sender(event_sender)
                .build();
            let id = plugin.get_id().context("Failed to get id")?;
            let name = plugin.get_name().context("Failed to get name")?;
            let spinners = spinners.clone();
            let spinner = spinners.add(name.clone());
            let pbh = std::thread::spawn(move || {
                while let Ok(event) = event_receiver.recv() {
                    match event {
                        Event::End => break,
                        Event::Progress(progress) => {
                            spinner.set(progress);
                            if progress == 100 {
                                spinner.finish();
                                break;
                            }
                        }
                    }
                }
            });
            anyhow::Ok(std::thread::spawn(move || {
                let packages = plugin.list_packages().context("Failed to list packages");
                pbh.join().unwrap();
                match packages {
                    Ok(packages) => Some((id, name, packages)),
                    Err(err) => {
                        spinners.clear().unwrap();
                        eprintln!("[{}] Error: {:#}", name, err);
                        None
                    }
                }
            }))
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
        .filter_map(|r| r.join().ok().flatten())
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
