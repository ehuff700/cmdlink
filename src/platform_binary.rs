use std::{
	borrow::Cow,
	fs::File,
	io::{ErrorKind, Write},
	path::Path,
};

use crate::{error::Error, Result, PROJECT_DIR};

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub enum Action {
	Create,
	Update,
	Remove,
	None,
}

#[derive(Debug)]
/// A struct representing a platform-specific binary/link. These are created and
/// managed by the `Config` struct to create aliases for commands.
pub struct PlatformBinary<'a> {
	/// Whether or not the platform binary file exists at it's expected path.
	exists: bool,
	/// The action to be taken for the platform binary file, see [Action]
	action: Action,
	/// The alias for the platform binary.
	alias: Cow<'a, str>,
	/// The command to run in place of the alias.
	cmd: Cow<'a, str>,
}

impl PlatformBinary<'_> {
	pub fn new(alias: impl Into<Cow<'static, str>>, cmd: impl Into<Cow<'static, str>>, action: Action) -> Self {
		let mut p = PlatformBinary {
			alias: alias.into(),
			cmd: cmd.into(),
			exists: false,
			action,
		};
		p.validate();
		p
	}

	/// Validates the existence of the platform binary file.
	#[inline]
	fn validate(&mut self) { self.exists = self.file_path().exists(); }

	/// Determines whether or not the platform binary file exists.
	#[inline]
	pub fn exists(&self) -> bool { self.exists }

	/// Determins the action to take for the binary.
	#[inline]
	pub fn action(&self) -> Action { self.action }

	/// Performs the appropriate action based on the platform binary's action.
	pub fn perform_action(&self) -> Result<()> {
		match self.action {
			Action::Create => self.create_link(),
			Action::Update => self.update_link(),
			Action::Remove => self.remove_link(),
			Action::None => Ok(()),
		}
	}

	/// Sets the action for the platform binary.
	pub fn set_action(&mut self, action: Action) { self.action = action; }

	/// Creates a link, returning an error if the link already exists.
	fn create_link(&self) -> Result<()> {
		let file_path = self.file_path();
		let mut file = File::create_new(file_path).map_err(|e| {
			if e.kind() == ErrorKind::AlreadyExists {
				Error::LinkAlreadyExists(self.alias().to_string())
			} else {
				Error::LinkCreation(self.alias().to_string(), e)
			}
		})?;
		file.write_all(self.contents().as_bytes())
			.map_err(|e| Error::LinkCreation(self.alias().to_string(), e))?;
		Ok(())
	}

	/// Updates the link with the new contents
	fn update_link(&self) -> Result<()> {
		std::fs::write(self.file_path(), self.contents()).map_err(|e| Error::LinkUpdate(self.alias().to_string(), e))
	}

	/// Removes the link, returning an error if the link does not exist.
	fn remove_link(&self) -> Result<()> {
		std::fs::remove_file(self.file_path()).map_err(|e| Error::LinkUpdate(self.alias().to_string(), e))
	}
}

impl Link for PlatformBinary<'_> {
	fn alias(&self) -> &str { self.alias.as_str() }

	fn cmd(&self) -> &str { self.cmd.as_str() }
}

/// Helper trait to abstract platform-specific link functionality.
pub trait Link {
	/// Getter for the alias.
	fn alias(&self) -> &str;
	/// Getter for the command.
	fn cmd(&self) -> &str;
	/// The extension of the link file.
	#[inline]
	fn extension(&self) -> &str {
		if cfg!(target_os = "windows") {
			".bat"
		} else {
			".sh"
		}
	}
	/// The file path of the link file.
	#[inline]
	fn file_path(&self) -> &'static Path {
		Box::leak(
			PROJECT_DIR
				.join("bins")
				.join(format!("{}{}", self.alias(), self.extension()))
				.into_boxed_path(),
		)
	}

	/// The contents of the link file
	#[inline]
	fn contents(&self) -> String {
		#[cfg(target_os = "windows")]
		{
			format!("@echo off\necho.\n{} %*", self.cmd())
		}
		#[cfg(any(target_os = "linux", target_os = "macos"))]
		{
			format!("#!/bin/sh\nexec {} \"$@\"", self.cmd())
		}
	}
}
