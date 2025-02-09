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
use std::future::Future;
use tokio::runtime::Handle;
use tokio::task::JoinSet;

trait IteratorExt<T> {
    fn to_set(self) -> JoinSet<T>;
}
impl<I, F, T> IteratorExt<T> for I
where
    T: Send + 'static,
    F: Future<Output = T> + Send + 'static,
    I: Iterator<Item = F> + Sized,
{
    fn to_set(self) -> JoinSet<T> {
        JoinSet::from_iter(self)
    }
}

trait FutureExt<T> {
    fn await_blocking(self) -> T;
}
impl<T, F: Future<Output = T> + Send> FutureExt<T> for F {
    fn await_blocking(self) -> T {
        tokio::task::block_in_place(|| Handle::current().block_on(self))
    }
}

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
            // DEBUG: Simulate different speeds
            // std::thread::sleep(std::time::Duration::from_secs(2));
            let (progress_sender, mut progress_receiver) = tokio::sync::mpsc::channel(100);
            let (end_sender, mut end_receiver) = tokio::sync::mpsc::channel(100);
            let plugin = Plugin::new(plugin, progress_sender, end_sender);
            let id = plugin
                .get_id()
                .await_blocking()
                .context("Failed to get id")?;
            let name = plugin
                .get_name()
                .await_blocking()
                .context("Failed to get name")?;
            let spinner = spinners.add(name.clone());
            spinner.set(0);
            let spinners = spinners.clone();
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
