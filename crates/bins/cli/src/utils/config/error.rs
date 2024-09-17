use derive_more::From;

#[derive(Debug, From)]
pub enum Error {
    NonUtf8FileName,
    Build(config::ConfigError),
    Serialize(),
}
