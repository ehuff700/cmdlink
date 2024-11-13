use clap::{Args, Parser, Subcommand};
use tracing::level_filters::LevelFilter;

use crate::{config::Config, Result};

#[derive(Args, Debug)]
pub struct Verbosity {
	#[arg(short, long, action = clap::ArgAction::Count, global = true)]
	/// Increases the verbosity level. -v for INFO (default), -vv for DEBUG,
	/// -vvv for TRACE
	verbose: u8,
	#[arg(short, long, action = clap::ArgAction::Count, global = true, conflicts_with = "verbose")]
	/// Silences all logging
	quiet: u8,
}

impl Verbosity {
	/// Converts the verbosity settings into a tracing level filter.
	pub fn as_level_filter(&self) -> Option<LevelFilter> {
		if self.quiet > 0 {
			return None;
		}

		Some(match self.verbose.min(3) {
			2 => LevelFilter::DEBUG,
			3 => LevelFilter::TRACE,
			_ => LevelFilter::INFO, // Default to ERROR
		})
	}
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Cli {
	#[command(flatten)]
	verbose: Verbosity,
	#[command(subcommand)]
	pub subcommand: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
	/// Refreshes links by retrieving the latest config file and updating the
	/// associated binaries in the `bins` directory.
	Refresh,
	/// Displays all current aliases and their associated descriptions.
	Display,
	/// Adds a new command link to the config file, adding the appropriate bin
	/// to the `bins` directory.
	Add {
		/// The alias for the command link.
		alias: String,
		#[arg(short, long = "desc")]
		/// An optional description for the alias.
		description: Option<String>,
		#[arg(short, long)]
		/// The command to run in place of the alias.
		cmd: String,
		#[arg(short, long, default_value = "false")]
		/// Forces the creation of the alias even if it already exists.
		force: bool,
	},
	/// Removes a command link from the config file and bins.
	Remove { alias: String },
}

impl Cli {
	/// Sets up the logging configuration based on the verbosity settings.
	fn setup_logging(&self) {
		if let Some(filter) = self.verbose.as_level_filter() {
			tracing_subscriber::fmt().with_max_level(filter).init();
		}
	}

	/// Runs the CLI application by processing the provided command-line
	/// arguments.
	pub fn run() -> Result<()> {
		let cli = Cli::parse();
		cli.setup_logging();

		// Cfg must be after logging setup to ensure logging is initialized
		let mut cfg = Config::new()?;

		match cli.subcommand {
			Commands::Refresh => cfg.refresh_links()?,
			Commands::Add {
				alias,
				description,
				cmd,
				force,
			} => cfg.create_alias(alias, cmd, description, force)?,
			Commands::Remove { alias } => cfg.remove_alias(&alias)?,
			Commands::Display => cfg.display_aliases(),
		}
		Ok(())
	}
}
