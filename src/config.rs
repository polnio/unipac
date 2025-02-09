use crate::PROJECT_DIRS;
use anyhow::{Context as _, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

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

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read config file")?;
        let config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }
    pub fn from_opt_file(path: Option<PathBuf>) -> Result<Self> {
        let is_default = path.is_none();
        let path = path.unwrap_or_else(|| PROJECT_DIRS.config_dir().join("config.toml"));

        let config = match (Config::from_file(&path), is_default) {
            (Ok(config), _) => config,
            (Err(_), true) => Config::default(),
            (Err(err), false) => return Err(err),
        };

        Ok(config)
    }
}
