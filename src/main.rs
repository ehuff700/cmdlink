mod config;
mod error;

use config::Config;
pub use error::Result;
use std::{path::Path, sync::LazyLock};
use tracing::level_filters::LevelFilter;

/// A static reference to the project directory.
pub static PROJECT_DIR: LazyLock<&'static Path> = LazyLock::new(|| {
    let base_path = dirs::home_dir()
        .expect("home directory not found!")
        .join(".cmdlink");

    // Leak the path as a static reference, using into_boxed_path to trim the excess capacity
    Box::leak(base_path.into_boxed_path())
});

/// Setup logging using tracing library
fn setup_logging() {
    // todo: load logging from config?
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::TRACE)
        .init();
}

fn entry() -> Result<()> {
    setup_logging();
    let cfg = Config::new()?;
    Ok(())
}

fn main() {
    if let Err(e) = entry() {
        eprintln!("fatal error occurred: {}", e);
    }
}
