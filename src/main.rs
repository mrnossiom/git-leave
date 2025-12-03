#![doc(
	html_logo_url = "https://raw.githubusercontent.com/mrnossiom/git-leave/main/assets/logo.png"
)]
#![doc = include_str!("../README.md")]

use std::{borrow::Cow, process, time::Instant};

use clap::Parser;
use eyre::Context;
use label_logger::{error, success};

use crate::{
	config::{Args, Config},
	crawl::crawl_repositories,
	diagnostic::print_diagnostics,
};

mod config;
mod crawl;
mod diagnostic;

fn main() -> eyre::Result<()> {
	let args = Args::parse();

	let mut config = Config::default();
	config.apply_git_config()?;
	config.apply_args(&args);

	// Set the path to the one specified in the global config
	// only if the default argument is enabled,
	// else set to the path specified in the arguments.
	let path = match (args.default, &config.default_folder) {
		(true, Some(directory)) => Cow::Borrowed(directory),
		(true, None) => {
			error!("No default folder set in config");
			process::exit(1);
		}
		_ => Cow::Borrowed(&args.directory),
	};

	// Get absolute path to the directory to crawl
	let search_directory = path
		.canonicalize()
		.wrap_err("Could not get absolute path of specified directory")?;

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let mut repos = crawl_repositories(&search_directory, &args);

	repos.sort();

	// Exit if no git repositories were found
	if repos.is_empty() {
		error!(label: "Found", "no git repositories");

		return Ok(());
	}

	success!(
		label: "Found",
		"{} repositories in {}s",
		&repos.len(),
		begin_search_time.elapsed().as_secs()
	);

	print_diagnostics(repos, config, &search_directory)?;

	Ok(())
}
