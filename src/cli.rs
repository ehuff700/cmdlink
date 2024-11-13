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
	/// Refreshes cmdlink by retrieving the latest config file and updating the
	/// command links.
	Refresh,
	/// Displays all current aliases and their associated descriptions if any.
	Display,
	/// Adds a new command link to the config file, adding the appropriate bin.
	Add {
		alias: String,
		#[arg(short, long = "desc")]
		/// An optional description for the alias.
		description: Option<String>,
		#[arg(short, long)]
		/// The command to run in place of the alias.
		cmd: String,
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
		let mut cfg = Config::new()?;
		let cli = Cli::parse();
		cli.setup_logging();

		match cli.subcommand {
			Commands::Refresh => {
				info!("refreshing command links...");
			},
			Commands::Add {
				alias,
				description,
				cmd,
			} => cfg.insert_alias(&alias, &cmd, description)?,
			Commands::Remove { alias } => cfg.remove_alias(&alias)?,
			Commands::Display => cfg.print_aliases(),
		}
		Ok(())
	}
}
