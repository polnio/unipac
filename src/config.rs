use anyhow::{Context as _, Result};
use serde::Deserialize;
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

impl Config {
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let contents = std::fs::read_to_string(path).context("Failed to read config file")?;
        let config = toml::from_str(&contents).context("Failed to parse config file")?;
        Ok(config)
    }
}
