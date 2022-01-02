use git2::Repository;
use std::{
	fs::read_dir,
	io::{self, stdin, stdout, Write},
	path::Path,
};

pub fn find_repos_in_dir(dir: &Path) -> io::Result<Vec<Repository>> {
	let mut repos: Vec<Repository> = Vec::new();

	// TODO: make this function more efficient (using threads)

	if dir.is_dir() {
		let dir_content = read_dir(dir).unwrap().collect::<Vec<_>>();

		for entry in dir_content {
			let path = entry.unwrap().path();

			if path.is_dir() {
				if let Ok(repo) = Repository::open(&path) {
					if repo.is_bare() {
						continue;
					}

					repos.push(repo);
				} else {
					find_repos_in_dir(&path)?;
				}
			}
		}
	}

	Ok(repos)
}

pub fn ask(question: &str) -> bool {
	print!("{} [Y/n]", question);
	stdout().flush().unwrap();

	let mut input = String::new();
	stdin().read_line(&mut input).unwrap();

	let input = input.trim().to_owned();

	match input.as_str() {
		"Y" => true,
		"y" => true,
		"N" => false,
		"n" => false,
		_ => {
			// TODO: handle this case properly
			true
		}
	}
}
