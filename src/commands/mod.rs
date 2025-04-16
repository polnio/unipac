mod info;
mod list_packages;
mod search;

pub use info::info;
pub use list_packages::list_packages;
pub use search::search;

use crate::plugin::Event;
use crate::spinners::Spinners;
use crate::{Args, Config, Package, Plugin};
use anyhow::{Context as _, Result};
use std::sync::mpsc;
use tabled::settings::Style;

fn fetch<T: Send + 'static>(
    f: impl Fn(&Plugin) -> Result<T> + Sync + Send + 'static,
) -> Vec<Result<(String, String, T)>> {
    let config = Config::get();
    let args = Args::get();
    let spinners = Spinners::new();
    let f = std::sync::Arc::new(f);
    let handles = config
        .general
        .plugins
        .iter()
        .filter_map(|plugin| {
            let (event_sender, event_receiver) = mpsc::channel();
            let plugin = Plugin::builder()
                .path(plugin.clone())
                .event_sender(event_sender)
                .build();
            let id = match plugin.get_id().context("Failed to get id") {
                Ok(id) => id,
                Err(err) => return Some(Err(err)),
            };
            if !args.plugins.is_empty() && !args.plugins.contains(&id) {
                return None;
            }
            let name = match plugin.get_name().context("Failed to get name") {
                Ok(name) => name,
                Err(err) => return Some(Err(err)),
            };
            let spinner = spinners.add(name.clone());
            let s = spinner.clone();
            let pbh = std::thread::spawn(move || {
                while let Ok(event) = event_receiver.recv() {
                    match event {
                        Event::End => break,
                        Event::Progress(progress) => {
                            s.set(progress);
                            if progress == 100 {
                                break;
                            }
                        }
                    }
                }
            });
            let f = f.clone();
            Some(anyhow::Ok(std::thread::spawn(move || {
                let packages = f(&plugin);
                pbh.join().unwrap();
                match packages {
                    Ok(packages) => {
                        spinner.success();
                        Ok((id, name, packages))
                    }
                    Err(err) => {
                        spinner.error();
                        Err(anyhow::anyhow!("[{}] {:#}", name, err))
                    }
                }
            })))
        })
        .collect::<Vec<_>>()
        .into_iter()
        .map(|r| r.and_then(|h| h.join().unwrap()))
        .collect::<Vec<_>>();

    spinners.clear().unwrap();
    handles
}

pub(self) fn fetch_multiple(f: impl Fn(&Plugin) -> Result<Vec<Package>> + Sync + Send + 'static) {
    let handles = fetch(f);
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

pub(self) fn fetch_one(f: impl Fn(&Plugin) -> Result<Option<Package>> + Sync + Send + 'static) {
    let handles = fetch(f);
    for handle in handles {
        match handle {
            Ok((_id, name, package)) => {
                let Some(package) = package else {
                    println!("Package not found for {}", name);
                    continue;
                };
                let mut table = package.into_tab();
                table.with(Style::modern_rounded());
                println!("{}\n{}\n", name, table);
            }
            Err(err) => {
                eprintln!("Error: {:#}", err);
            }
        }
    }
}
