mod config;
mod error;

mod cli;

use std::{path::Path, sync::LazyLock};

use cli::Cli;
pub use error::Result;

#[macro_use]
extern crate tracing;

#[macro_use]
extern crate tabled;

/// A static reference to the project directory.
pub static PROJECT_DIR: LazyLock<&'static Path> = LazyLock::new(|| {
	let base_path = dirs::home_dir().expect("home directory not found!").join(".cmdlink");

	// Leak the path as a static reference, using into_boxed_path to trim the excess
	// capacity
	Box::leak(base_path.into_boxed_path())
});

fn main() {
	if let Err(e) = Cli::run() {
		eprintln!("fatal error occurred: {}", e);
	}
}
