use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::{Context, Result};
use hs_style::reporter::Reporter;
use hs_style::styles::Styles;
use paper_core::config::Config;
use paper_core::models::SearchQuery;
use paper_core::ports::provider::PaperProvider;
use paper_core::providers::arxiv::ArxivProvider;
use paper_core::providers::downloader::PaperDownloader;
use paper_core::services::download::{download_batch, DownloadEvent, OnProgress};

use crate::cli::{GlobalOpts, PaperAction, ProviderArg};
use crate::output;

pub async fn run(
    action: PaperAction,
    global: &GlobalOpts,
    reporter: &Arc<dyn Reporter>,
    styles: &Styles,
) -> Result<()> {
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

            let stage = reporter.begin_stage("Searching", None);
            stage.set_message(&format!("{} for:{}", provider.name(), query));

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

            stage.finish_and_clear();

            if global.is_json() {
                output::print_json(&result)?;
            } else {
                output::print_search_result(&result, styles);
            }

            Ok(())
        }
        PaperAction::Get { doi, provider } => {
            let config = Config::load().context("Failed to load config")?;
            let stage = reporter.begin_stage("Looking up", None);
            stage.set_message(&format!("DOI: {}", doi));

            let provider = make_provider(&provider, &config)?;

            let paper = provider
                .get_by_doi(&doi)
                .await
                .context("DOI lookup failed")?;

            stage.finish_and_clear();

            match paper {
                Some(p) => {
                    if global.is_json() {
                        output::print_json(&p)?;
                    } else {
                        output::print_paper(&p, styles);
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
            let downloader: Arc<dyn paper_core::ports::download_service::DownloadService> =
                Arc::new(downloader);

            if let Some(doi_str) = doi {
                // Single DOI download

                let stage = reporter.begin_stage("Downloading", None);
                stage.set_message(&format!("DOI: {}", doi_str));

                let result = downloader
                    .download_by_doi(&doi_str)
                    .await
                    .context("Download failed")?;

                stage.finish_and_clear();
                if global.is_json() {
                    output::print_json(&result)?;
                } else {
                    reporter.finish(&format!(
                        "{} ({} bytes)",
                        result.file_path.display(),
                        result.size_bytes
                    ));
                }
            } else if let Some(query_str) = query {
                // Search + batch download
                let provider_impl = make_provider(&provider, &config)?;

                let search_stage = reporter.begin_stage("Searching", None);
                search_stage.set_message(&format!("{} for {}", provider_impl.name(), query_str));

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

                search_stage.finish_and_clear();

                let paper_count = search_result.papers.len();
                if paper_count == 0 {
                    reporter.warn("No papers found.");
                    return Ok(());
                }

                reporter.status(
                    "Found",
                    &format!(
                        "{} papers, downloading (concurrency={})...",
                        paper_count, concurrency
                    ),
                );

                let bars: Arc<Mutex<HashMap<usize, Box<dyn hs_style::reporter::StageHandle>>>> =
                    Arc::new(Mutex::new(HashMap::new()));
                let bars_ref = Arc::clone(&bars);
                let reporter_ref = Arc::clone(reporter);

                let progress: Option<OnProgress> =
                    Some(Arc::new(move |event: DownloadEvent| match event {
                        DownloadEvent::Started { index, title, .. } => {
                            let stage = reporter_ref.begin_stage(&title, None);
                            bars_ref.lock().unwrap().insert(index, stage);
                        }
                        DownloadEvent::Progress {
                            index,
                            bytes_downloaded,
                            bytes_total,
                            ..
                        } => {
                            if let Some(bar) = bars_ref.lock().unwrap().get(&index) {
                                if let Some(total) = bytes_total {
                                    bar.set_length(total);
                                }
                                bar.set_position(bytes_downloaded);
                            }
                        }
                        DownloadEvent::Completed {
                            index, size_bytes, ..
                        } => {
                            if let Some(bar) = bars_ref.lock().unwrap().remove(&index) {
                                bar.finish_with_message(&format_bytes(size_bytes));
                            }
                        }
                        DownloadEvent::Failed { index, error, .. } => {
                            if let Some(bar) = bars_ref.lock().unwrap().remove(&index) {
                                bar.finish_with_message(&format!("FAILED: {}", error));
                            }
                        }
                    }));

                let batch_result =
                    download_batch(downloader, search_result.papers, concurrency, progress).await;

                if global.is_json() {
                    output::print_json(&batch_result)?;
                } else {
                    reporter.finish(&format!(
                        "\nCompleted: {}/{} succeeded, {} failed",
                        batch_result.succeeded.len(),
                        batch_result.total_requested,
                        batch_result.failed.len(),
                    ));
                    if !batch_result.failed.is_empty() {
                        for f in &batch_result.failed {
                            reporter.warn(&format!("{} -- {}", f.paper_id, f.error));
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

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_048_576 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
