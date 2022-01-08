mod git;
mod log;
mod utils;

use crate::{
	git::{has_repo_not_pushed_commits, is_repo_dirty},
	log::{println, println_label},
	utils::{ask, find_repos_in_dir, AskDefault},
};
use clap::Parser;
use git2::Repository;
use log::OutputLabel;
use std::path::Path;
use yansi::Paint;

// TODO: handle unwrap properly

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

fn main() {
	// Enable coloring on Windows if possible
	if cfg!(windows) && !Paint::enable_windows_ascii() {
		Paint::disable();
	}

	// Parse command line arguments
	let args = Arguments::parse();

	// Get absolute path
	let directory = Path::new(&args.directory)
		.canonicalize()
		.expect("Could not get absolute path");

	// Find git repositories in the specified directory
	let repos = find_repos_in_dir(&directory).expect("Could not read folder content");

	// Exit if no git repositories were found
	if repos.is_empty() {
		println_label(OutputLabel::Error, "No git repositories found");

		return;
	}

	println_label(
		OutputLabel::Info("Found"),
		format!("{} repositories", &repos.len()).as_str(),
	);

	// Check if there are dirty repositories
	let mut dirty_repos: Vec<&Repository> = Vec::new();
	let mut not_pushed_commits_repos: Vec<&Repository> = Vec::new();

	for repo in &repos {
		if is_repo_dirty(repo) {
			dirty_repos.push(repo)
		}

		if has_repo_not_pushed_commits(repo) {
			not_pushed_commits_repos.push(repo)
		}
	}

	if !dirty_repos.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!("{} dirty repositories", &dirty_repos.len()).as_str(),
		);
		for repo in dirty_repos {
			println(repo.path().parent().unwrap().to_str().unwrap());
		}
	}

	if !not_pushed_commits_repos.is_empty() {
		println_label(
			OutputLabel::Info("Found"),
			format!(
				"{} repositories that have not pushed commits to remote",
				&not_pushed_commits_repos.len()
			)
			.as_str(),
		);
		for repo in not_pushed_commits_repos {
			println(repo.path().parent().unwrap().to_str().unwrap());
		}
	}

	// If user decided to push commits, then push them
	if args.push || ask("Push commits to remote?", AskDefault::Yes) {
		println_label(OutputLabel::Success("Pushing"), "commits to remote");
	}
}
