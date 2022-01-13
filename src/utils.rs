use crate::log::{print_label, OutputLabel};
use git2::Repository;
use std::{fs::read_dir, io::Result as IoResult, path::Path};

pub fn find_repos_in_dir(dir: &Path) -> IoResult<Vec<Repository>> {
	let mut repos: Vec<Repository> = Vec::new();

	// TODO: make this function more efficient (using threads)

	if dir.is_dir() {
		print_label(OutputLabel::Info("Directory"), dir.display().to_string());

		let dir_content = read_dir(dir)
			.expect("Couldn't read directory")
			.collect::<Vec<_>>();

		for entry in dir_content {
			let path = entry.expect("Couldn't read file or directory").path();

			if path.is_dir() {
				if let Ok(repo) = Repository::open(&path) {
					if repo.is_bare() {
						continue;
					}

					repos.push(repo);
				} else {
					repos.extend(find_repos_in_dir(&path)?);
				}
			}
		}
	}

	Ok(repos)
}
