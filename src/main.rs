use std::env;
use std::fs::*;
use std::path::Path;
use std::io::prelude::*;

mod common;
mod scanner;
mod github;
mod gitlab;
mod crates;
mod gitweb;

fn write_urls(urls: &Vec<String>, mut output_filename: &str, compare_file: &str, prepand_command: &str) {
	let mut compare_list = Vec::new();

	if !compare_file.is_empty() {
		let mut file = File::open(compare_file).unwrap();
		let mut contents = String::new();
		file.read_to_string(&mut contents).unwrap();

		contents = contents.replace("\r", "");
		
		for line in contents.split("\n") {
			compare_list.push(line.trim().to_string().to_lowercase());
		}
	}

	if output_filename.is_empty() {
		output_filename = "output.txt";
	}
	
	let filtered = urls.iter()
		.filter(|u| {
			let p = Path::new(u);
			let mut alt = String::new();
			let u = u.to_string().to_lowercase();

			if let Some(ext) = p.extension() {
				if ext == "git" {
					alt = u[0..u.len()-4].to_owned();
				}
			}

			if alt.is_empty() {
				alt = u.to_string();
				alt.push_str(".git");
			}

			!compare_list.contains(&u) && !compare_list.contains(&alt)
		})
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

enum Action {
	None,
	DownloadGithub,
	DownloadGitlab,
	DownloadCrates,
	DownloadGitweb,
	ScanReposDir,
	GitSubmodulesToUrls
}

fn main() {
	let args: Vec<String> = env::args().collect();

	let mut prepand_command = String::new();
	let mut output_filename = String::new();
	let mut input_filename = String::new();
	let mut project = String::new();
	let mut project_type = github::ProjectType::None;
	let mut reposdir = String::new();
	let mut compare_file = String::new();
	let mut filter_forks = false;
	let mut only_forks = false;
	let mut cmd = Action::None;
	let mut gitlab_group = String::new();
	let mut crates_file = String::new();
	let mut gitweb_url = String::new();
	let mut get_submodules = false;

	for (i, arg) in args.iter().enumerate() {
		match arg.as_str() {
			"-d" | "--default" => {
				prepand_command = "git clone --mirror".to_string();
				let org_name = args.get(i + 1).unwrap();
				output_filename = org_name.clone();
				output_filename.push_str(".txt");
				project = org_name.clone();
				project_type = github::ProjectType::Organization;
				cmd = Action::DownloadGithub;
			}
			"-o" => {
				output_filename = args.get(i + 1).unwrap().clone();
			}
			"-m" => {
				input_filename =  args.get(i + 1).unwrap().clone();
				cmd = Action::GitSubmodulesToUrls;
			}
			"-p" | "--prepend" => {
				prepand_command = args.get(i + 1).unwrap().clone();
			}
			"--github-org" | "--github-orgs" => {
				project = args.get(i + 1).unwrap().to_owned();
				project_type = github::ProjectType::Organization;
				cmd = Action::DownloadGithub;
			}
			"--github-user" | "--github-users" => {
				project = args.get(i + 1).unwrap().to_owned();
				project_type = github::ProjectType::User;
				cmd = Action::DownloadGithub;
			}
			"--gitlab-group" => {
				gitlab_group = args.get(i + 1).unwrap().clone();
				cmd = Action::DownloadGitlab;
			},
			"--gitweb" => {
				gitweb_url = args.get(i + 1).unwrap().clone();
				cmd = Action::DownloadGitweb;
			},
			"--crates" => {
				crates_file = args.get(i + 1).unwrap().clone();
				cmd = Action::DownloadCrates;
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
			"--get-submodules" => {
				get_submodules = true;
			}
			_ => {}
		}
	}

	match cmd {
		Action::None => {
			println!("nothing to be done, TODO: print help");
		}
		Action::DownloadGithub => {
			let urls = github::get_urls(&project, &project_type, filter_forks, only_forks, get_submodules);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		}
		Action::DownloadGitlab => {
			let urls = gitlab::get_urls(&gitlab_group);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		},
		Action::DownloadGitweb => {
			let urls = gitweb::get_urls(&gitweb_url);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		}
		Action::DownloadCrates => {
			let urls = crates::get_urls(&crates_file);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		}
		Action::ScanReposDir => {
			let urls = scanner::scan_repos(Path::new(reposdir.as_str()), 0);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		}
		Action::GitSubmodulesToUrls => {
			assert!(!input_filename.is_empty());
			let mut file = File::open(input_filename).unwrap();
			let mut contents = String::new();
			file.read_to_string(&mut contents).unwrap();
			let urls = common::get_urls_from_git(contents);
			write_urls(&urls, &output_filename, &compare_file, &prepand_command);
		}
	}
}