// https://www.osohq.com/post/rust-reflection-pt-1

mod utils;

use crate::utils::Plugin;
pub use utils::error::{Error, Result};

fn main() -> Result<()> {
    let config = utils::Config::new()?;
    for plugin in config.plugins {
        println!("plugin: {}", plugin);
        let plugin = Plugin::load(&plugin)?;
        println!("plugin loaded");
        println!("plugin name: {}", plugin.name);
    }

    Ok(())
}
