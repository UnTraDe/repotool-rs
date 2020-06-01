pub fn request_get(url: &str)  -> (String, String) {
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

	(resp.text().unwrap(), next_url)
}