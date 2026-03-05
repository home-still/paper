use crate::resilience::config::ResilienceConfig;
use anyhow::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Resilience patterns configuration
    pub resilience: ResilienceConfig,

    /// Directory where downloaded papers are stored
    pub download_path: PathBuf,

    /// Directory for caching metadata and search results
    pub cache_path: PathBuf,

    /// Paper providers
    pub providers: ProvidersConfig,

    /// Download config
    pub download: DownloadConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            resilience: ResilienceConfig::default(),
            download_path: dirs::home_dir()
                .map(|h| h.join("Downloads/home-still/papers"))
                .unwrap_or_else(|| PathBuf::from("./downloads")),
            cache_path: dirs::home_dir()
                .map(|h| h.join(".home-still/paper-fetch/cache"))
                .unwrap_or_else(|| PathBuf::from("./cache")),
            providers: ProvidersConfig::default(),
            download: DownloadConfig::default(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::home_dir().map(|h| h.join(".home-still/paper-fetch/config.yaml"))
    }

    pub fn load() -> anyhow::Result<Self> {
        let Some(path) = Self::config_path() else {
            return Ok(Self::default());
        };

        if !path.exists() {
            let config = Self::default();

            // Create parent directories
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).with_context(|| {
                    format!("Failed to create config directory {}", parent.display())
                })?;
            }

            // Write default config
            let yaml =
                serde_yaml_ng::to_string(&config).context("Failed to serialize default config")?;

            fs::write(&path, yaml)
                .with_context(|| format!("Failed to write default config to {}", path.display()))?;

            eprintln!("Created default config at {}", path.display());

            return Ok(config);
        }

        let contents = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read config at {}", path.display()))?;

        serde_yaml_ng::from_str(&contents)
            .with_context(|| format!("Failed to parse config at {}", path.display()))
    }

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArxivConfig {
    pub base_url: String,
    pub timeout_secs: u64,
}

impl Default for ArxivConfig {
    fn default() -> Self {
        Self {
            base_url: String::from("http://export.arxiv.org/api/query"),
            timeout_secs: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProvidersConfig {
    pub arxiv: ArxivConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    /// Maximum concurrent downloads
    pub max_concurrent: usize,
    /// Per-file download timeout in seconds
    pub timeout_secs: u64,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 4,
            timeout_secs: 120,
        }
    }
}
