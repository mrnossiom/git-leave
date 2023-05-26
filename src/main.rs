#![warn(
	clippy::missing_docs_in_private_items,
	clippy::unwrap_used,
	clippy::expect_used,
	clippy::nursery,
	clippy::pedantic,
	clippy::cargo
)]
#![doc(
	html_logo_url = "https://raw.githubusercontent.com/MrNossiom/git-leave/main/assets/logo.png"
)]
#![doc = include_str!("../README.md")]

mod config;
mod crawl;
mod diagnostic;

use crate::{
	config::{Arguments, Config},
	crawl::crawl_directory_for_repos,
	diagnostic::Diagnostic,
};
use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use dirs::home_dir;
use label_logger::{console::style, error, info, log, success, warn, OutputLabel};
use std::{path::Path, time::Instant};

fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;

	// Parse command line arguments and get related config
	let args = Arguments::parse();
	let config = Config::from_git_config();
	let home_dir_path = home_dir().wrap_err("Could not get your home directory")?;
	let home_dir = home_dir_path
		.to_str()
		.wrap_err("Your home directory is not valid UTF-8")?;

	// Display the name of the program and welcome the user
	success!(label: "Welcome", "to {}", style("git leave").yellow().bold());

	// Set the path to the one specified in the global config
	// only if the default argument is enabled,
	// else set to the path specified in the arguments.
	let mut path = match (config, args.default) {
		(Some(git_config), true) => {
			if let Some(directory) = git_config.default_folder {
				directory
			} else {
				warn!("No default folder set in config, fallback to the one specified in the arguments");

				args.directory.clone()
			}
		}
		_ => args.directory.clone(),
	};

	path = path.replacen('~', home_dir, 1);

	// Get absolute path to the directory to crawl
	let search_directory = Path::new(&path)
		.canonicalize()
		.wrap_err("Could not get absolute path of specified directory")?;

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let repos = crawl_directory_for_repos(&search_directory, &args);

	// Exit if no git repositories were found
	if repos.is_empty() {
		error!(label: "Found", "no git repositories");

		return Ok(());
	}

	success!(
		label: "Found",
		"{} repositories in {}s",
		&repos.len(),
		begin_search_time.elapsed().as_millis() / 1000
	);

	let diagnostics = repos
		.iter()
		.map(Diagnostic::from_repo)
		.filter_map(Result::ok)
		.filter(Diagnostic::useful);

	for diagnostic in diagnostics {
		info!(
			label: "Repo",
			"{} {}",
			diagnostic.repository.path().parent()
				.wrap_err("Repository path points to a `.git` subdirectory, it always has a parent")?
				.to_str().wrap_err("Parent directory is not valid UTF-8")?
				.replace(home_dir, "~"),
			if diagnostic.is_dirty { style("is dirty").yellow() } else { style("") }
		);

		let ahead_branches = diagnostic
			.ahead_branches
			.iter()
			.map(|branch| match branch.name() {
				Ok(Some(name)) => name,
				Ok(None) => "<no name>",
				Err(_) => "<no UTF-8 name>",
			})
			.map(|name| style(name).yellow().to_string())
			.collect::<Vec<_>>();
		if !ahead_branches.is_empty() {
			log!(
				label: OutputLabel::Custom(style("└")),
				"has ahead branches: {}",
				ahead_branches.join(", ")
			);
		}

		let branches_no_upstream = diagnostic
			.no_upstream_branches
			.iter()
			.map(|branch| match branch.name() {
				Ok(Some(name)) => name,
				Ok(None) => "<no name>",
				Err(_) => "<no UTF-8 name>",
			})
			.map(|name| style(name).yellow().to_string())
			.collect::<Vec<_>>();
		if !branches_no_upstream.is_empty() {
			log!(
				label: OutputLabel::Custom(style("└")),
				"has branches with no upstream: {}",
				branches_no_upstream.join(", ")
			);
		}
	}

	Ok(())
}
