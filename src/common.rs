pub fn get_urls_from_git(mut contents: String) -> Vec<String> {
	let mut urls = Vec::new();
	contents = contents.replace("\r", "");
	
	for line in contents.split("\n") {
		let clean = line.replace(" ", "").replace("\t", "").replace("\n", "");
		let pair: Vec<&str> = clean.split('=').collect();
		
		if pair.len() == 2 && pair[0] == "url" {
			urls.push(pair[1].to_owned());
		}
	}

	urls
}

pub fn request_get(url: &str)  -> Result<(String, String), reqwest::StatusCode> {
	let client = reqwest::blocking::Client::new();

	let resp = client.get(url)
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:75.0) Gecko/20100101 Firefox/75.0")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "en-US,en;q=0.5")
		//.header("Accept-Encoding", "gzip, deflate, br")
		.send()
		.unwrap();
	

	if resp.status() != reqwest::StatusCode::OK {
		//panic!("failed to get response code 200 from: {}, instead got: {:?}", url, resp.status());
		return Err(resp.status());
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

	Ok((resp.text().unwrap(), next_url))
}