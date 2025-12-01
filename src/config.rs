//! Parse CLI arguments and local user config

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use label_logger::warn;

/// Check for unsaved or uncommitted changes on your machine
#[derive(Parser)]
#[clap(name = "git-leave", about, version, author, long_about = None)]
pub struct Args {
	/// Directory to search in
	#[clap(default_value = ".")]
	pub directory: PathBuf,

	/// Use default folder specified in git config for the directory to search in
	#[clap(long, short)]
	pub default: bool,

	/// Follow symlinks
	#[clap(long)]
	pub follow_symlinks: bool,

	/// Show the directories we are actually crawling
	#[clap(long)]
	pub show_directories: bool,

	/// Number of cores to use for crawling
	#[clap(long, default_value_t = num_cpus::get())]
	pub threads: usize,

	// Singular is used because of repetition on the CLI
	// e.g. `--check dirty --check ahead-branches`
	#[clap(long)]
	/// Override checks to run on found repositories
	pub check: Vec<Check>,
}

#[derive(Debug, Clone, PartialEq, Eq, ValueEnum)]
pub enum Check {
	/// Whether the repository has a dirty working copy.
	Dirty,
	/// List all branches that are ahead of their remote.
	AheadBranches,
	/// List all branches with no upstream.
	NoUpstreamBranches,
}

// Keys used in `.gitconfig` file
/// Default folder to start crawling from
const CONFIG_KEY_DEFAULT_FOLDER: &str = "leaveTool.defaultFolder";
/// Override the checks run on repositories
const CONFIG_KEY_CHECKS: &str = "leaveTool.checks";

/// Contains all the parsed configuration keys for this tool
pub struct Config {
	/// Default folder to search in when using the `--default` argument
	pub default_folder: Option<PathBuf>,

	/// Checks to run in a repository diagnostic
	pub checks: Vec<Check>,
}

impl Default for Config {
	fn default() -> Self {
		Self {
			default_folder: None,

			checks: vec![
				Check::Dirty,
				Check::AheadBranches,
				Check::NoUpstreamBranches,
			],
		}
	}
}

impl Config {
	/// Parse the global git config file and return the keys we are interested in.
	pub fn apply_git_config(&mut self) -> eyre::Result<()> {
		let config = match gix_config::File::from_globals() {
			Ok(config) => config,
			Err(err) => {
				warn!("could not open global config: {err}");
				return Ok(());
			}
		};

		if let Some(default_folder) = config.string(CONFIG_KEY_DEFAULT_FOLDER) {
			self.default_folder = Some(default_folder.to_string().into());
		}

		if let Some(checks) = config.string(CONFIG_KEY_CHECKS) {
			let checks = checks
				.to_string()
				.split_ascii_whitespace()
				.map(|check| Check::from_str(check, false))
				.collect::<Result<Vec<_>, _>>();

			match checks {
				Ok(checks) => self.checks = checks,
				Err(err) => return Err(eyre::eyre!("could not parse checks: {err}")),
			}
		}

		Ok(())
	}

	/// Mutate the config using CLI provided arguments
	pub fn apply_args(&mut self, args: &Args) {
		if !args.check.is_empty() {
			args.check.clone_into(&mut self.checks);
		}
	}
}
