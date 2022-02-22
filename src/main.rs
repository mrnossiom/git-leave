mod log;
mod utils;

use clap::Parser;
use dirs::home_dir;
use git2::{Branch, Repository};
use log::{println, println_label, OutputLabel};
use std::{path::Path, sync::atomic::Ordering, time::Instant};
use utils::{
	config::{get_related_config, Arguments, TRIM_OUTPUT},
	crawl_directory_for_repos,
	git::{find_ahead_branches_in_repo, is_repo_dirty},
};
use yansi::Paint;

fn main() {
	// Enable coloring on Windows if possible
	#[cfg(windows)]
	if !Paint::enable_windows_ascii() {
		Paint::disable();
	}

	// Parse command line arguments and get related config
	let args = Arguments::parse();
	TRIM_OUTPUT.store(!args.no_trim, Ordering::Relaxed);
	let config = get_related_config();

	// Display the name of the program and welcome the user
	println_label(
		OutputLabel::Success("Welcome"),
		format!("to {}", Paint::yellow("git leave")),
	);

	// Set the path to the one specified in the global config
	// only if the default argument is enabled,
	// else set to the path specified in the arguments.
	let mut path = match config {
		Some(conf) => match (args.default, conf.default_folder) {
			(true, Some(dir)) => dir,
			(true, None) => {
				println_label(OutputLabel::Warning, "No default folder set in config, fallback to the one specified in the arguments");

				args.directory
			}
			(_, _) => args.directory,
		},
		_ => args.directory,
	};

	path = path.replace("~", home_dir().unwrap().to_str().unwrap());

	// Get absolute path to the directory to crawl
	let search_directory = match Path::new(&path).canonicalize() {
		Ok(path) => path,
		Err(err) => {
			println_label(
				OutputLabel::Error,
				format!(
					"Could not get absolute path of specified directory: {}",
					err
				),
			);

			return;
		}
	};

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let repos = match crawl_directory_for_repos(&search_directory) {
		Ok(repos) => repos,
		Err(err) => {
			println_label(
				OutputLabel::Error,
				format!(
					"Something went wrong while trying to crawl the directory: {}",
					err
				),
			);

			return;
		}
	};

	// Exit if no git repositories were found
	if repos.is_empty() {
		println_label(OutputLabel::Info("Empty"), "No git repositories found");

		return;
	}

	println_label(
		OutputLabel::Info("Found"),
		format!(
			"{} repositories in {}s",
			&repos.len(),
			begin_search_time.elapsed().as_millis() as f64 / 1000.0
		),
	);

	// Check if there are dirty repositories
	let dirty_repos: Vec<&Repository> = repos.iter().filter(|repo| is_repo_dirty(repo)).collect();

	if !dirty_repos.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!("{} dirty repositories", &dirty_repos.len()),
		);

		dirty_repos.iter().for_each(|repo| {
			println(
				repo.path()
					.parent()
					.unwrap()
					.to_str()
					.unwrap()
					.replace(home_dir().unwrap().as_path().to_str().unwrap(), "~"),
			);
		});
	}

	// Check if a repo has any local ahead branch
	let repos_with_ahead_branches: Vec<(&Repository, Vec<Branch>)> = repos
		.iter()
		.map(|repo| (repo, find_ahead_branches_in_repo(repo)))
		.filter(|vec| !vec.1.is_empty())
		.collect();

	if !repos_with_ahead_branches.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!(
				"{} repositories that have not pushed commits to remote",
				&repos_with_ahead_branches.len()
			),
		);

		repos_with_ahead_branches
			.iter()
			.for_each(|(repo, ahead_branches)| {
				println(format!(
					"Repository {} have these branches ahead: {}",
					Paint::yellow(
						repo.path()
							.parent()
							.unwrap()
							.file_name()
							.unwrap()
							.to_string_lossy()
					),
					Paint::yellow(
						ahead_branches
							.iter()
							.map(|branch| branch.name().unwrap().unwrap_or("<no name found>"))
							.collect::<Vec<&str>>()
							.join("/")
					)
				));
			});
	}
}
