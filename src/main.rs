use clap::Parser;

mod git;
mod lib;
mod log;

use std::path::Path;
use yansi::{Color, Paint};

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
	let directory = Path::new(&args.directory).canonicalize().unwrap();

	// Find git repositories in the specified directory
	let repos = lib::find_repos_in_dir(&directory).unwrap();

	log::println_label(
		"Found",
		Color::Blue,
		format!("{} repositories", &repos.len()).as_str(),
	);

	// Check if there are dirty repositories
	let mut dirty_repos: Vec<String> = Vec::new();
	let mut not_pushed_commits_repos: Vec<String> = Vec::new();

	for repo in repos {
		if git::is_repo_dirty(&repo) {
			dirty_repos.push(git::repo_folder_name(&repo))
		}

		if git::has_repo_not_pushed_commits(&repo) {
			not_pushed_commits_repos.push(git::repo_folder_name(&repo))
		}
	}

	if !dirty_repos.is_empty() {
		log::println_label(
			"Found",
			Color::Blue,
			format!("{} dirty repositories", &dirty_repos.len()).as_str(),
		);
		for repo in dirty_repos {
			log::println(&repo.as_str());
		}
	}

	if !not_pushed_commits_repos.is_empty() {
		log::println_label(
			"Found",
			Color::Blue,
			format!(
				"{} repositories that have not pushed commits to remote",
				&not_pushed_commits_repos.len()
			)
			.as_str(),
		);
		for repo in not_pushed_commits_repos {
			log::println(&repo.as_str());
		}
	}

	// If user decided to push commits, then push them
	if args.push && lib::ask("Push commits to remote?") {
		log::println_label("Pushing", Color::Green, "commits to remote");
		// TODO: Prompt user to push commits
	}
}
