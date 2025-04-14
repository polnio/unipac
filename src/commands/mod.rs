mod list_packages;
mod search;

pub use list_packages::list_packages;
pub use search::search;

use crate::plugin::Event;
use crate::spinners::Spinners;
use crate::{Config, Package, Plugin};
use anyhow::{Context as _, Result};
use std::sync::mpsc;
use tabled::settings::Style;

pub(self) fn fetch_multiple(
    config: Config,
    f: impl Fn(&Plugin) -> Result<Vec<Package>> + Sync + Send + 'static,
) {
    let spinners = Spinners::new();
    let f = std::sync::Arc::new(f);
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
            let f = f.clone();
            anyhow::Ok(std::thread::spawn(move || {
                let packages = f(&plugin);
                pbh.join().unwrap();
                match packages {
                    Ok(packages) => Ok((id, name, packages)),
                    Err(err) => Err(anyhow::anyhow!("[{}] {:#}", name, err)),
                }
            }))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|r| r.and_then(|h| h.join().unwrap()))
        .collect::<Vec<_>>();

    spinners.clear().unwrap();

    for handle in handles {
        match handle {
            Ok((_id, name, packages)) => {
                if packages.is_empty() {
                    println!("No packages found for {}", name);
                    continue;
                }
                let mut table = Package::list_into_tab(packages);
                table.with(Style::modern_rounded());
                println!("{}\n{}\n", name, table);
            }
            Err(err) => {
                eprintln!("Error: {:#}", err);
            }
        }
    }
}
