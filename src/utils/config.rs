use crate::log::{println_label, OutputLabel};
use clap::Parser;
use git2::Config as GitConfig;
use std::sync::atomic::AtomicBool;

#[derive(Parser)]
#[clap(name = "git-leave", about, version, author, long_about = None)]
pub struct Arguments {
	/// The directory to search in
	#[clap(default_value_t = String::from("."))]
	pub directory: String,

	/// Use git config default folder value for the directory to search in
	#[clap(short, long)]
	pub default: bool,

	/// Don't trim output path (useful for debugging)
	#[clap(long)]
	pub no_trim: bool,
}

/// The atomic bool to store the value of the `--no-trim` flag
pub static TRIM_OUTPUT: AtomicBool = AtomicBool::new(true);

// Keys used in `.gitconfig` file
const CONFIG_KEY_DEFAULT_FOLDER: &str = "leaveTool.defaultFolder";

/// Contains all the parsed configuration keys for this tool
pub struct GitLeaveConfig {
	pub default_folder: Option<String>,
}

/// Return all config entries related to this tool
pub fn get_related_config() -> Option<GitLeaveConfig> {
	let config_path = match GitConfig::find_global() {
		Ok(path) => path,
		_ => return None,
	};

	let config = match GitConfig::open(&config_path) {
		Ok(config) => config,
		Err(err) => {
			println_label(
				OutputLabel::Error,
				format!("Could not open global config: {}", err),
			);

			return None;
		}
	};

	Some(GitLeaveConfig {
		default_folder: get_key_string_value(&config, CONFIG_KEY_DEFAULT_FOLDER),
	})
}

// Correctly parse string value for a given key
fn get_key_string_value(config: &GitConfig, key: &str) -> Option<String> {
	let string_value = match config.get_string(key) {
		Ok(value) => value,
		Err(_) => return None,
	};

	match string_value.as_str() {
		"" => None,
		string => Some(string.to_string()),
	}
}
