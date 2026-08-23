//! All the logic to crawl through directories in order to find git repos

use std::{
	fs::read_dir,
	io,
	path::{Path, PathBuf},
	thread,
};

use crossbeam::queue::SegQueue;
use eyre::Context;
use indicatif::ProgressBar;
use label_logger::{OutputLabel, format_label, indicatif::label_theme};

use crate::config::Args;

/// Spawn the threads needed for crawling directories
pub fn crawl_repositories(directory: &Path, settings: &Args) -> eyre::Result<Vec<PathBuf>> {
	let pending_paths = SegQueue::new();
	pending_paths.push(directory.to_path_buf());

	// Contains found repositories
	let repositories = SegQueue::new();

	let dirty_bar = ProgressBar::new(1).with_style(label_theme(OutputLabel::Info("Crawling")));

	let cwd = std::env::current_dir().wrap_err("could not get current directory")?;

	thread::scope(|scope| {
		for _ in 0..settings.threads {
			scope.spawn(|| {
				while let Some(path) = pending_paths.pop() {
					if let Err(error) = crawl(&path, &pending_paths, &repositories, &dirty_bar, settings) {
						let rel_path = pathdiff::diff_paths(&path, &cwd).unwrap_or(path);
						let msg = format_label!(label: OutputLabel::Error("Error"), "could not crawl {}: {error}", rel_path.display());
						dirty_bar.println(msg);
					}
				}
			});
		}
	});

	dirty_bar.finish_and_clear();

	Ok(repositories.into_iter().collect::<Vec<_>>())
}

/// Search for git repositories and report folder to the given queue
fn crawl(
	directory: &PathBuf,
	pending_paths: &SegQueue<PathBuf>,
	repositories: &SegQueue<PathBuf>,
	dirty_bar: &ProgressBar,
	settings: &Args,
) -> io::Result<()> {
	if directory.is_dir() {
		if settings.show_directories {
			dirty_bar.set_message(directory.display().to_string());
		}

		dirty_bar.inc(1);

		// Return if the directory is a repo
		if git2::Repository::open_ext(
			directory,
			git2::RepositoryOpenFlags::NO_SEARCH,
			std::iter::empty::<&std::ffi::OsStr>(),
		)
		.is_ok()
		{
			repositories.push(directory.clone());
			return Ok(());
		}

		// Loop through the directory contents and add new directories to the queue
		for entry in read_dir(directory)? {
			let path = entry.map(|entry| entry.path())?;

			if path.is_symlink() && !settings.follow_symlinks {
				continue;
			}

			if path.is_dir() {
				pending_paths.push(path);
				dirty_bar.inc_length(1);
			}
		}
	}

	Ok(())
}
