use anyhow::{Context, Result};
use paper_fetch_core::config::Config;
use paper_fetch_core::models::SearchQuery;
use paper_fetch_core::ports::provider::PaperProvider;
use paper_fetch_core::providers::arxiv::ArxivProvider;

use crate::cli::{GlobalOpts, PaperAction, ProviderArg};
use crate::output;

pub async fn run(action: PaperAction, global: &GlobalOpts) -> Result<()> {
    match action {
        PaperAction::Search {
            query,
            search_type,
            max_results,
            offset,
            provider,
        } => {
            let config = Config::load().context("Failed to load config")?;
            let provider = make_provider(&provider, &config)?;

            if global.verbose {
                eprintln!("Searching {} for: {}", provider.name(), query);
            }

            let search_query = SearchQuery {
                query,
                search_type: search_type.into(),
                max_results: max_results as usize,
                offset,
            };

            let result = provider
                .search(&search_query)
                .await
                .context("Search failed")?;

            if global.json {
                output::print_json(&result)?;
            } else {
                output::print_search_result(&result, global);
            }

            Ok(())
        }
        PaperAction::Get { doi, provider } => {
            let config = Config::load().context("Failed to load config")?;
            let provider = make_provider(&provider, &config)?;

            if global.verbose {
                eprintln!("Looking up DOI: {}", doi);
            }

            let paper = provider
                .get_by_doi(&doi)
                .await
                .context("DOI lookup failed")?;

            match paper {
                Some(p) => {
                    if global.json {
                        output::print_json(&p)?;
                    } else {
                        output::print_paper(&p, global);
                    }
                }
                None => {
                    anyhow::bail!("Not found: {}", doi)
                }
            }

            Ok(())
        }
    }
}

fn make_provider(provider: &ProviderArg, config: &Config) -> Result<Box<dyn PaperProvider>> {
    match provider {
        ProviderArg::Arxiv => {
            let p = ArxivProvider::new(&config.providers.arxiv)
                .context("Failed to create arXiv provider")?;
            Ok(Box::new(p))
        }
    }
}
