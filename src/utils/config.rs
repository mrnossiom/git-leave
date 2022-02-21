use crate::log::{println_label, OutputLabel};
use git2::Config;

const CONFIG_KEY_DEFAULT_FOLDER: &str = "leaveTool.defaultFolder";

pub struct GitLeaveConfig {
	pub default_folder: Option<String>,
}

/// Return all config entries related to this tool
pub fn get_related_config() -> Option<GitLeaveConfig> {
	let config_path = match Config::find_global() {
		Ok(path) => path,
		_ => return None,
	};

	let config = match Config::open(&config_path) {
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

fn get_key_string_value(config: &Config, key: &str) -> Option<String> {
	let string_value = match config.get_string(key) {
		Ok(value) => value,
		Err(_) => return None,
	};

	match string_value.as_str() {
		"" => None,
		string => Some(string.to_string()),
	}
}
