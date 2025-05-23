//! All the logic to crawl through directories in order to find git repos

use crate::config::Arguments;
use crossbeam::{queue::SegQueue, thread};
use git2::Repository;
use indicatif::ProgressBar;
use label_logger::{OutputLabel, format_label, indicatif::label_theme};
use std::{
	fs::read_dir,
	io,
	path::{Path, PathBuf},
};

/// Spawn the threads needed for crawling directories
#[allow(clippy::module_name_repetitions)]
pub fn crawl_directory_for_repos(
	directory: &Path,
	settings: &Arguments,
) -> io::Result<Vec<Repository>> {
	// Contains paths to explore
	let paths = SegQueue::new();
	paths.push(directory.to_path_buf());

	// Contains found repositories
	let repositories = SegQueue::new();

	let dirty_bar = ProgressBar::new(1).with_style(label_theme(OutputLabel::Info("Crawling")));

	thread::scope(|scope| {
		for _ in 0..settings.threads {
			scope.spawn(|_| {
				while let Some(path) = paths.pop() {
					if let Err(error) = crawl(&path, &paths, &repositories, &dirty_bar, settings) {
						let msg = format_label!(label: OutputLabel::Error("Error"), "could not crawl {}: {}", path.display(), error);
						dirty_bar.println(msg);
					}
				}
			});
		}
	})
	.map_err(|err| io::Error::new(
		io::ErrorKind::Other,
		format!("could not spawn threads because of error of type: {:?}", err.type_id())
	))?;

	dirty_bar.finish_and_clear();

	Ok(repositories.into_iter().collect::<Vec<_>>())
}

// TODO: make tests for this function
/// Search for git repositories and report folder to the given queue
fn crawl(
	directory: &PathBuf,
	path_queue: &SegQueue<PathBuf>,
	repositories: &SegQueue<Repository>,
	dirty_bar: &ProgressBar,
	settings: &Arguments,
) -> io::Result<()> {
	if directory.is_dir() {
		if settings.show_directories {
			dirty_bar.set_message(directory.display().to_string());
		}

		dirty_bar.inc(1);

		// Return is the directory is a repo
		if let Ok(repo) = Repository::open(directory) {
			// Skip bare repos
			if !repo.is_bare() {
				repositories.push(repo);

				return Ok(());
			}
		}

		// Loop through the directory contents and add new directories to the queue
		for entry in read_dir(directory)? {
			let path = entry.map(|entry| entry.path())?;

			if path.is_symlink() && !settings.follow_symlinks {
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
