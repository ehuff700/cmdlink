use thiserror::Error;

#[derive(Error, Debug)]
/// Error container for all Cmdlink errors
pub enum Error {
    #[error("failed to create project directory: {0}")]
    ProjectDirCreation(#[source] std::io::Error),
    #[error("failed to read config file: {0}")]
    ConfigRead(#[source] std::io::Error),
    #[error("error writing config data: {0}")]
    ConfigWrite(#[source] std::io::Error),
    #[error("failed to parse config file: {0}")]
    ConfigParse(#[from] toml::de::Error),
    #[error("failed to serialize config data: {0}")]
    ConfigSerialize(#[from] toml::ser::Error),
}

/// Cmdlink result type
pub type Result<T> = std::result::Result<T, Error>;
