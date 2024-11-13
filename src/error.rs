use thiserror::Error;

#[derive(Error, Debug)]
/// Error container for all Cmdlink errors
pub enum Error {
	#[error("Failed to create project directory: {0}")]
	ProjectDirCreation(#[source] std::io::Error),
	#[error("Failed to read config file: {0}")]
	ConfigRead(#[source] std::io::Error),
	#[error("Error writing config data: {0}")]
	ConfigWrite(#[source] std::io::Error),
	#[error("Failed to parse config file: {0}")]
	ConfigParse(#[from] toml::de::Error),
	#[error("Failed to serialize config data: {0}")]
	ConfigSerialize(#[from] toml::ser::Error),
	#[error("Failed to create link for alias '{0}': {1}")]
	LinkCreation(String, #[source] std::io::Error),
	#[error("Alias '{0}' already exists")]
	LinkAlreadyExists(String),
	#[error("Failed to update link for alias '{0}': {1}")]
	LinkUpdate(String, #[source] std::io::Error),
	#[error("Failed to remove link for alias '{0}': {1}")]
	LinkRemoval(String, #[source] std::io::Error),
}

/// Cmdlink result type
pub type Result<T> = std::result::Result<T, Error>;
