use crate::PROJECT_DIRS;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::borrow::Cow;
use std::path::Path;
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: ConfigGeneral,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct ConfigGeneral {
    #[serde(default)]
    pub plugins: Vec<String>,
}

static mut CONFIG: Option<Config> = None;

impl Config {
    pub fn init_from_opt_file(path: Option<&Path>) -> Result<()> {
        let config = Config::from_opt_file(path)?;
        unsafe {
            CONFIG = Some(config);
        }
        Ok(())
    }
    pub fn get() -> &'static Config {
        unsafe { CONFIG.as_ref().unwrap_unchecked() }
    }
    fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read config file")?;
        let config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }
    fn from_opt_file(path: Option<&Path>) -> Result<Self> {
        let is_default = path.is_none();
        let path = path
            .map(Cow::from)
            .unwrap_or_else(|| PROJECT_DIRS.config_dir().join("config.toml").into());

        if is_default && !path.exists() {
            return Ok(Config::default());
        }

        let config = Config::from_file(&path).context("Failed to parse config file")?;
        Ok(config)
    }
}
