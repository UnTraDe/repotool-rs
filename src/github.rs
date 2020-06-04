use crate::common;

pub enum ProjectType {
	None,
	Organization,
	User
}

impl ProjectType {
	pub fn as_str(&self) -> &str {
		match self {
			ProjectType::None => "",
			ProjectType::Organization => "orgs",
			ProjectType::User => "orgs"
		}
	}
}

fn get_urls_from_url(url: &str, project_type: &ProjectType, filter_forks: bool, only_forks: bool, get_submodules: bool) -> Vec<String> {
	assert!(!(filter_forks && only_forks));

	let (resp, next_url) = common::request_get(&url).expect(&format!("failed: {} ", url));
	let repos: serde_json::Value = serde_json::from_str(&resp).unwrap();
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

			if get_submodules {
				let full_name = repo.get("full_name").unwrap().as_str().unwrap();
				let default_branch = repo.get("default_branch").unwrap().as_str().unwrap();
				let gitmodules_url = format!("https://raw.githubusercontent.com/{}/{}/.gitmodules", full_name, default_branch);

				//let (gitmodules, _) = common::request_get(&gitmodules_url) 
				let resp = common::request_get(&gitmodules_url);

				match resp {
					Ok((gitmodules, _)) => {
						let mut submodules = common::get_urls_from_git(gitmodules);
						urls.append(&mut submodules);
					}
					Err(code) => {
						if code != reqwest::StatusCode::NOT_FOUND {
							panic!("request to {} had unexpected response {}", gitmodules_url, code);
						}
					}
				}
			}
		} else {
			panic!("repo should be an object");
		}
	}

	if !next_url.is_empty() {
		let mut next_urls = get_urls_from_url(&next_url, project_type, filter_forks, only_forks, get_submodules);
		urls.append(&mut next_urls);
	}

	urls
}


pub fn get_urls(project: &str, project_type: &ProjectType, filter_forks: bool, only_forks: bool, get_submodules: bool) -> Vec<String> {
	let url = format!("https://api.github.com/{}/{}/repos?per_page=200", project_type.as_str(), project);
	get_urls_from_url(&url, project_type, filter_forks, only_forks, get_submodules)
}
