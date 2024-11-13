use std::path::PathBuf;

/// Adds .cmdlink/bins to the user's PATH environment variable.
fn add_to_user_path(new_path: &str) -> Result<(), Box<dyn std::error::Error>> {
	#[cfg(target_os = "windows")]
	add_win_path(new_path)?;

	Ok(())
}

/// Setup the project directory
fn setup_project_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let base_path = dirs::home_dir().expect("home directory not found!").join(".cmdlink");

	// Create the project directory if it doesn't exist
	std::fs::create_dir_all(&base_path).map_err(|e| format!("error creating project directory: {e}"))?;

	// Create the bins directory within the project directory
	let bins_dir = base_path.join("bins");
	std::fs::create_dir_all(&bins_dir).map_err(|e| format!("error creating bins directory: {e}"))?;

	add_to_user_path(&bins_dir.display().to_string())?;
	Ok(base_path)
}

fn main() {
	// Setup the project directory
	if let Err(err) = setup_project_dir() {
		panic!("error setting up project directory: {}", err);
	}
}

#[cfg(target_os = "windows")]
/// Adds a new path to the user's PATH environment variable on Windows.
fn add_win_path(new_path: &str) -> Result<(), Box<dyn std::error::Error>> {
	use std::{ffi::OsString, os::windows::ffi::OsStrExt};

	use base64::{engine::general_purpose, Engine};
	use windows_registry::CURRENT_USER;

	let environment_key = CURRENT_USER.open("Environment")?;

	// Get the current PATH value
	let current_path = environment_key.get_string("Path").unwrap();

	// Check if the new path is already in the PATH to avoid duplicates
	if current_path.split(';').any(|p| p == new_path) {
		return Ok(());
	}

	let updated_path = if current_path.is_empty() {
		new_path.to_string()
	} else {
		format!("{}{}", current_path, new_path)
	};

	let ps_command = format!("[Environment]::SetEnvironmentVariable('PATH', '{}', 'User')", updated_path);

	// Encode the powershell command as UTF-16LE base64 and run
	let utf_16_bytes: Vec<u8> = OsString::from(ps_command)
		.encode_wide()
		.flat_map(|u| u.to_le_bytes())
		.collect();

	runas::Command::new("powershell.exe")
		.args(&["-EncodedCommand", &general_purpose::STANDARD.encode(utf_16_bytes)])
		.gui(true)
		.status()?;
	Ok(())
}
