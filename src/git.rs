use git2::{Repository, Status};

pub fn is_repo_dirty(repo: &Repository) -> bool {
	let mut dirty = false;

	if let Ok(statuses) = repo.statuses(None) {
		for status in statuses.iter() {
			match status.status() {
				Status::IGNORED => continue,
				_ => {
					dirty = true;
					break;
				}
			}
		}
	}

	dirty
}

pub fn has_repo_not_pushed_commits(_repo: &Repository) -> bool {
	// TODO: Implement
	// Get the remote tracking branch
	// Diff the last commits of each branch
	// Return true if there are any not pushed commits

	false
}

pub fn repo_folder_name(repo: &Repository) -> String {
	repo.path()
		.parent()
		.unwrap()
		.file_name()
		.unwrap()
		.to_str()
		.unwrap()
		.to_string()
}
