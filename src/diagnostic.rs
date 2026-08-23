//! Wrappers around git crates to simplify some specific git operations

use std::path::{Path, PathBuf};

use eyre::{Context, ContextCompat};
use git2::Repository;
use indicatif::ProgressBar;
use label_logger::{OutputLabel, console::style, error, info, label_theme, log};
use pariter::IteratorExt;

use crate::config::{Check, Config};

pub fn print_diagnostics(
	repos: Vec<PathBuf>,
	config: Config,
	search_directory: &Path,
) -> eyre::Result<()> {
	let len = repos
		.len()
		.try_into()
		.wrap_err("could not fit repos in a usize")?;
	let diag_bar = ProgressBar::new(len).with_style(label_theme(OutputLabel::Info("Checking")));
	let diag_bar_parallel = diag_bar.clone();

	let diagnostics = repos
		.into_iter()
		.parallel_map(move |path| {
			diag_bar_parallel.inc(1);

			let cwd = std::env::current_dir().ok()?;

			let repo = match Repository::open(&path) {
				Ok(repo) => repo,
				Err(err) => {
					let rel_path = pathdiff::diff_paths(&path, &cwd).unwrap_or(path);
					error!(
						"could not open repository {}: {}",
						rel_path.display(),
						err.message()
					);
					return None;
				}
			};

			let diag = match Diagnostic::analyze(&repo, &config) {
				Ok(diag) => diag,
				Err(err) => {
					let rel_path = pathdiff::diff_paths(&path, &cwd).unwrap_or(path);
					error!(
						"could not diagnostic repository {}: {err}",
						rel_path.display()
					);
					return None;
				}
			};

			if !diag.useful() {
				return None;
			}

			Some((path, diag))
		})
		.flatten()
		.collect::<Vec<_>>();

	diag_bar.finish_and_clear();

	for (path, diag) in diagnostics {
		let project_name = path
			.file_name()
			.wrap_err("could not get project name")?
			.to_string_lossy();
		let directory = path
			.parent()
			.wrap_err("could not get project directory")?
			.to_string_lossy();
		// Make path relative to root search directory
		let directory = directory.replacen(search_directory.to_string_lossy().as_ref(), ".", 1);

		let formatted_path = format!(
			"{}{}{}",
			style(directory).dim(),
			style(std::path::MAIN_SEPARATOR).dim(),
			project_name,
		);

		let dirty_info = if diag.is_dirty {
			style(" is dirty").yellow()
		} else {
			style("")
		};

		info!(label: "Repo", "{formatted_path}{dirty_info}");

		let ahead_branches = diag
			.ahead_branches
			.iter()
			.map(|name| style(name).yellow().to_string())
			.collect::<Vec<_>>();
		if !ahead_branches.is_empty() {
			log!(
				label: OutputLabel::Custom(style("└")),
				"has ahead branches: {}",
				ahead_branches.join(", ")
			);
		}

		let branches_no_upstream = diag
			.no_upstream_branches
			.iter()
			.map(|name| style(name).yellow().to_string())
			.collect::<Vec<_>>();
		if !branches_no_upstream.is_empty() {
			log!(
				label: OutputLabel::Custom(style("└")),
				"has branches with no upstream: {}",
				branches_no_upstream.join(", ")
			);
		}
	}

	Ok(())
}

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
	let mut opts = git2::StatusOptions::new();
	opts.include_untracked(false);

	let statuses = repo
		.statuses(Some(&mut opts))
		.wrap_err("could not get status")?;

	Ok(!statuses.is_empty())
}

/// Finds branches ahead of remote branches
fn check_ahead_branches(
	repo: &Repository,
	config: &Config,
) -> eyre::Result<(Vec<String>, Vec<String>)> {
	let mut ahead_branches = vec![];
	let mut branches_no_upstream = vec![];

	let check_ahead = config.checks.contains(&Check::AheadBranches);
	let check_no_upstream = config.checks.contains(&Check::NoUpstreamBranches);

	if !check_ahead && !check_no_upstream {
		return Ok((ahead_branches, branches_no_upstream));
	}

	let branches = repo
		.branches(Some(git2::BranchType::Local))
		.wrap_err("could not get local branches")?;

	for branch in branches {
		let (branch, _) = branch.wrap_err("could not iterate local branches")?;

		let name = match branch.name() {
			Ok(Some(name)) => name.to_string(),
			_ => continue,
		};

		let Ok(upstream_branch) = branch.upstream() else {
			if check_no_upstream {
				branches_no_upstream.push(name);
			}
			continue;
		};

		if check_ahead {
			let Ok(local_commit) = branch.get().peel_to_commit() else {
				continue;
			};
			let Ok(upstream_commit) = upstream_branch.get().peel_to_commit() else {
				continue;
			};

			if local_commit.id() != upstream_commit.id() {
				if let Ok((ahead, _)) =
					repo.graph_ahead_behind(local_commit.id(), upstream_commit.id())
				{
					if ahead > 0 {
						ahead_branches.push(name);
					}
				}
			}
		}
	}

	Ok((ahead_branches, branches_no_upstream))
}
