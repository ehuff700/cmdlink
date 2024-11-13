use std::path::PathBuf;

/// Setup the project directory
fn setup_project_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
	let base_path = dirs::home_dir().expect("home directory not found!").join(".cmdlink");

	// Create the project directory if it doesn't exist
	std::fs::create_dir_all(&base_path).map_err(|e| format!("error creating project directory: {e}"))?;

	// Create the bins directory within the project directory
	let bins_dir = base_path.join("bins");
	std::fs::create_dir_all(&bins_dir).map_err(|e| format!("error creating bins directory: {e}"))?;

	Ok(base_path)
}

fn main() {
	// Setup the project directory
	if let Err(err) = setup_project_dir() {
		panic!("error setting up project directory: {}", err);
	}
}
