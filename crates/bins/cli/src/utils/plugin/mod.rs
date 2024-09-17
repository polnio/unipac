mod error;

use crate::Result;
use abi_stable::library::RootModule;
pub use error::Error;
use std::path::Path;
use unipac_shared::SharedPlugin;

pub struct Plugin {
    pub name: &'static str,
}

impl Plugin {
    pub fn load(path: &str) -> Result<Self> {
        if !path.contains(".") {
            Self::load(format!("libunipac_{}_plugin.so", path).as_str())
        } else {
            let shared_plugin =
                SharedPlugin::load_from_file(Path::new(path)).map_err(Error::OpenLibrary)?;
            let name = shared_plugin
                .name()
                .ok_or(Error::SymbolNotFound("name"))?
                .into();

            Ok(Self { name })
        }
    }
}
