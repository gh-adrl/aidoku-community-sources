use aidoku::prelude::format;
use aidoku::{
	std::{String, Vec},
	Chapter, Manga, MangaContentRating, MangaStatus,
};
use alloc::string::ToString;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GraphQLResponse<T> {
	pub data: T,
}

#[derive(Debug, Deserialize)]
pub struct Nodes<T> {
	pub nodes: Vec<T>,
}

#[derive(Debug, Deserialize)]
pub struct Mangas {
	pub mangas: Nodes<MangaDto>,
}

#[derive(Debug, Deserialize)]
pub struct MangaData {
	pub manga: MangaDto,
}

#[derive(Default, Debug, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MangaDto {
	pub id: u32,
	pub title: String,
	pub thumbnail_url: String,
	pub author: Option<String>,
	pub artist: Option<String>,
	pub url: String,
	pub genre: Vec<String>,
	pub status: String,
	pub description: String,
}

impl MangaDto {
	pub fn into_manga<T: AsRef<str>>(self, base_url: T) -> Manga {
		let base_url = base_url.as_ref();

		Manga {
			id: self.id.to_string(),
			url: [base_url, &self.url].concat(),
			cover: [base_url, &self.thumbnail_url].concat(),
			title: self.title.clone(),
			author: self.author.clone().unwrap_or("Unknown".to_string()),
			artist: self.artist.clone().unwrap_or("Unknown".to_string()),
			description: self.description.clone(),
			categories: self.genre.clone(),
			// https://github.com/Suwayomi/tachiyomi-extension/blob/main/src/all/tachidesk/src/eu/kanade/tachiyomi/extension/all/tachidesk/Tachidesk.kt#L666
			status: match self.status.as_str() {
				"ONGOING" => MangaStatus::Ongoing,
				"COMPLETED" => MangaStatus::Completed,
				"CANCELLED" => MangaStatus::Cancelled,
				"ON_HIATUS" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			},
			nsfw: MangaContentRating::Safe,        // TODO: Fix this
			viewer: aidoku::MangaViewer::Vertical, // TODO: Fix this
		}
	}
}

#[derive(Debug, Deserialize)]
pub struct Chapters {
	pub chapters: Nodes<ChapterDto>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChapterDto {
	pub id: u32,
	pub chapter_number: f32,
	pub name: String,
	pub scanlator: Option<String>,
	pub manga: SlimManga,
	pub manga_id: u32,
	pub source_order: i32,
	pub upload_date: String,
}

#[derive(Debug, Deserialize)]
pub struct SlimManga {
	pub source: Source,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
	pub display_name: String,
}

impl ChapterDto {
	pub fn into_chapter<T: AsRef<str>>(self, base_url: T) -> Chapter {
		let base_url = base_url.as_ref();
		let url = [
			base_url,
			&format!("/manga/{}/chapter/{}", self.manga_id, self.source_order),
		]
		.concat();

		let date_updated = self
			.upload_date
			.parse::<f64>()
			.map(|ms| ms / 1000.0)
			.unwrap_or(0.0);

		Chapter {
			id: self.id.to_string(),
			title: self.name.clone(),
			chapter: self.chapter_number,
			date_updated,
			scanlator: self
				.scanlator
				.clone()
				.unwrap_or(self.manga.source.display_name.clone()),
			url,
			..Default::default()
		}
	}
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchChaptersPages {
	pub fetch_chapter_pages: ChapterPages,
}

#[derive(Debug, Deserialize)]
pub struct ChapterPages {
	pub pages: Vec<String>,
}
