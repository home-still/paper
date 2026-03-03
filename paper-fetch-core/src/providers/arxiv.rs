use async_trait::async_trait;
use reqwest::Client;
use roxmltree;

use crate::error::PaperFetchError;
use crate::models::{Paper, SearchQuery, SearchType};
use crate::ports::provider::PaperProvider;

pub struct ArxivProvider {
    client: Client,
    base_url: String,
}

impl ArxivProvider {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap(), // TODO: Remove later
            base_url: String::from("http://export.arxiv.org/api/query"),
        }
    }

    fn build_query_url(&self, query: &SearchQuery) -> String {
        let search_prefix = match query.search_type {
            SearchType::Keywords => "all:",
            SearchType::Title => "ti:",
            SearchType::Author => "au;",
            _ => "all:",
        };

        format!(
            "{}?search_query={}\"{}\"&start={}&max_results={}",
            self.base_url, search_prefix, query.query, query.offset, query.max_results
        )
    }

    fn parse_atom_feed(&self, xml: &str) -> Result<Vec<Paper>, PaperFetchError> {
        let doc = roxmltree::Document::parse(xml)
            .map_err(|e| PaperFetchError::ParseError(e.to_string()))?;

        let root = doc.root_element();
        let ns = "http://www.w3.org/2005/Atom";
4
        let papers: Vec<Paper> = root
            .children()
            .filter(|n| n.has_tag_name((ns, "entry")))
            .filter_map(|entry| self.extract_paper(entry, ns).ok())
            .collect();

        Ok(papers)
    }
}

#[async_trait]
impl PaperProvider for ArxivProvider {
    fn name(&self) -> &'static str {
        "arxiv"
    }

    fn priority(&self) -> u8 {
        80 // High priority for CS/Physics papers
    }

    fn supported_search_types(&self) -> Vec<SearchType> {
        vec![SearchType::Keywords, SearchType::Title, SearchType::Author]
    }
}
