mod git;
mod log;
mod utils;

use crate::{
	git::{find_ahead_branches_in_repo, is_repo_dirty},
	log::{println, println_label},
	utils::{ask, find_repos_in_dir, AskDefault},
};
use clap::Parser;
use git2::{Branch, Repository};
use log::OutputLabel;
use std::{path::Path, time::Instant};
use yansi::Paint;

/// Push all commits in git repositories
#[derive(Parser, Debug)]
#[clap(name = "git-leave", about, version, author)]
struct Arguments {
	/// The directory to search in
	#[clap(default_value_t = String::from("."))]
	directory: String,

	/// Push commits to remote
	#[clap(short, long)]
	push: bool,
}

// TODO: default project directory in .gitconfig via gitleave.projects_dir or something like this

fn main() {
	// Enable coloring on Windows if possible
	if cfg!(windows) && !Paint::enable_windows_ascii() {
		Paint::disable();
	}

	// Parse command line arguments
	let args = Arguments::parse();

	// Display the name of the program
	println_label(
		OutputLabel::Success("Welcome"),
		format!("to {}", Paint::yellow("git-leave")),
	);

	// Get absolute path
	let search_directory = Path::new(&args.directory)
		.canonicalize()
		.expect("Could not get absolute path");

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let repos = find_repos_in_dir(&search_directory).expect("Could not read folder content");

	// Exit if no git repositories were found
	if repos.is_empty() {
		println_label(OutputLabel::Error, "No git repositories found");

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
					.replace(env!("HOME"), "~"),
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
							.map(|branch| branch.name().unwrap().unwrap())
							.collect::<Vec<&str>>()
							.join("/")
					)
				));
			});
	}

	return;

	#[allow(unreachable_code)]
	// If there is ahead branches and that user decided to push commits, then push to remote.
	if !repos_with_ahead_branches.is_empty()
		&& (args.push || ask("Push commits to remote?", AskDefault::Yes))
	{
		println_label(OutputLabel::Success("Pushing"), "commits to remote");
		// TODO: implement
	}
}
