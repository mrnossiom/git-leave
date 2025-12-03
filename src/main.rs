#![doc(
	html_logo_url = "https://raw.githubusercontent.com/mrnossiom/git-leave/main/assets/logo.png"
)]
#![doc = include_str!("../README.md")]

use std::{borrow::Cow, process, time::Instant};

use clap::Parser;
use eyre::Context;
use gix::ThreadSafeRepository;
use indicatif::ProgressBar;
use label_logger::{OutputLabel, console::style, error, info, label_theme, log, success};
use pariter::IteratorExt;

use crate::{
	config::{Args, Config},
	crawl::crawl_repositories,
	diagnostic::Diagnostic,
};

mod config;
mod crawl;
mod diagnostic;

fn main() -> eyre::Result<()> {
	let args = Args::parse();

	let mut config = Config::default();
	config.apply_git_config()?;
	config.apply_args(&args);

	// Set the path to the one specified in the global config
	// only if the default argument is enabled,
	// else set to the path specified in the arguments.
	let path = match (args.default, &config.default_folder) {
		(true, Some(directory)) => Cow::Borrowed(directory),
		(true, None) => {
			error!("No default folder set in config");
			process::exit(1);
		}
		_ => Cow::Borrowed(&args.directory),
	};

	// Get absolute path to the directory to crawl
	let search_directory = path
		.canonicalize()
		.wrap_err("Could not get absolute path of specified directory")?;

	// Start the timer
	let begin_search_time = Instant::now();

	// Find git repositories in the specified directory
	let mut repos = crawl_repositories(&search_directory, &args);
	repos.sort();

	// Exit if no git repositories were found
	if repos.is_empty() {
		error!(label: "Found", "no git repositories");

		return Ok(());
	}

	success!(
		label: "Found",
		"{} repositories in {}s",
		&repos.len(),
		begin_search_time.elapsed().as_secs()
	);

	let diag_bar = ProgressBar::new(repos.len().try_into().unwrap())
		.with_style(label_theme(OutputLabel::Info("Checking")));
	let diag_bar_parallel = diag_bar.clone();

	let diagnostics = repos
		.into_iter()
		.parallel_map(move |path| {
			diag_bar_parallel.inc(1);

			let Ok(repo) = ThreadSafeRepository::open(path) else {
				error!("could not open repository");
				return None;
			};

			let Ok(diag) = Diagnostic::analyze(&repo.to_thread_local(), &config) else {
				error!("could not open diagnostic");
				return None;
			};

			if !diag.useful() {
				return None;
			}

			Some((repo, diag))
		})
		.flatten()
		.collect::<Vec<_>>();

	diag_bar.finish_and_clear();

	for (repo, diag) in diagnostics {
		let path = repo
			.path()
			.parent()
			.expect("repository .git folder always has a parent");

		let project_name = path.file_name().unwrap().to_string_lossy();
		let directory = path.parent().unwrap().to_string_lossy();
		// Make path relative to root search directory
		let directory = directory.replacen(search_directory.to_string_lossy().as_ref(), ".", 1);

		let path = format!(
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

		info!(label: "Repo", "{path}{dirty_info}");

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
