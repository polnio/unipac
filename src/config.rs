use crate::PROJECT_DIRS;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::borrow::Cow;
use std::path::Path;
use std::sync::OnceLock;

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

static CONFIG: OnceLock<Config> = OnceLock::new();

impl Config {
    pub fn init(this: Self) {
        let _ = CONFIG.set(this);
    }
    pub fn get() -> &'static Config {
        unsafe { CONFIG.get().unwrap_unchecked() }
    }
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read config file")?;
        let config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }
    pub fn from_opt_file(path: Option<&Path>) -> Result<Self> {
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
