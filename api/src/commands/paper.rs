use std::sync::Arc;

use anyhow::{Context, Result};
use paper_fetch_core::config::Config;
use paper_fetch_core::models::SearchQuery;
use paper_fetch_core::ports::provider::PaperProvider;
use paper_fetch_core::providers::arxiv::ArxivProvider;
use paper_fetch_core::providers::downloader::PaperDownloader;
use paper_fetch_core::services::download::{download_batch, ProgressEvent};

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
        PaperAction::Download {
            query,
            doi,
            max_results,
            concurrency,
            search_type,
            provider,
        } => {
            let config = Config::load().context("Failed to load config")?;

            let downloader = PaperDownloader::new(config.download_path.clone(), &config.download)
                .context("Failed to create downloader")?;
            let downloader: Arc<dyn paper_fetch_core::ports::download_service::DownloadService> =
                Arc::new(downloader);

            if let Some(doi_str) = doi {
                // Single DOI download
                if global.verbose {
                    eprintln!("Downloading DOI: {}", doi_str);
                }

                let result = downloader
                    .download_by_doi(&doi_str)
                    .await
                    .context("Download failed")?;

                if global.json {
                    output::print_json(&result)?;
                } else {
                    eprintln!("Downloaded: {}", result.file_path.display());
                    eprintln!("SHA256: {}", result.sha256);
                    eprintln!("Size: {} bytes", result.size_bytes);
                }
            } else if let Some(query_str) = query {
                // Search + batch download
                let provider_impl = make_provider(&provider, &config)?;

                if !global.quiet {
                    eprintln!("Searching {} for {}", provider_impl.name(), query_str);
                }

                let search_query = SearchQuery {
                    query: query_str,
                    search_type: search_type.into(),
                    max_results: max_results as usize,
                    offset: 0,
                };

                let search_result = provider_impl
                    .search(&search_query)
                    .await
                    .context("Search failed")?;

                let paper_count = search_result.papers.len();
                if paper_count == 0 {
                    if !global.quiet {
                        eprintln!("No papers found.");
                    }
                    return Ok(());
                }

                if !global.quiet {
                    eprintln!(
                        "Found {} papers. Downloading (concurrency={})...",
                        paper_count, concurrency
                    );
                }

                let quiet = global.quiet;
                let progress: Option<paper_fetch_core::services::download::ProgressCallback> =
                    if quiet {
                        None
                    } else {
                        Some(Box::new(
                            move |done: usize, total: usize, title: &str, event: &ProgressEvent| {
                                match event {
                                    ProgressEvent::Started => {
                                        eprintln!(
                                            "[{}/{}] Downloading: {}",
                                            done + 1,
                                            total,
                                            title
                                        );
                                    }
                                    ProgressEvent::Completed(size) => {
                                        eprintln!(
                                            "[{}/{}] Done ({} bytes): {}",
                                            done, total, size, title
                                        );
                                    }
                                    ProgressEvent::Failed(err) => {
                                        eprintln!(
                                            "[{}/{}] FAILED: {} -- {}",
                                            done, total, title, err
                                        );
                                    }
                                }
                            },
                        ))
                    };

                let batch_result =
                    download_batch(downloader, search_result.papers, concurrency, progress).await;

                if global.json {
                    output::print_json(&batch_result)?;
                } else {
                    eprintln!(
                        "\nCompleted: {}/{} succeeded, {} failed",
                        batch_result.succeeded.len(),
                        batch_result.total_requested,
                        batch_result.failed.len(),
                    );
                    if !batch_result.failed.is_empty() {
                        eprintln!("\nFailed downloads:");
                        for f in &batch_result.failed {
                            eprintln!("  {} -- {}", f.paper_id, f.error);
                        }
                    }
                }

                if !batch_result.failed.is_empty() {
                    anyhow::bail!(
                        "{} of {} downloads failed",
                        batch_result.failed.len(),
                        batch_result.total_requested
                    );
                }
            } else {
                anyhow::bail!("Provide either a search query or --doi");
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
