mod info;
mod install;
mod list_packages;
mod remove;
mod search;

pub use info::info;
pub use install::install;
pub use list_packages::list_packages;
pub use remove::remove;
pub use search::search;

use crate::spinners::Spinners;
use crate::{Args, Config, Package, Plugin};
use anyhow::{bail, Context as _, Result};
use std::sync::mpsc;
use tabled::settings::Style;

struct PluginResult<T> {
    id: String,
    name: String,
    color: String,
    data: T,
}

pub(self) fn fetch<T: Send + 'static>(
    f: impl Fn(&Plugin) -> Result<T> + Sync + Send + 'static,
) -> Vec<Result<PluginResult<T>>> {
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
            let color = match plugin.get_color().context("Failed to get color") {
                Ok(color) => color,
                Err(err) => return Some(Err(err)),
            };
            let style = if args.colors {
                console::Style::from_dotted_str(&color)
            } else {
                console::Style::default()
            };
            let spinner = spinners.add(style.apply_to(&name).to_string());
            let s = spinner.clone();
            let pbh = std::thread::spawn(move || s.watch_events(event_receiver));
            let f = f.clone();
            Some(anyhow::Ok(std::thread::spawn(move || {
                let packages = f(&plugin);
                pbh.join().unwrap();
                match packages {
                    Ok(packages) => {
                        spinner.success();
                        // Ok((id, name, packages))
                        Ok(PluginResult {
                            id,
                            name,
                            color,
                            data: packages,
                        })
                    }
                    Err(err) => {
                        spinner.error();
                        let message = format!("[{}] {:#}", name, err);
                        Err(anyhow::anyhow!("{}", style.apply_to(message)))
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

pub(self) fn fetch_id<T>(
    id: String,
    f: impl Fn(&Plugin) -> Result<T>,
) -> Result<(String, String, T)> {
    let args = Args::get();
    if !args.plugins.is_empty() && !args.plugins.contains(&id) {
        bail!("Plugin {} not enabled", id);
    }
    let config = Config::get();
    let (plugin, event_receiver) = config
        .general
        .plugins
        .iter()
        .find_map(|plugin| {
            let (event_sender, event_receiver) = mpsc::channel();
            let plugin = Plugin::builder()
                .path(plugin.clone())
                .event_sender(event_sender)
                .build();
            match plugin.get_id().context("Failed to get id") {
                Ok(i) if i == id => Some((plugin, event_receiver)),
                Ok(_) => None,
                Err(err) => {
                    eprintln!("Error: {:#}", err);
                    None
                }
            }
        })
        .context("Failed to find plugin")?;

    let name = plugin.get_name().context("Failed to get name")?;
    let color = plugin.get_color().unwrap_or_default();

    let spinners = Spinners::new();
    let style = if args.colors && !color.is_empty() {
        console::Style::from_dotted_str(&color)
    } else {
        console::Style::default()
    };
    let spinner = spinners.add(style.apply_to(&name).to_string());
    let s = spinner.clone();
    let pbh = std::thread::spawn(move || s.watch_events(event_receiver));
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
}

pub(self) fn fetch_multiple(f: impl Fn(&Plugin) -> Result<Vec<Package>> + Sync + Send + 'static) {
    let handles = fetch(f);
    let args = Args::get();
    for handle in handles {
        match handle {
            Ok(result) => {
                let style = if args.colors {
                    console::Style::from_dotted_str(&result.color)
                } else {
                    console::Style::default()
                };
                if result.data.is_empty() {
                    let message = format!("No packages found for {}", result.name);
                    println!("{}", style.apply_to(message));
                    continue;
                }
                let mut table = Package::list_into_tab(result.data);
                table.with(Style::modern_rounded());
                let str = format!("{}\n{}\n", result.name, table);
                println!("{}", style.apply_to(str));
            }
            Err(err) => {
                let error = if args.colors {
                    console::style("Error").red().to_string()
                } else {
                    "Error".to_owned()
                };
                eprintln!("{}: {:#}", error, err);
            }
        }
    }
}

pub(self) fn fetch_one(f: impl Fn(&Plugin) -> Result<Option<Package>> + Sync + Send + 'static) {
    let handles = fetch(f);
    for handle in handles {
        match handle {
            Ok(result) => {
                let Some(package) = result.data else {
                    println!("Package not found for {}", result.name);
                    continue;
                };
                let mut table = package.into_tab();
                table.with(Style::modern_rounded());
                println!("{}\n{}\n", result.name, table);
            }
            Err(err) => {
                eprintln!("Error: {:#}", err);
            }
        }
    }
}
