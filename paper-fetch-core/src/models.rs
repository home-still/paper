use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

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

#[derive(Error, Debug)]
pub enum PaperFetchError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
}
