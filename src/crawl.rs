//! All the logic to crawl through directories in order to find git repos

use crossbeam::{queue::SegQueue, thread};
use git2::Repository;
use indicatif::ProgressBar;
use label_logger::{error, indicatif::label_theme, OutputLabel};

use std::{
	cmp::max,
	fs::read_dir,
	io,
	path::{Path, PathBuf},
};

/// Spawn the threads needed for crawling directories
pub fn crawl_directory_for_repos(directory: &Path) -> io::Result<Vec<Repository>> {
	// Contains paths to explore
	let paths = SegQueue::new();
	paths.push(directory.to_path_buf());

	// Contains found repositories
	let repositories = SegQueue::new();

	// Set the number of threads to use for crawling
	let thread_count = max(8, num_cpus::get() * 2);

	let dirty_bar = ProgressBar::new(1_u64).with_style(label_theme(OutputLabel::Info("Crawling")));

	thread::scope(|scope| {
		for _ in 0..thread_count {
			scope.spawn(|_| {
				while let Some(path) = paths.pop() {
					if let Err(error) = crawl(&path, &paths, &repositories, &dirty_bar) {
						error!("could not crawl {}: {}", path.display(), error);
					};
				}
			});
		}
	})
	.unwrap_or_else(|_| error!("Could not spawn threads"));

	dirty_bar.finish_and_clear();

	// Return the repositories in a `Vec`
	Ok(repositories.into_iter().collect::<Vec<_>>())
}

/// The actual crawling function
/// Search for git repositories and report folder to the given queue
fn crawl(
	directory: &PathBuf,
	path_queue: &SegQueue<PathBuf>,
	repositories: &SegQueue<Repository>,
	dirty_bar: &ProgressBar,
) -> io::Result<()> {
	if directory.is_dir() {
		// TODO: not sure we let this here
		dirty_bar.set_message(directory.display().to_string());
		dirty_bar.inc(1);

		// Return is the directory is a repo
		if let Ok(repo) = Repository::open(directory) {
			// Skip bare repos
			if !repo.is_bare() {
				repositories.push(repo);

				return Ok(());
			}
		}

		// Get the directory contents
		let dir_content = match read_dir(directory) {
			Ok(dir_content) => dir_content.collect::<Vec<_>>(),
			Err(err) => {
				error!("in {}: {}", directory.display(), err);
				return Ok(());
			}
		};

		// TODO: skip if the directory has a large number of subdirectories, add a flag to control this behaviour

		// Loop through the directory contents and add new directories to the queue
		for entry in dir_content {
			let path = match entry {
				Ok(entry) => entry.path(),
				Err(err) => {
					error!("in {}: {}", directory.display(), err);
					return Ok(());
				}
			};

			if path.is_symlink() {
				continue;
			}

			if path.is_dir() {
				path_queue.push(path);
				dirty_bar.inc_length(1);
			}
		}
	}

	Ok(())
}
