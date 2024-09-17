mod error;

pub use error::Error;
// use once_cell::sync::Lazy;

use super::UNIPAC_DIR;
use crate::Result;
use serde::Deserialize;

/* static CONFIG: Lazy<Config> = Lazy::new(|| match Config::new() {
    Ok(config) => config,
    Err(error) => {
        eprintln!("{:?}", error);
        std::process::exit(1);
    }
}); */

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub plugins: Vec<String>,
}

impl Config {
    pub fn new() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(Self::get_file()?.required(false))
            .add_source(config::Environment::with_prefix("UNIPAC"))
            .build()
            .map_err(Error::Build)?;

        let serialized = config.try_deserialize().map_err(Error::Build)?;

        Ok(serialized)
    }

    fn get_file() -> Result<config::File<config::FileSourceFile, config::FileFormat>> {
        let config_file_path = UNIPAC_DIR.config_dir().join("config");
        let config_file_path = config_file_path.to_str().ok_or(Error::NonUtf8FileName)?;
        let config_file = config::File::with_name(config_file_path);

        Ok(config_file)
    }
}
