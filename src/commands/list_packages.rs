use crate::plugin::Event;
use crate::{Args, Config, Plugin, Spinners};
use anyhow::Context as _;
use std::sync::mpsc;

pub fn list_packages(_args: Args, config: Config) {
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
}
