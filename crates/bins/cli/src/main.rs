// https://www.osohq.com/post/rust-reflection-pt-1

mod commands;
mod utils;

use clap::Parser;
pub use utils::error::{Error, Result};

fn main() -> Result<()> {
    // let args = utils::Args::parse();
    // match args.command {
    //     utils::args::Command::List => commands::list(),
    // }

    let config = utils::Config::new()?;

    for plugin in config.plugins {
        println!("plugin: {}", plugin);
        let plugin = unipac_shared::Plugin::load(&plugin)?;
        println!("plugin loaded");
        // println!("plugin name: {}", plugin.name);
        let packages = plugin.list_packages();
        println!("{:?}\n", packages);
    }

    Ok(())
}
