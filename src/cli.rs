use crate::{config::Config, Result};
use clap::{Args, Parser, Subcommand};
use tracing::level_filters::LevelFilter;

#[derive(Args, Debug)]
pub struct Verbosity {
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    /// Increases the verbosity level. -v for WARN, -vv for INFO, -vvv for DEBUG, -vvvv for TRACE
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

        Some(match self.verbose.min(4) {
            1 => LevelFilter::WARN,
            2 => LevelFilter::INFO,
            3 => LevelFilter::DEBUG,
            4 => LevelFilter::TRACE,
            _ => LevelFilter::ERROR, // Default to ERROR
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
    /// Refreshes cmdlink by retrieving the latest config file and updating the command links.
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

    /// Runs the CLI application by processing the provided command-line arguments.
    pub fn run(cfg: &mut Config) -> Result<()> {
        let cli = Cli::parse();
        cli.setup_logging();

        match cli.subcommand {
            Commands::Refresh => {
                // let cfg = Config::new()?;
                info!("refreshing command links...");
            }
            Commands::Add {
                alias,
                description,
                cmd,
            } => {
                cfg.insert_alias(&alias, &cmd, description)?;
                info!("successfully added alias: {alias} for command: \"{cmd}\"");
            }
            Commands::Remove { alias } => {
                cfg.remove_alias(&alias)?;
                info!("successfully removed alias: {alias}");
            }
            Commands::Display => {
                let map = cfg.aliases();
                if map.is_empty() {
                    info!("cmdlink has no aliases available to display");
                    return Ok(());
                }

                println!("      Aliases       |     Description     ");
                for (k, v) in cfg.aliases().iter() {
                    let description = v.description.as_ref().unwrap_or(&v.cmd);
                    println!("      {k}         |       {description}       ")
                }
            }
        }
        Ok(())
    }
}
