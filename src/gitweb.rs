use crate::common;
use scraper::{Html, Selector};

pub fn get_urls(url: &str) -> Vec<String> {
	let (resp, next_url) = common::request_get(url).unwrap();
	assert!(next_url.is_empty());
	let html = Html::parse_document(&resp);
	let selector = Selector::parse("table.project_list td>a").unwrap();
	let mut urls = Vec::new();

	for name in html.select(&selector) {
		let r = name.inner_html();
		
		if r.ends_with(".git") {
			urls.push(r);
		}
	}
	
	urls
}