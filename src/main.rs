mod args;
mod config;
mod dirs;
mod package;
mod plugin;
mod prelude;
mod spinners;

pub use args::Args;
pub use config::Config;
pub use dirs::PROJECT_DIRS;
pub use package::Package;
pub use plugin::Plugin;

use crate::prelude::*;
use crate::spinners::Spinners;
use anyhow::{Context as _, Result};
use clap::Parser as _;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    let config = Config::from_opt_file(args.config_path)?;
    let spinners = Spinners::new();

    let handles = config
        .general
        .plugins
        .into_iter()
        .map(|plugin| {
            let (progress_sender, mut progress_receiver) = mpsc::channel(100);
            let (end_sender, mut end_receiver) = mpsc::channel(100);
            let plugin = Plugin::builder()
                .path(plugin)
                .progress_sender(progress_sender)
                .end_sender(end_sender)
                .build();
            let id = plugin
                .get_id()
                .await_blocking()
                .context("Failed to get id")?;
            let name = plugin
                .get_name()
                .await_blocking()
                .context("Failed to get name")?;
            let spinners = spinners.clone();
            let spinner = spinners.add(name.clone());
            anyhow::Ok(async move {
                let pbh = async {
                    loop {
                        tokio::select! {
                            _ = end_receiver.recv() => break,
                            Some(progress) = progress_receiver.recv() => {
                                spinner.set(progress);
                                if progress == 100 {
                                    spinner.finish();
                                    break;
                                }
                            }
                        };
                    }
                };
                let ph = async {
                    plugin
                        .list_packages()
                        .await
                        .context("Failed to list packages")
                };
                let (_, packages) = tokio::join!(pbh, ph);
                match packages {
                    Ok(packages) => Some((id, name, packages)),
                    Err(err) => {
                        spinners.clear().unwrap();
                        eprintln!("[{}] Error: {:#}", name, err);
                        None
                    }
                }
            })
        })
        .filter_map(|r| match r {
            Ok(r) => Some(r),
            Err(err) => {
                spinners.clear().unwrap();
                eprintln!("Error: {:#}", err);
                None
            }
        })
        .to_set()
        .join_all()
        .await
        .into_iter()
        .filter_map(|r| r)
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
