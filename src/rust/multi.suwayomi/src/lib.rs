#![no_std]
mod dto;
extern crate alloc;
use crate::dto::{Chapters, FetchChaptersPages, GraphQLResponse, MangaData, Mangas};
use aidoku::{
	error::{AidokuError, AidokuErrorKind, Result},
	helpers::uri::encode_uri,
	prelude::*,
	std::{defaults::defaults_get, net::Request, String, StringRef, Vec},
	Chapter, Filter, FilterType, Listing, Manga, MangaPageResult, Page,
};
use alloc::{borrow::ToOwned, string::ToString, vec};

fn get_base_url() -> Result<String> {
	defaults_get("baseURL")?
		.as_string()
		.map(|v| v.read().trim_end_matches('/').to_string())
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let base_url = get_base_url()?;

	let query = r#"
		query GET_MANGA_LIST($condition: MangaConditionInput, $order: [MangaOrderInput!]) {
			mangas(condition: $condition, order: $order) {
				nodes {
					id
					title
					thumbnailUrl
					author
					artist
					url
					genre
					status
					description
				}
			}
		}
		"#;


	let mut condition = serde_json::Map::new();
	condition.insert("inLibrary".to_string(), serde_json::json!(true));

	let mut order: Vec<serde_json::Value> = Vec::new();

	for filter in filters {
		match filter.kind {
			FilterType::Sort => {
				if let Ok(value) = filter.value.as_object() {
					let index = value.get("index").as_int().unwrap_or(0);
					let ascending = value.get("ascending").as_bool().unwrap_or(true);
					let property = match index {
						0 => "TITLE",
						1 => "IN_LIBRARY_AT",
						2 => "LAST_FETCHED_AT",
						_ => continue,
					};
					order.push(serde_json::json!({
						"by": property,
						"byType": if ascending { "ASC" } else { "DESC" }
					}));
				}
			}
			_ => continue
		}
	}

	let mut variables = serde_json::Map::new();
	variables.insert("condition".to_string(), serde_json::Value::Object(condition));
	variables.insert("order".to_string(), serde_json::Value::Array(order));

	let json_value = serde_json::Value::Object(variables);

	let body = serde_json::json!({
		"operationName": "GET_MANGA_LIST",
		"query": query,
		"variables": json_value,
	});

	let data = Request::post(format!("{base_url}/api/graphql"))
		.header("Content-Type", "application/json")
		.body(body.to_string())
		.data();

	serde_json::from_slice::<GraphQLResponse<Mangas>>(&data)
		.map(|v| MangaPageResult {
			manga: v
				.data
				.mangas
				.nodes
				.into_iter()
				.map(|m| m.into_manga(&base_url))
				.collect(),
			has_more: false,
		})
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	let base_url = get_base_url()?;

	let query = r#"
		query GET_MANGA_DETAILS($id: Int!) {
			manga(id: $id) {
				id
				title
				thumbnailUrl
				author
				artist
				url
				genre
				status
				description
			}
		}
	"#;

	let body = serde_json::json!({
		"operationName": "GET_MANGA_DETAILS",
		"query": query,
		"variables": {
			"id": id.parse::<i32>().expect("Invalid manga ID")
		}
	});

	let data = Request::post(format!("{base_url}/api/graphql"))
		.header("Content-Type", "application/json")
		.body(body.to_string())
		.data();

	serde_json::from_slice::<GraphQLResponse<MangaData>>(&data)
		.map(|v| v.data.manga.into_manga(base_url))
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	let base_url = get_base_url()?;

	let query = r#"
	query GET_CHAPTERS_LIST($condition: ChapterConditionInput, $order: [ChapterOrderInput!]) {
		chapters(condition: $condition, order: $order) {
			nodes {
				id
				chapterNumber
				name
				scanlator
				uploadDate
				mangaId
				sourceOrder
				manga {
					source {
						displayName
					}
				}
			}
		}
	}
	"#;

	let body = serde_json::json!({
		"operationName": "GET_CHAPTERS_LIST",
		"query": query,
		"variables": {
			"condition": { "mangaId": 1 },
			"order": [{ "by": "SOURCE_ORDER", "byType": "DESC" }]
		}
	});

	let data = Request::post(format!("{base_url}/api/graphql"))
		.header("Content-Type", "application/json")
		.body(body.to_string())
		.data();

	serde_json::from_slice::<GraphQLResponse<Chapters>>(&data)
		.map(|v| {
			v.data
				.chapters
				.nodes
				.into_iter()
				.map(|c| c.into_chapter(&base_url))
				.collect()
		})
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}

#[get_page_list]
fn get_page_list(_: String, id: String) -> Result<Vec<Page>> {
	let base_url = get_base_url()?;

	let query = r#"
	mutation GET_PAGE_LIST($input: FetchChapterPagesInput!) {
		fetchChapterPages(input: $input) {
			pages
		}
	}
	"#;

	let body = serde_json::json!({
		"operationName": "GET_PAGE_LIST",
		"query": query,
		"variables": {
			"input": {
				"chapterId": id.parse::<i32>().expect("Invalid number")
			}
		}
	});

	let data = Request::post(format!("{base_url}/api/graphql"))
		.header("Content-Type", "application/json")
		.body(body.to_string())
		.data();

	serde_json::from_slice::<GraphQLResponse<FetchChaptersPages>>(&data)
		.map(|v| {
			v.data
				.fetch_chapter_pages
				.pages
				.into_iter()
				.enumerate()
				.map(|(index, url)| Page {
					index: index as i32,
					url: format!("{}{}", base_url, url),
					..Default::default()
				})
				.collect()
		})
		.map_err(|_| AidokuError {
			reason: AidokuErrorKind::JsonParseError,
		})
}
