use std::fs::File;
use std::io::prelude::*;
use crate::common;

fn get_crate_repository_url(crate_name: &str) -> Option<String> {
	let (resp, _) = common::request_get(&format!("https://crates.io/api/v1/crates/{}", crate_name)).unwrap();
	let repos: serde_json::Value = serde_json::from_str(&resp).unwrap();
	let crate_object = repos.as_object().expect("result is not an object").get("crate").unwrap().as_object().unwrap();

	if let Some(repo_url) = crate_object.get("repository").unwrap().as_str() {
		return Some(repo_url.to_owned())
	} else {
		None
	}
}

pub fn get_urls(crates_file: &str) -> Vec<String> {
	let mut file = File::open(crates_file).unwrap();
	let mut contents = String::new();
	file.read_to_string(&mut contents).unwrap();
	contents = contents.replace("\r", "");
	let mut crates = Vec::new();

	for line in contents.split("\n") {
		let parts: Vec<&str> = line.split(' ').collect();
		let crate_name = parts[1];
		crates.push(crate_name);
	}

	crates.sort_unstable();
	crates.dedup();

	let mut urls = Vec::new();

	for c in crates {
		if let Some(repo_url) = get_crate_repository_url(c) {
			urls.push(repo_url);
		} else {
			println!("crate {} has no repository url", c);
		}
	}

	urls.sort_unstable();
	urls.dedup();

	urls
}