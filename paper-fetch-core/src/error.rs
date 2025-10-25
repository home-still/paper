use thiserror::Error;

#[derive(Error, Debug)]
pub enum PaperFetchError {
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Provider unavailable: {0}")]
    ProviderUnavailable(String),

    #[error("Rate limited: {provider}, retry after {retry_after:?}")]
    RateLimited {
        provider: String,
        retry_after: Option<std::time::Duration>,
    },

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Parse error: {0}")]
    ParseError(String),
}
