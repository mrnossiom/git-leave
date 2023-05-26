//! All the logic to crawl through directories in order to find git repos

use crate::config::Arguments;
use crossbeam::queue::SegQueue;
use git2::Repository;
use ignore::{WalkBuilder, WalkState};
use label_logger::error;
use std::path::Path;

/// Spawn the threads needed for crawling directories
#[allow(clippy::module_name_repetitions)]
pub fn crawl_directory_for_repos(
	directory: &Path,
	settings: &Arguments,
) -> std::vec::Vec<git2::Repository> {
	// Contains found repositories
	let repositories = SegQueue::new();

	let walker = WalkBuilder::new(directory)
		.follow_links(settings.follow_symlinks)
		.build_parallel();

	walker.run(|| {
		let repositories = &repositories;
		Box::new(move |result| {
			match result {
				Ok(entry) => {
					if entry.file_type().map_or(false, |ft| ft.is_dir()) {
						// Return is the directory is a repo
						if let Ok(repo) = Repository::open(entry.path()) {
							// Skip bare repos
							if !repo.is_bare() {
								repositories.push(repo);

								return WalkState::Skip;
							}
						}
					}
				}
				Err(ignore::Error::Io(error)) => {
					error!("could not access path: {}", error);
				}
				Err(error) => {
					error!("something wrong happened: {}", error);
				}
			}

			WalkState::Continue
		})
	});

	repositories.into_iter().collect::<Vec<_>>()
}
