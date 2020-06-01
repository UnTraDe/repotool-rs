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

pub fn get_urls(project: &str, project_type: &ProjectType, filter_forks: bool, only_forks: bool) -> Vec<String> {
	assert!(!(filter_forks && only_forks));

	let (resp, next_url) = common::request_get(&format!("https://api.github.com/{}/{}/repos?per_page=200", project_type.as_str(), project));
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
		} else {
			panic!("repo should be an object");
		}
	}

	if !next_url.is_empty() {
		let mut next_urls = get_urls(&next_url, project_type, filter_forks, only_forks);
		urls.append(&mut next_urls);
	}

	urls
}