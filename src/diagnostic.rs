//! Wrappers around git2 crate to simplify some specific git operations

use eyre::{Context, bail};
use gix::{Repository, features::progress, remote::Direction, status::UntrackedFiles};

use crate::config::{Check, Config};

/// A repository diagnostic
pub struct Diagnostic {
	/// Does the repository contains changes staged or not
	pub is_dirty: bool,
	/// Branches that are ahead of their remote counterpart
	pub ahead_branches: Vec<String>,
	/// Branches that have no remote counterpart
	pub no_upstream_branches: Vec<String>,
}

impl Diagnostic {
	/// Diagnostic a repo and make a report
	pub(crate) fn analyze(repo: &Repository, config: &Config) -> eyre::Result<Self> {
		let is_dirty = config.checks.contains(&Check::Dirty) && is_dirty(repo)?;

		let (ahead_branches, no_upstream_branches) = check_ahead_branches(repo, config)?;

		Ok(Self {
			is_dirty,
			ahead_branches,
			no_upstream_branches,
		})
	}

	/// Says if the report has something to say or if everything is ok
	pub(crate) fn useful(&self) -> bool {
		self.is_dirty || !self.ahead_branches.is_empty() || !self.no_upstream_branches.is_empty()
	}
}

/// Check if repository has unsaved files in working or dirty directory
fn is_dirty(repo: &Repository) -> eyre::Result<bool> {
	let mut statuses = repo
		.status(progress::Discard)
		.wrap_err("could not get status")?
		.untracked_files(UntrackedFiles::None)
		.into_iter(None)
		.wrap_err("could not iterate on statuses")?;

	// Return true if there are any changes
	Ok(statuses.any(|_| true))
}

/// Do not visit the commit graph further than this arbitrary limit
const MAX_ANCESTORS_VISIT: usize = 50;

/// Finds branches ahead of remote branches
fn check_ahead_branches(
	repo: &Repository,
	config: &Config,
) -> eyre::Result<(Vec<String>, Vec<String>)> {
	let references = repo.references().wrap_err("could not get references")?;
	let local_branches = references
		.local_branches()
		.wrap_err("could not get local branches")?;

	let mut ahead_branches = vec![];
	let mut branches_no_upstream = vec![];
	for mut local_ref in local_branches.filter_map(Result::ok) {
		let mut remote_ref = match local_ref.remote_ref_name(Direction::Push) {
			Some(Ok(remote_ref_name)) => repo
				.find_reference(remote_ref_name.as_partial_name())
				.wrap_err("could not get remote reference")?,
			None => {
				branches_no_upstream.push(local_ref.name().shorten().to_string());
				continue;
			}
			Some(Err(err)) => bail!("could not get branch remote: {err}"),
		};

		let last_local_commit = local_ref
			.peel_to_commit()
			.wrap_err("could not get last commit on local branch")?;
		let last_remote_commit = remote_ref
			.peel_to_commit()
			.wrap_err("could not get last commit on remote branch")?;

		if last_local_commit.id == last_remote_commit.id {
			continue;
		}

		let found = last_local_commit
			.ancestors()
			.first_parent_only()
			.all()
			.wrap_err("could not iterate on last commit ancestors")?
			.take(MAX_ANCESTORS_VISIT)
			.filter_map(Result::ok)
			.find(|info| info.id == last_local_commit.id);

		if found.is_some() {
			ahead_branches.push(local_ref.name().shorten().to_string());
		}
	}

	if !config.checks.contains(&Check::AheadBranches) {
		ahead_branches = Vec::new();
	}
	// TODO: avoid doing ancestors traversal earlier
	if !config.checks.contains(&Check::NoUpstreamBranches) {
		branches_no_upstream = Vec::new();
	}

	Ok((ahead_branches, branches_no_upstream))
}
