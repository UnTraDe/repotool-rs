use std::fs::File;
use std::io::prelude::*;

fn get_crate_url(crate_name: &str) -> String {
	format!("https://crates.io/api/v1/crates/{}", crate_name)
}

fn get_crate_repository_url(crate_name: &str) -> Option<String> {
	let client = reqwest::blocking::Client::new();
	let crate_url = get_crate_url(crate_name);

	let resp = client.get(crate_url.as_str())
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "en-US,en;q=0.5")
		//.header("Accept-Encoding", "gzip, deflate, br")
		.send()
		.unwrap();
	

	if resp.status() != reqwest::StatusCode::OK {
		panic!("failed to get response code 200 from: {}, instead got: {:?}", crate_url, resp.status());
	}

	let repos: serde_json::Value = serde_json::from_str(&resp.text().unwrap()).unwrap();
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

	urls
}