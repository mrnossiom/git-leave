use crate::log::{print_label, println_label, OutputLabel};
use crossbeam::{queue::SegQueue, thread};
use git2::Repository;
use std::{
	cmp::max,
	fs::read_dir,
	io::Result as IoResult,
	path::{Path, PathBuf},
};

pub fn crawl_directory_for_repos(dir: &Path) -> IoResult<Vec<Repository>> {
	// Contains paths to explore
	let paths = SegQueue::new();
	paths.push(dir.to_path_buf());

	// Contains repos
	let repos = SegQueue::new();

	let thread_count = max(8, num_cpus::get() * 2);

	thread::scope(|scope| {
		for _ in 0..thread_count {
			scope.spawn(|_| {
				while let Some(path) = paths.pop() {
					crawl(path, &paths, &repos).unwrap();
				}
			});
		}
	})
	.unwrap_or_else(|_| eprintln!("Could not spawn threads"));

	let mut repositories = Vec::new();
	while let Some(repo) = repos.pop() {
		repositories.push(repo);
	}

	Ok(repositories)
}

fn crawl(
	dir: PathBuf,
	path_queue: &SegQueue<PathBuf>,
	repos: &SegQueue<Repository>,
) -> IoResult<()> {
	if dir.is_dir() {
		print_label(OutputLabel::Info("Directory"), dir.display().to_string());

		let dir_content = match read_dir(&dir) {
			Ok(dir_content) => dir_content.collect::<Vec<_>>(),
			Err(err) => {
				println_label(OutputLabel::Error, format!("in {}: {}", dir.display(), err));
				return Ok(());
			}
		};

		for entry in dir_content {
			let path = match entry {
				Ok(entry) => entry.path(),
				Err(err) => {
					println_label(OutputLabel::Error, format!("in {}: {}", dir.display(), err));
					return Ok(());
				}
			};

			if path.is_symlink() {
				continue;
			}

			if path.is_dir() {
				if let Ok(repo) = Repository::open(&path) {
					if repo.is_bare() {
						continue;
					}

					repos.push(repo);
				} else {
					path_queue.push(path);
				}
			}
		}
	}

	Ok(())
}
