use anyhow::Result;
use paper_fetch_core::models::{Paper, SearchResult};
use serde::Serialize;

use crate::cli::GlobalOpts;

/// Print any Serialize value as JSON to stdout.
pub fn print_json(value: &impl Serialize) -> Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{json}");
    Ok(())
}

/// Print search results as a human-readable list.
pub fn print_search_result(result: &SearchResult, _global: &GlobalOpts) {
    eprintln!(
        "Found {} results from {} (showing {})\n",
        result.total_results,
        result.provider,
        result.papers.len()
    );

    for (i, paper) in result.papers.iter().enumerate() {
        print_paper_row(i + 1, paper);
    }

    if let Some(offset) = result.next_offset {
        if offset < result.total_results {
            eprintln!(
                "\nMore results available. Use --offset {} to see next page.",
                offset
            );
        }
    }
}

fn print_paper_row(index: usize, paper: &Paper) {
    let authors = paper
        .authors
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let date = paper
        .publication_date
        .map(|d| d.to_string())
        .unwrap_or_default();

    println!("{}. {}", index, paper.title);
    println!("   {} ({})", authors, date);
    print!("   {}", paper.id);
    if let Some(doi) = &paper.doi {
        print!("  doi:{doi}");
    }
    println!();
    if let Some(url) = &paper.download_url {
        println!("   {url}");
    }
    println!();
}

/// Print a single paper in human-readable format.
pub fn print_paper(paper: &Paper, _global: &GlobalOpts) {
    let authors = paper
        .authors
        .iter()
        .map(|a| a.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    println!("Title:    {}", paper.title);
    println!("Authors:  {}", authors);
    if let Some(date) = paper.publication_date {
        println!("Date:     {}", date);
    }
    println!("ID:       {}", paper.id);
    if let Some(doi) = &paper.doi {
        println!("DOI:      {}", doi);
    }
    if let Some(url) = &paper.download_url {
        println!("PDF:      {}", url);
    }
    if let Some(abs) = &paper.abstract_text {
        println!("\n{}", abs);
    }
}
