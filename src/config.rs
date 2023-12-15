//! Parse CLI arguments and local user config

use clap::Parser;
use git2::Config as GitConfig;
use label_logger::error;

/// Check for unsaved or uncommitted changes on your machine.
#[derive(Parser)]
#[clap(name = "git-leave", about, version, author, long_about = None)]
#[allow(clippy::struct_excessive_bools)]
pub struct Arguments {
	/// The directory to search in
	#[clap(default_value_t = String::from("."))]
	pub directory: String,

	/// Use git config default folder value for the directory to search in
	#[clap(long, short)]
	pub default: bool,

	/// Should we follow symlinks
	#[clap(long)]
	pub follow_symlinks: bool,

	/// Should we show the directories we are actually crawling
	#[clap(long)]
	pub show_directories: bool,

	/// The number of cores to use for crawling
	#[clap(long, default_value_t = num_cpus::get())]
	pub threads: usize,
}

// Keys used in `.gitconfig` file
/// The key used to store the default folder in `.gitconfig`
const CONFIG_KEY_DEFAULT_FOLDER: &str = "leaveTool.defaultFolder";

/// Contains all the parsed configuration keys for this tool
#[derive(Default)]
pub struct Config {
	/// The default folder to search in when using the `--default` argument
	pub default_folder: Option<String>,
}

impl Config {
	/// Parse the global git config file and return the keys we are interested in.
	pub fn from_git_config() -> Self {
		let Ok(config_path) = GitConfig::find_global() else {
			return Self::default();
		};

		let config = match GitConfig::open(&config_path) {
			Ok(config) => config,
			Err(err) => {
				error!("Could not open global config: {}", err);

				return Self::default();
			}
		};

		Self {
			default_folder: get_key_string_value(&config, CONFIG_KEY_DEFAULT_FOLDER),
		}
	}
}

/// Correctly parse string value for a given key
fn get_key_string_value(config: &GitConfig, key: &str) -> Option<String> {
	let Ok(string_value) = config.get_string(key) else {
		return None;
	};

	match &*string_value {
		"" => None,
		string => Some(string.to_string()),
	}
}
