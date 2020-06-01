use crate::common;

fn get_subgroups(group_url: &str) -> Vec<u64> {
	let (resp, next_url) = common::request_get(group_url);
	assert!(next_url.is_empty()); // currently not handling next page

	let repos: serde_json::Value = serde_json::from_str(&resp).unwrap();
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

fn get_repositories(group_url: &str) -> Vec<String> {
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

pub fn get_urls(group_name: &str) -> Vec<String> {
	let mut url_list = Vec::new();

	let group_api_url = format!("https://gitlab.com/api/v4/groups/{}/subgroups", group_name);
	let subgroups = get_subgroups(&group_api_url);

	for group_id in subgroups {
		let mut urls = get_urls(&group_id.to_string());
		url_list.append(&mut urls);
	}

	let repositories_api_url = format!("https://gitlab.com/api/v4/groups/{}/projects", group_name);
	let mut repositories = get_repositories(&repositories_api_url);
	url_list.append(&mut repositories);

	url_list
}