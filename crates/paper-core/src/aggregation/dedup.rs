// DOI-first deduplication with fuzzy title matching
use crate::models::{DedupStats, Paper};
use std::collections::HashMap;

pub struct DedupGroup {
    pub papers: Vec<SourcedPaper>,
    pub doi: Option<String>,
    pub maatch_type: MatchType,
}

pub struct SourcedPaper {
    pub paper: Paper,
    pub source: String,
    pub rank: usize, // position in that source's result list (0-indexed)
}

pub enum MatchType {
    Doi,
    FuzzyTitle { similarity: f64 },
    Single,
}

pub fn deduplicate(source_results: Vec<(String, Vec<Paper>)>) -> (Vec<DedupGroup>, DedupStats) {
    let mut stats = DedupStats::default();
    let mut groups: Vec<DedupGroup> = Vec::new();

    // Flatten into SourcedPapers, preserving rank
    let mut all_papers: Vec<SourcedPaper> = Vec::new();
    for (source, papers) in source_results {
        stats.total_raw += papers.len();
        for (rank, paper) in papers.into_iter().enumerate() {
            all_papers.push(SourcedPaper {
                paper,
                source: source.clone(),
                rank,
            });
        }
    }

    // Stage 1: DOI indexing
    let mut doi_map: HashMap<String, Vec<SourcedPaper>> = HashMap::new();
    let mut no_doi: Vec<SourcedPaper> = Vec::new();

    for sp in all_papers {
        if let Some(ref doi) = sp.paper.doi {
            let key = normalize_doi(doi);
            doi_map.entry(key).or_default().push(sp);
        } else {
            no_doi(sp)
        }
    }

    todo!()
}

fn normalize_doi(doi: &str) -> String {
    doi.strip_prefix("https://doi.org/")
        .unwrap_or(doi)
        .to_lowercase()
}

fn preprocess_title(title: &str) -> String {
    let lower = title.to_lowercase();
    let stripped: String = lower
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect();
    stripped
        .split_whitespace()
        .filter(|w| !matches!(*w, "the" | "a" | "an"))
        .collect::<Vec<&str>>()
        .join(" ")
}
