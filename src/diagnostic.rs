//! Wrappers around git2 crate to simplify some specific git operations

use git2::{Branch, BranchType, Repository, StatusOptions, Statuses};
use std::{fmt::Display, io};

/// A repository diagnostic
pub struct Diagnostic<'a> {
	/// The repository related to this diagnostic
	pub repository: &'a Repository,
	/// Does the repository contains changes staged or not
	pub is_dirty: bool,
	/// Branches that are ahead of their remote counterpart
	pub ahead_branches: Vec<Branch<'a>>,
	/// Branches that have no remote counterpart
	pub no_upstream_branches: Vec<Branch<'a>>,
}

impl<'a> Diagnostic<'a> {
	/// Diagnostic a repo and make a report
	pub(crate) fn from_repo(repository: &'a Repository) -> io::Result<Self> {
		let (ahead_branches, no_upstream_branches) = Self::ahead_branches(repository)?;

		Ok(Self {
			repository,
			is_dirty: Self::is_dirty(repository),
			ahead_branches,
			no_upstream_branches,
		})
	}

	/// Check if repository has unsaved files in working or dirty directory
	fn is_dirty(repo: &Repository) -> bool {
		let mut options = StatusOptions::new();
		options.include_ignored(false);
		options.include_untracked(true);

		repo.statuses(Some(&mut options))
			.iter()
			.flat_map(Statuses::iter)
			// Return true if there are any changes
			.any(|_| true)
	}

	/// Finds branches ahead of remote branches
	fn ahead_branches(repo: &Repository) -> io::Result<(Vec<Branch>, Vec<Branch>)> {
		let local_branches = match repo.branches(Some(BranchType::Local)) {
			Ok(branches) => branches.filter_map(Result::ok).map(|(branch, _)| branch),
			Err(err) => return Err(io::Error::new(io::ErrorKind::Other, err)),
		};

		let mut ahead_branches = vec![];
		let mut branches_no_upstream = vec![];
		for branch in local_branches {
			if let Ok(remote_branch) = branch.upstream() {
				let last_local_commit = branch.get().peel_to_commit().map_err(
					map_error_git_to_io("could not get last commit on local branch"),
				)?;
				let last_remote_commit =
					remote_branch
						.get()
						.peel_to_commit()
						.map_err(map_error_git_to_io(
							"could not get last commit on local branch",
						))?;

				if repo
					.graph_descendant_of(last_local_commit.id(), last_remote_commit.id())
					.map_err(map_error_git_to_io(
						"could not get graph difference between commits",
					))? {
					ahead_branches.push(branch);
				}
			} else {
				branches_no_upstream.push(branch);
			}
		}

		Ok((ahead_branches, branches_no_upstream))
	}

	/// Says if the report has something to say or if everything is ok
	pub(crate) fn useful(&self) -> bool {
		self.is_dirty || !self.ahead_branches.is_empty() || !self.no_upstream_branches.is_empty()
	}
}

/// Maps a [`git2::Error`] to an [`io::Error`] with a message
fn map_error_git_to_io<A: Display>(context: A) -> impl FnOnce(git2::Error) -> io::Error {
	move |err: git2::Error| {
		io::Error::new(
			io::ErrorKind::Other,
			format!("{context}: {}", err.message(),),
		)
	}
}
