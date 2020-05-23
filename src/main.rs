use std::env;
use std::fs::OpenOptions;
use std::fs::File;
use std::io::prelude::*;

fn get_github_url(t: &str, name: &str) -> String {
	format!("https://api.github.com/{}/{}/repos?per_page=200", t, name)
}

fn github_to_list(url: &str, verbose: bool, filter_forks: bool, only_forks: bool) -> Vec<String> {
	assert!(!(filter_forks && only_forks));

	if verbose {
		println!("GET {}", url)
	}

	let client = reqwest::blocking::Client::new();

	let resp = client.get(url)
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "en-US,en;q=0.5")
		//.header("Accept-Encoding", "gzip, deflate, br")
		.send()
		.unwrap();
	

	if resp.status() != reqwest::StatusCode::OK {
		panic!("failed to get response code 200 from: {}, instead got: {:?}", url, resp.status());
	}

	let mut next_url = String::new();

	if let Some(next_link) = resp.headers().get("link") {
		for link in next_link.to_str().unwrap().split(',') {
			let entries: Vec<&str> = link.split(";").collect();

			if entries[1].split('=').collect::<Vec<&str>>()[1].replace('"', "") == "next" {
				next_url = entries[0]
					.replace("<", "")
					.replace(">", "");

				println!("found next page url: {}", next_url);
				break;
			}
		}
	}

	let repos: serde_json::Value = serde_json::from_str(&resp.text().unwrap()).unwrap();
	let repos_array = repos.as_array().expect("result is not array");
	let mut urls = Vec::new();

	for repo in repos_array {
		if let Some(repo) = repo.as_object() {
			let is_fork = repo.get("fork").expect("object does not have 'fork' attribute")
			.as_bool().expect("'fork' should be bool");
		
			if filter_forks && is_fork {
				continue;
			}

			if only_forks && !is_fork {
				continue;
			}

			urls.push(String::from(repo.get("clone_url").expect("should have 'clone_url' attribute")
				.as_str().expect("clone_url should be a string")));
		} else {
			panic!("repo should be an object");
		}
	}

	if verbose {
		println!("found {0} repositories", urls.len());
	}

	if !next_url.is_empty() {
		let mut next_urls = github_to_list(&next_url, verbose, filter_forks, only_forks);
		urls.append(&mut next_urls);
	}

	urls
}

fn write_urls(urls: &Vec<String>, mut output_filename: &str, compare_list: Vec<String>, prepand_command: &str) {
	if output_filename.is_empty() {
		output_filename = "output.txt";
	}
	
	let filtered = urls.iter()
		.filter(|u| !compare_list.contains(u))
		.collect::<Vec<&String>>();

	let mut f = OpenOptions::new()
		.write(true)
		.create_new(true)
		.open(output_filename)
		.expect("failed to open file");

	let temp = filtered.len(); // why the following line is moving filtered?

	for url in filtered {
		f.write(prepand_command.as_bytes()).unwrap();
		f.write(b" ").unwrap();
		f.write(url.as_bytes()).unwrap();
		f.write(b"\n").unwrap();
	}

    println!("written {} repositories to {} (skipped {})", temp, output_filename, (urls.len() - temp))
}

fn download_and_save_from_github(url: &str, output_filename: &str, verbose: bool, filter_forks: bool, only_forks: bool, compare_file: &str, prepand_command: &str) {
	let urls = github_to_list(url, verbose, filter_forks, only_forks);
	let mut compare_list = Vec::new();

	if !compare_file.is_empty() {
		let mut file = File::open(compare_file).unwrap();
		let mut contents = String::new();
		file.read_to_string(&mut contents).unwrap();

		contents = contents.replace("\r", "");
		
		for line in contents.split("\n") {
			compare_list.push(line.trim().to_string());
		}
	}

	write_urls(&urls, output_filename, compare_list, prepand_command);
}

fn gitlab_get_subgroups(group_url: &str) -> Vec<u64> {
	let client = reqwest::blocking::Client::new();

	let resp = client.get(group_url)
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "en-US,en;q=0.5")
		//.header("Accept-Encoding", "gzip, deflate, br")
		.send()
		.unwrap();
	

	if resp.status() != reqwest::StatusCode::OK {
		panic!("failed to get response code 200 from: {}, instead got: {:?}", group_url, resp.status());
	}

	let repos: serde_json::Value = serde_json::from_str(&resp.text().unwrap()).unwrap();
	let repos_array = repos.as_array().expect("result is not array");
	let mut groups = Vec::new();

	for repo in repos_array {
		if let Some(repo) = repo.as_object() {
			let id = repo.get("id")
						.expect("should have 'id' attribute")
						.as_u64()
						.expect("id should be an integer");

			groups.push(id);
		} else {
			panic!("repo should be an object");
		}
	}

	groups
}

