//! All the logic to crawl through directories in order to find git repos

use crossbeam::{queue::SegQueue, thread};
use git2::Repository;
use label_logger::OutputLabel;
use std::{
	cmp::max,
	fs::read_dir,
	io::Result as IoResult,
	path::{Path, PathBuf},
};

/// Spawn the threads needed for crawling directories
pub fn crawl_directory_for_repos(directory: &Path) -> IoResult<Vec<Repository>> {
	// Contains paths to explore
	let paths = SegQueue::new();
	paths.push(directory.to_path_buf());

	// Contains found repositories
	let repositories = SegQueue::new();

	// Set the number of threads to use for crawling
	let thread_count = max(8, num_cpus::get() * 2);

	thread::scope(|scope| {
		for _ in 0..thread_count {
			scope.spawn(|_| {
				while let Some(path) = paths.pop() {
					crawl(path, &paths, &repositories).unwrap();
				}
			});
		}
	})
	.unwrap_or_else(|_| error!("Could not spawn threads"));

	// Return the repositories in a `Vec`
	Ok(repositories.into_iter().collect::<Vec<Repository>>())
}

/// The actual crawling function
/// Search for git repositories and report folder to the given queue
fn crawl(
	directory: PathBuf,
	path_queue: &SegQueue<PathBuf>,
	repositories: &SegQueue<Repository>,
) -> IoResult<()> {
	if directory.is_dir() {
		print!(
			"{}\r",
			format_label!(
				label: OutputLabel::Info("Directory"),
				"{}",
				directory.display().to_string(),)
		);

		// Return is the directory is a repo
		if let Ok(repo) = Repository::open(&directory) {
			// Skip bare repos
			if !repo.is_bare() {
				repositories.push(repo);

				return Ok(());
			}
		}

		// Get the directory contents
		let dir_content = match read_dir(&directory) {
			Ok(dir_content) => dir_content.collect::<Vec<_>>(),
			Err(err) => {
				error!("in {}: {}", directory.display(), err);
				return Ok(());
			}
		};

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
			}
		}
	}

	Ok(())
}
