use std::{collections::HashMap, sync::mpsc::channel};

use serde::{Deserialize, Serialize};
use tabled::{settings::Style, Table};

use crate::{
	error::Error,
	platform_binary::{Action, Link, PlatformBinary},
	Result,
};

type AliasName = String;

#[derive(Tabled)]
/// Helper struct to display alias information in a table format.
struct AliasInfo<'a> {
	#[tabled(rename = "Alias")]
	alias: &'a str,
	#[tabled(rename = "Description")]
	description: &'a str,
}

#[derive(Default, Debug, Serialize, Deserialize)]
/// Configuration file for Cmdlink.
pub struct Config {
	#[serde(skip, default)]
	/// Whether or not the config.toml file has been changed since load.
	changed: bool,
	/// List of aliases defined in the config.toml file.
	aliases: HashMap<AliasName, AliasValues>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasValues {
	#[serde(skip)]
	pub link: Option<PlatformBinary>,
	/// An optional description for the alias.
	pub description: Option<String>,
	/// The command to be executed when the alias is invoked.
	pub cmd: String,
}

impl Config {
	/// Creates an empty Config instance.
	fn empty() -> Self { Config::default() }

	/// Creates a new Config instance from the config.toml file.
	///
	/// If the config.toml file does not exist, it creates a new one with
	/// default values.
	pub fn new() -> Result<Self> {
		let config_file_path = crate::PROJECT_DIR.join("config.toml");

		// If the config.toml file does not exist, create a new one with default values.
		if !config_file_path.exists() {
			let mut cfg = Config::empty();
			cfg.save()?;
			return Ok(cfg);
		}

		// Otherwise, open the file and read the contents to a Config instance.
		let config_str = std::fs::read_to_string(config_file_path).map_err(Error::ConfigRead)?;
		let mut cfg: Self = toml::from_str(&config_str)?;
		cfg.initialize_links()?;

		Ok(cfg)
	}

	/// Inserts a new alias to the config.toml file.
	pub fn create_alias(&mut self, alias: String, cmd: String, description: Option<String>, force: bool) -> Result<()> {
		let action = if force { Action::Update } else { Action::Create };
		if force && self.aliases.contains_key(&alias) {
			info!("Alias already exists, overriding...");
		}

		let link = Some(PlatformBinary::new(alias.clone(), cmd.clone(), action));
		self.aliases.insert(alias, AliasValues { link, description, cmd });
		self.changed = true;
		Ok(())
	}

	/// Removes an alias, marking the config as changed.
	pub fn remove_alias(&mut self, alias: &str) -> Result<()> {
		if let Some(old_alias) = self.aliases.get_mut(alias) {
			// SAFETY: all links are initialized in Config creation
			let link = unsafe { old_alias.link.as_mut().unwrap_unchecked() };
			link.set_action(Action::Remove);
			self.changed = true;
		} else {
			warn!("Alias \"{}\" did not exist in the config", alias);
		}
		Ok(())
	}

	/// Prints all the aliases defined in the config.toml file.
	pub fn display_aliases(&self) {
		if self.aliases.is_empty() {
			info!("No aliases available.");
			return;
		}
		info!("Available aliases:");

		let alias_iter = self.aliases.iter().map(|(alias, v)| AliasInfo {
			alias,
			description: v.description.as_deref().unwrap_or(&v.cmd),
		});
		let mut table = Table::new(alias_iter);
		table.with(Style::rounded()); // TODO: explore styling changes

		println!("{}", table);
	}

	/// Refreshes all the bad links, setting the action to Create for any links
	/// that do not exist.
	pub fn refresh_links(&mut self) -> Result<()> {
		info!("Refreshing command links...");

		for alias_values in self.aliases.values_mut() {
			if let Some(link) = alias_values.link.as_mut() {
				if !link.exists() {
					debug!("Bad link for alias: {}", link.alias());
					link.set_action(Action::Create);
				}
			}
		}
		self.changed = true;
		Ok(())
	}

	/// Saves the current Config instance to the config.toml file.
	fn save(&mut self) -> Result<()> {
		self.save_links()?;
		let config_file_path = crate::PROJECT_DIR.join("config.toml");
		let cfg_bytes = toml::to_string(&self)?.into_bytes();
		std::fs::write(config_file_path, cfg_bytes).map_err(Error::ConfigWrite)
	}

	/// Saves link changes, if any, to the platform binary files.
	fn save_links(&mut self) -> Result<()> {
		let (tx, rx) = channel();

		for alias_values in self.aliases.values_mut() {
			// Safetey: all links are initialized in Config creation
			let link = unsafe { alias_values.link.as_mut().unwrap_unchecked() };
			if !matches!(link.action(), Action::None) {
				link.perform_action()?;
			}
			if matches!(link.action(), Action::Remove) {
				debug!("Removing link for alias: {}", link.alias());
				let _ = tx.send(link.alias().to_string());
			}
		}
		drop(tx);
		while let Ok(alias) = rx.recv() {
			trace!("Removed link for alias: {}", alias);
			self.aliases.remove(&alias);
		}

		Ok(())
	}

	/// Initializes the links for all aliases defined in the config.toml file.
	fn initialize_links(&mut self) -> Result<()> {
		for (alias, AliasValues { link, cmd, .. }) in self.aliases.iter_mut() {
			let platform_binary = PlatformBinary::new(alias.to_string(), cmd.to_string(), Action::None);

			if !platform_binary.exists() {
				warn!(
					"Platform binary file for alias \"{}\" not found. Either the binary files were deleted, or the config was updated manually. Run [refresh] command to refresh config and create links.",
					alias
				);
			}
			*link = Some(platform_binary);
		}

		Ok(())
	}
}

impl Drop for Config {
	fn drop(&mut self) {
		if self.changed {
			if let Err(why) = self.save() {
				error!("Config Save Error: {why}");
			} else {
				info!("Configuration changes saved successfully");
			}
		}
	}
}
