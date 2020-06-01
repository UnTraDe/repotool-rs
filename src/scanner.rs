use std::path::{PathBuf, Path};
use std::fs::*;
use std::io::prelude::*;

fn find_git_dir(path: &Path) -> Option<PathBuf> {
	for d in read_dir(path).unwrap() {
		let entry = d.unwrap();

		if entry.file_type().unwrap().is_dir() && entry.file_name() == ".git" {
			return Some(entry.path());
		}
	}

	None
}

fn find_config(path: &Path) -> Option<PathBuf> {
	for d in read_dir(path).unwrap() {
		let entry = d.unwrap();

		if entry.file_type().unwrap().is_file() && entry.file_name() == "config" {
			return Some(entry.path());
		}
	}

	None
}

fn get_url(path: &Path) -> Option<String> {
	let mut file = File::open(path).unwrap();
	let mut contents = String::new();
	file.read_to_string(&mut contents).unwrap();
	contents = contents.replace("\r", "");
	
	for line in contents.split("\n") {
		let clean = line.replace(" ", "").replace("\t", "").replace("\n", "");
		let pair: Vec<&str> = clean.split('=').collect();
		
		if pair.len() == 2 && pair[0] == "url" {
			return Some(pair[1].to_owned());
		}
	}

	None
}

pub fn scan_repos(reposdir: &Path, level: usize) -> Vec<String> {
	let mut urls = Vec::new();

	for d in read_dir(reposdir).unwrap() {
		let entry = d.unwrap();

		if !entry.file_type().unwrap().is_dir() {
			continue;
		}

		let git_dir;

		if let Some(dir) = find_git_dir(entry.path().as_path()) {
			git_dir = dir;
		} else {
			git_dir = entry.path();
		}
		
		if let Some(cfg) = find_config(git_dir.as_path()) {
			if let Some(url) = get_url(cfg.as_path()) {
				urls.push(url);
			}
		} else {
			let mut subfolder_urls = scan_repos(git_dir.as_path(), level + 1);
			urls.append(&mut subfolder_urls);
		}
	}

	urls
}