fn gitlab_get_repositories(group_url: &str) -> Vec<String> {
	let client = reqwest::blocking::Client::new();

	let resp = client.get(group_url)
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "en-US,en;q=0.5")
		//.header("Accept-Encoding", "gzip, deflate, br")
		.send()
		.unwrap();
	

	if resp.status() != reqwest::StatusCode::OK {
		panic!("failed to get response code 200 from: {}, instead got: {:?}", group_url, resp.status());
	}

	let repos: serde_json::Value = serde_json::from_str(&resp.text().unwrap()).unwrap();
	let repos_array = repos.as_array().expect("result is not array");
	let mut urls = Vec::new();

	for repo in repos_array {
		if let Some(repo) = repo.as_object() {
			urls.push(String::from(repo.get("http_url_to_repo").expect("should have 'http_url_to_repo' attribute")
				.as_str().expect("http_url_to_repo should be a string")));
		} else {
			panic!("repo should be an object");
		}
	}

	urls
}

fn gitlab_to_list(group_name: &str) -> Vec<String> {
	let mut url_list = Vec::new();

	let group_api_url = format!("https://gitlab.com/api/v4/groups/{}/subgroups", group_name);
	let subgroups = gitlab_get_subgroups(&group_api_url);
	//println!("{:?}", subgroups);

	for group_id in subgroups {
		let mut urls = gitlab_to_list(&group_id.to_string());
		url_list.append(&mut urls);
	}

	let repositories_api_url = format!("https://gitlab.com/api/v4/groups/{}/projects", group_name);
	let mut repositories = gitlab_get_repositories(&repositories_api_url);
	//println!("{:?}", repositories);
	url_list.append(&mut repositories);

	url_list
}

fn download_and_save_from_gitlab(group_name: &str, output_filename: &str, compare_file: &str, prepand_command: &str) {
	let urls = gitlab_to_list(group_name);
	let mut compare_list = Vec::new();

	if !compare_file.is_empty() {
		let mut file = File::open(compare_file).unwrap();
		let mut contents = String::new();
		file.read_to_string(&mut contents).unwrap();

		contents = contents.replace("\r", "");
		
		for line in contents.split("\n") {
			compare_list.push(line.trim().to_string());
		}
	}

	write_urls(&urls, output_filename, compare_list, prepand_command);
}

enum Action {
	None,
	DownloadGithub,
	DownloadGitlab,
	ScanReposDir
}

fn main() {
	let args: Vec<String> = env::args().collect();
	//println!("{:?}", args);

	let mut prepand_command = String::new();
	let mut output_filename = String::new();
	let mut url = String::new();
	// let mut input_filename = String::new();
	let mut reposdir = String::new();
	let mut compare_file = String::new();
	let mut filter_forks = false;
	let mut only_forks = false;
	let mut cmd = Action::None;
	let mut gitlab_group = String::new();
	
	for (i, arg) in args.iter().enumerate() {
		match arg.as_str() {
			"-d" | "--default" => {
				prepand_command = "git clone --mirror".to_string();
				let org_name = args.get(i + 1).unwrap();
				output_filename = org_name.clone();
				output_filename.push_str(".txt");
				url = get_github_url("orgs", org_name);
				cmd = Action::DownloadGithub;
			}
			// "-i" => {
			// 	input_filename = args.get(i + 1).unwrap().clone();
			// }
			"-o" => {
				output_filename = args.get(i + 1).unwrap().clone();
			}
			// "-m2url" => {
			// 	unimplemented!();
			// }
			"-p" | "--prepend" => {
				prepand_command = args.get(i + 1).unwrap().clone();
			}
			"--github-org" | "--github-orgs" => {
				url = get_github_url("orgs", args.get(i + 1).unwrap());
				cmd = Action::DownloadGithub;
			}
			"--github-user" | "--github-users" => {
				url = get_github_url("users", args.get(i + 1).unwrap());
				cmd = Action::DownloadGithub;
			}
			"--gitlab-group" => {
				gitlab_group = args.get(i + 1).unwrap().clone();
				cmd = Action::DownloadGitlab;
			}
			"--scan-repos" => {
				reposdir = args.get(i + 1).unwrap().clone();
				cmd = Action::ScanReposDir;
			}
			"--filter-forks" => {
				filter_forks = true;
			}
			"--only-forks" => {
				only_forks = true;
			}
			"-c" => {
				compare_file = args.get(i + 1).unwrap().clone();
			}
			_ => {}
		}
	}

	match cmd {
		Action::None => {
			println!("nothing to be done, TODO: print help");
		}
		Action::DownloadGithub => {
			download_and_save_from_github(url.as_str(), output_filename.as_str(), false, filter_forks, only_forks, &compare_file, prepand_command.as_str());
		}
		Action::DownloadGitlab => {
			download_and_save_from_gitlab(&gitlab_group, output_filename.as_str(), &compare_file, prepand_command.as_str());
		}
		Action::ScanReposDir => {
			unimplemented!();
		}
	}
}

