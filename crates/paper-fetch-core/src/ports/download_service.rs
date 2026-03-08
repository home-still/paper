use crate::error::PaperFetchError;
use crate::models::DownloadResult;
use async_trait::async_trait;

#[async_trait]
pub trait DownloadService: Send + Sync {
    async fn download_by_doi(&self, doi: &str) -> Result<DownloadResult, PaperFetchError>;

    async fn download_by_url(
        &self,
        url: &str,
        filename: &str,
    ) -> Result<DownloadResult, PaperFetchError>;
}
