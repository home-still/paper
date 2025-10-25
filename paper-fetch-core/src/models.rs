use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub name: String,
    pub affiliation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub abstract_text: Option<String>,
    pub publication_date: Option<NaiveDate>,
    pub doi: Option<String>,
    pub download_url: Option<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchType {
    Keywords,
    Title,
    Author,
    DOI,
    Subject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub search_type: SearchType,
    pub max_results: usize,
    pub offset: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub papers: Vec<Paper>,
    pub total_results: usize,
    pub next_offset: Option<usize>,
    pub provider: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    pub file_path: PathBuf,
    pub doi: Option<String>,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperMetadata {
    pub title: Option<String>,
    pub authors: Vec<Author>,
    pub doi: Option<String>,
    pub confidence_score: f32,
}
