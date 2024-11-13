use std::{borrow::Cow, collections::HashMap, mem::MaybeUninit};

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
	changed: bool,
	/// List of aliases defined in the config.toml file.
	aliases: HashMap<AliasName, AliasValues>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AliasValues {
	#[serde(skip, default = "std::mem::MaybeUninit::uninit")]
	pub link: MaybeUninit<PlatformBinary<'static>>,
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
			let cfg = Config::empty();
			let cfg_bytes = toml::to_string(&cfg)?.into_bytes();

			// Write the default config to the config.toml file.
			std::fs::write(config_file_path, cfg_bytes).map_err(Error::ConfigWrite)?;
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
		if matches!(action, Action::Update) {
			info!("Alias already exists, overriding...");
		}

		self.aliases.insert(
			alias.clone(),
			AliasValues {
				link: MaybeUninit::new(PlatformBinary::new(alias, cmd.clone(), action)),
				description,
				cmd: cmd.clone(),
			},
		);
		self.changed = true;
		Ok(())
	}

	/// Removes an alias with the given alias name
	/// This function will automatically remove the associated links as well.
	pub fn remove_alias(&mut self, alias: &str) -> Result<()> {
		if self.aliases.remove(alias).is_none() {
			warn!("Alias \"{alias}\" did not exist in the config");
			return Ok(());
		};
		self.changed = true;
		Ok(())
	}

	/// Prints all the aliases defined in the config.toml file.
	pub fn display_aliases(&self) {
		if self.aliases.is_empty() {
			info!("cmdlink has no aliases available to display");
			return;
		}
		info!("Available aliases:");

		let alias_iter = self.aliases.iter().map(|(alias, v)| AliasInfo {
			alias,
			description: v.description.as_deref().unwrap_or(&v.cmd),
		});
		let mut table = Table::new(alias_iter);
		table.with(Style::rounded());

		println!("{}", table);
	}

	/// Refreshes all the bad links, setting the action to Create for any links
	/// that do not exist.
	pub fn refresh_links(&mut self) -> Result<()> {
		info!("Refreshing command links...");

		// SAFETY: All links are initialized during Config creation.
		for bad_link in self
			.aliases
			.values_mut()
			.map(|AliasValues { link, .. }| unsafe { link.assume_init_mut() })
			.filter(|l| !l.exists())
		{
			debug!("Bad link for alias: {}", bad_link.alias());
			bad_link.set_action(Action::Create);
		}
		self.changed = true;
		Ok(())
	}

	/// Saves the current Config instance to the config.toml file.
	fn save(&mut self) -> Result<()> {
		let config_file_path = crate::PROJECT_DIR.join("config.toml");
		let cfg_bytes = toml::to_string(&self)?.into_bytes();
		std::fs::write(config_file_path, cfg_bytes).map_err(Error::ConfigWrite)?;
		self.save_links()
	}

	/// Saves link changes, if any, to the platform binary files.
	fn save_links(&mut self) -> Result<()> {
		let link = self
			.aliases
			.values_mut()
			.map(|AliasValues { link, .. }| unsafe { link.assume_init_mut() })
			.find(|l| !matches!(l.action(), Action::None));

		if let Some(l) = link {
			l.perform_action()?;
		}
		Ok(())
	}

	/// Initializes the links for all aliases defined in the config.toml file.
	fn initialize_links(&mut self) -> Result<()> {
		for (alias, AliasValues { link, cmd, .. }) in self.aliases.iter_mut() {
			let platform_binary =
				PlatformBinary::new(Cow::Owned(alias.to_string()), Cow::Owned(cmd.to_string()), Action::None);

			if !platform_binary.exists() {
				warn!(
					"Platform binary file for alias \"{}\" not found. Either the binary files were deleted, or the config was updated manually. Run [refresh] command to refresh config and create links.",
					alias
				);
			}

			link.write(platform_binary);
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
