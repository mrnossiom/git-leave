//! All the logic to crawl through directories in order to find git repos

use crate::config::Arguments;
use crossbeam::{queue::SegQueue, thread};
use git2::Repository;
use indicatif::ProgressBar;
use label_logger::{indicatif::label_theme, pretty_output, OutputLabel};
use std::{
	fs::read_dir,
	io,
	path::{Path, PathBuf},
	sync::atomic::{AtomicU64, Ordering},
	thread::sleep,
	time::Duration,
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
	let bar = Bar::default();

	thread::scope(|scope| {
		for _ in 0..num_cpus::get() {
			scope.spawn(|_| {
				while let Some(path) = paths.pop() {
					bar.increment_crawled(1);
					if let Err(error) = crawl(&path, &paths, &repositories, &bar, settings) {
						bar.error(format!("could not crawl {}: {}", path.display(), error));
					};
				}
			});
		}

		scope.spawn(|_| loop {
			if bar.is_over() {
				bar.finish_and_clear();
				break;
			}

			bar.update();

			sleep(Duration::from_millis(500));
		});
	})
	.map_err(|err| {
		io::Error::new(
			io::ErrorKind::Other,
			format!(
				"could not spawn threads because of error of type: {:?}",
				err.type_id()
			),
		)
	})?;

	Ok(repositories.into_iter().collect::<Vec<_>>())
}

// TODO: make tests for this function
/// Search for git repositories and report folder to the given queue
fn crawl(
	directory: &PathBuf,
	paths: &SegQueue<PathBuf>,
	repositories: &SegQueue<Repository>,
	bar: &Bar,
	settings: &Arguments,
) -> io::Result<()> {
	if directory.is_dir() {
		// Return is the directory is a repo
		if let Ok(repo) = Repository::open(directory) {
			// Skip bare repos
			if !repo.is_bare() {
				repositories.push(repo);

				return Ok(());
			}
		}

		let mut nb = 0;
		// Loop through the directory contents and add new directories to the queue
		for entry in read_dir(directory)? {
			let path = entry.map(|entry| entry.path())?;

			if path.is_symlink() && !settings.follow_symlinks {
				continue;
			}

			if path.is_dir() {
				paths.push(path);
				nb += 1;
			}
		}

		bar.increment_length(nb);
	}

	Ok(())
}

/// Wraps an [`indicatif::ProgressBar`] to make it more efficient for multiple threads since the inner bar uses a mutex.
struct Bar {
	/// The inner [`indicatif::ProgressBar`]
	inner: ProgressBar,

	/// The number of directories crawled
	advancement: AtomicU64,
	/// The number of directories to crawl
	length: AtomicU64,
}

impl Default for Bar {
	fn default() -> Self {
		Self {
			inner: ProgressBar::new(1).with_style(label_theme(OutputLabel::Info("Crawling"))),
			advancement: AtomicU64::new(0),
			length: AtomicU64::new(0),
		}
	}
}

impl Bar {
	/// Increment the number of directories crawled
	fn increment_crawled(&self, nb: u64) {
		self.advancement.fetch_add(nb, Ordering::Relaxed);
	}

	/// Increment the number of directories to crawl
	fn increment_length(&self, nb: u64) {
		self.length.fetch_add(nb, Ordering::Relaxed);
	}

	/// Update the inner progress bar
	fn update(&self) {
		self.inner.update(|state| {
			state.set_len(self.length.load(Ordering::Relaxed));
			state.set_pos(self.advancement.load(Ordering::Relaxed));
		});
	}

	/// Is the progress bar over
	fn is_over(&self) -> bool {
		self.inner.position() > self.inner.length().unwrap_or(0)
	}

	/// Print an error message above the progress bar
	fn error(&self, message: String) {
		self.inner
			.println(pretty_output(OutputLabel::Error("Error"), message));
	}

	/// Finish the progress bar and clear it
	fn finish_and_clear(&self) {
		self.inner.finish_and_clear();
	}
}
