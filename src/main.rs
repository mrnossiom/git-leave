#![warn(clippy::missing_docs_in_private_items, clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

#[macro_use]
extern crate label_logger;

mod config;
mod crawl;
mod git;

use crate::{
	config::{get_related_config, Arguments},
	crawl::crawl_directory_for_repos,
	git::{find_ahead_branches_in_repo, is_repo_dirty},
};
use clap::Parser;
use color_eyre::eyre::{Context, ContextCompat};
use console::Term;
use dirs::home_dir;
use git2::{Branch, Repository};
use indicatif::ProgressBar;
use label_logger::{console::style, indicatif::label_theme, info, log, success, warn, OutputLabel};
use std::{path::Path, time::Instant};

fn main() -> color_eyre::Result<()> {
	color_eyre::install()?;

	// Parse command line arguments and get related config
	let args = Arguments::parse();
	let config = get_related_config();
	let home_dir_path = home_dir().wrap_err("Could not get your home directory")?;
	let home_dir = home_dir_path
		.to_str()
		.wrap_err("Your home directory is not valid UTF-8")?;

	// Display the name of the program and welcome the user
	success!(label: "Welcome", "to {}", style("git leave").yellow());

	// Set the path to the one specified in the global config
	// only if the default argument is enabled,
	// else set to the path specified in the arguments.
	let mut path = match config {
		Some(conf) => match (args.default, conf.default_folder) {
			(true, Some(dir)) => dir,
			(true, None) => {
				warn!("No default folder set in config, fallback to the one specified in the arguments");

				args.directory
			}
			(_, _) => args.directory,
		},
		_ => args.directory,
	};

	path = path.replacen('~', home_dir, 1);

	// Get absolute path to the directory to crawl
	let search_directory = Path::new(&path)
		.canonicalize()
		.wrap_err("Could not get absolute path of specified directory")?;

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let repos = crawl_directory_for_repos(&search_directory)
		.wrap_err("Something went wrong while trying to crawl the directory")?;

	Term::stdout().clear_line().ok();

	// Exit if no git repositories were found
	if repos.is_empty() {
		info!(label: "Empty", "No git repositories found");

		return Ok(());
	}

	info!(
		label: "Found",
		"{} repositories in {}s",
		&repos.len(),
		begin_search_time.elapsed().as_millis() / 1000
	);

	let dirty_bar =
		ProgressBar::new(repos.len() as u64).with_style(label_theme(OutputLabel::Info("Progress")));

	// Check if there are dirty repositories
	let dirty_repos: Vec<&Repository> = repos
		.iter()
		.filter(|repo| {
			let is_dirty = is_repo_dirty(repo);
			dirty_bar.inc(1);
			is_dirty
		})
		.collect();

	dirty_bar.finish();

	if !dirty_repos.is_empty() {
		info!(label: "Found", "{} dirty repositories", &dirty_repos.len());

		for repo in &dirty_repos {
			log!(
				"{}",
				repo.path()
					.parent()
					.expect(
						"Repository path points to a `.git` subdirectory, it always has a parent"
					)
					.to_str()
					.expect("Parent directory is not valid UTF-8")
					.replace(home_dir, "~"),
			);
		}
	}

	let ahead_bar =
		ProgressBar::new(repos.len() as u64).with_style(label_theme(OutputLabel::Info("Progress")));

	// Check if a repo has any local ahead branch
	let repos_with_ahead_branches: Vec<(&Repository, Vec<Branch>)> = repos
		.iter()
		.map(|repo| {
			let ret = (repo, find_ahead_branches_in_repo(repo));
			ahead_bar.inc(1);
			ret
		})
		.filter(|vec| !vec.1.is_empty())
		.collect();

	ahead_bar.finish();

	if !repos_with_ahead_branches.is_empty() {
		info!(
			label: "Found",
			"{} repositories that have not pushed commits to remote",
			&repos_with_ahead_branches.len()
		);

		for (repo, ahead_branches) in &repos_with_ahead_branches {
			log!(
					"Repository {} have these branches ahead: {}",
					style(
						repo.path()
							.parent()
							.expect("Repository path points to a `.git` subdirectory, it always has a parent")
							.file_name()
							.expect("parent has an absolute name")
							.to_string_lossy()
					)
					.yellow(),
					style(
						ahead_branches
							.iter()
							.map(|branch| branch
								.name()
								.expect("Found an ahead branch with non valid UTF-8")
								.unwrap_or("<no name found>"))
							.collect::<Vec<&str>>()
							.join("/")
					)
					.yellow()
				);
		}
	}

	Ok(())
}
