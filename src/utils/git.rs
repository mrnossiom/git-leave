use crate::log::{println_label, OutputLabel};
use git2::{Branch, BranchType, Repository, Status};

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

pub fn find_ahead_branches_in_repo(repo: &Repository) -> Vec<Branch> {
	// Iterate over all local branches
	// For each, check is a branch is ahead of its remote counterpart

	// Get all local branches
	let local_branches = repo
		.branches(Some(BranchType::Local))
		.expect("Could not get local branches")
		.map(|b| b.unwrap().0)
		.collect::<Vec<Branch>>();

	let mut ahead_branches: Vec<Branch> = Vec::new();

	// Iterate over all local branches
	for branch in local_branches {
		if let Ok(remote_branch) = branch.upstream() {
			let (last_local_commit, last_remote_commit) = (
				branch
					.get()
					.peel_to_commit()
					.expect("could not get last commit on local branch"),
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
			println_label(
				OutputLabel::Info("Info"),
				format!(
					"No upstream branch for {} in {}",
					branch.name().unwrap().unwrap(),
					repo.path().parent().unwrap().to_str().unwrap()
				),
			);
		}
	}

	ahead_branches
}
