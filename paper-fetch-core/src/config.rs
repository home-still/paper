use crate::resilience::config::ResilienceConfig;
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Resilience patterns configuration
    pub resilience: ResilienceConfig,

    /// Directory where downloaded papers are stored
    pub download_path: PathBuf,
    /// Directory for caching metadata and search results
    pub cache_path: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resilience: ResilienceConfig::default(),
            download_path: PathBuf::from("./downloads"),
            cache_path: PathBuf::from("./cache"),
        }
    }
}

impl Config {
    /// Creates config with custom download directory
    pub fn with_download_path(mut self, path: PathBuf) -> Self {
        self.download_path = path;
        self
    }
    /// Creates config with custom cache directory
    pub fn with_cache_path(mut self, path: PathBuf) -> Self {
        self.cache_path = path;
        self
    }
}
