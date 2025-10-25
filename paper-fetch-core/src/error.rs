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

#[derive(Debug, Clone, Copy)]
pub enum ErrorCategory {
    Permanent,
    Transient,
    RateLimited,
    CircuitBreaker,
}

impl PaperFetchError {
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::InvalidInput(_) => ErrorCategory::Permanent,
            Self::NotFound(_) => ErrorCategory::Permanent,
            Self::ParseError(_) => ErrorCategory::Permanent,
            Self::Http(e) if e.is_timeout() => ErrorCategory::Transient,
            Self::ProviderUnavailable(_) => ErrorCategory::Transient,
            Self::RateLimited { .. } => ErrorCategory::RateLimited,
            Self::CircuitBreakerOpen(_) => ErrorCategory::CircuitBreaker,
            _ => ErrorCategory::Transient,
        }
    }

    pub fn retry_after(&self) -> Option<std::time::Duration> {
        match self {
            PaperFetchError::RateLimited { retry_after, .. } => *retry_after,
            _ => None,
        }
    }
}
