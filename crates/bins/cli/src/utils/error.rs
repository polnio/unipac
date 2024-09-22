use super::config;
use derive_more::From;
use unipac_shared::plugin;

#[derive(Debug, From)]
pub enum Error {
    Plugin(plugin::Error),
    Config(config::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

/* impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for Error {} */
