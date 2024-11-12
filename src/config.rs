use std::collections::HashMap;

use crate::error::Error;
use crate::Result;
use serde::{Deserialize, Serialize};

type AliasName = String;

#[derive(Debug, Serialize, Deserialize)]
/// Configuration file for Cmdlink.
pub struct Config {
    /// List of aliases defined in the config.toml file.
    aliases: Aliases,
}

#[derive(Debug, Default, Serialize, Deserialize)]
/// Aliases defined in the config.toml file.
struct Aliases {
    #[serde(flatten)]
    all: HashMap<AliasName, AliasValues>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AliasValues {
    /// An optional description for the alias.
    description: Option<String>,
    /// The command to be executed when the alias is invoked.
    cmd: String,
}

impl Config {
    /// Creates an empty Config instance.
    fn empty() -> Self {
        Self {
            aliases: Aliases::default(),
        }
    }
    /// Creates a new Config instance from the config.toml file.
    ///
    /// If the config.toml file does not exist, it creates a new one with default values.
    pub fn new() -> Result<Self> {
        let config_file_path = crate::PROJECT_DIR.join("config.toml");

        // If the config.toml file does not exist, create a new one with default values.
        if !config_file_path.exists() {
            let cfg = Config::empty();
            let cfg_bytes = toml::to_string(&cfg)?.into_bytes();

            // Write the default config to the config.toml file.
            std::fs::write(config_file_path, cfg_bytes).map_err(Error::ConfigWrite)?;
            return Ok(cfg);
        }

        // Otherwise, open the file and read the contents to a Config instance.
        let config_str = std::fs::read_to_string(config_file_path).map_err(Error::ConfigRead)?;
        Ok(toml::from_str(&config_str)?)
    }
}
