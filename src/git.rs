//! Wrappers around git2 crate to simplify some specific git operations

use git2::{Branch, BranchType, Repository, Status};

/// Check if repository has unsaved files in working or dirty directory
pub fn is_repo_dirty(repo: &Repository) -> bool {
	if let Ok(statuses) = repo.statuses(None) {
		for status in statuses.iter() {
			match status.status() {
				Status::IGNORED => continue,
				_ => {
					return true;
				}
			}
		}
	}

	false
}

/// Finds branches ahead of remote branches
pub fn find_ahead_branches_in_repo(repo: &Repository) -> Vec<Branch> {
	// Iterate over all local branches
	// For each, check is a branch is ahead of its remote counterpart

	// Get all local branches
	let local_branches = match repo.branches(Some(BranchType::Local)) {
		Ok(branches) => branches
			.filter_map(|branch| branch.ok())
			.map(|(branch, _branch_type)| branch)
			.collect::<Vec<Branch>>(),
		Err(err) => {
			error!("in {}: {}", repo.path().display(), err.message());

			return vec![];
		}
	};

	let mut ahead_branches: Vec<Branch> = Vec::new();

	// Iterate over all local branches
	for branch in local_branches {
		if let Ok(remote_branch) = branch.upstream() {
			let (last_local_commit, last_remote_commit) = (
				match branch.get().peel_to_commit() {
					Ok(commit) => commit,
					Err(err) => {
						error!(
							"in {}: could not get last commit on local branch: {}",
							repo.path().display(),
							err.message()
						);

						return vec![];
					}
				},
				remote_branch
					.get()
					.peel_to_commit()
					.expect("could not get last commit on remote branch"),
			);

			if repo
				.graph_descendant_of(last_local_commit.id(), last_remote_commit.id())
				.expect("could not get graph difference between commits")
			{
				ahead_branches.push(branch)
			}
		} else {
			info!(
				"No upstream branch for {} in {}",
				branch
					.name()
					.expect("Found a branch with non valid UTF-8")
					.unwrap_or("<no name found>"),
				repo.path()
					.parent()
					.expect(
						"Repository path points to a `.git` subdirectory, it always has a parent"
					)
					.to_string_lossy()
			);
		}
	}

	ahead_branches
}
