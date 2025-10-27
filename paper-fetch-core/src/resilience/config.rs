use std::time::Duration;

/// Configuration for resilience patterns
#[derive(Debug, Clone)]
pub struct ResilienceConfig {
    /// Requests per second for rate limiting
    pub rate_limit_rps: u32,

    /// Circuit breaker: initial backoff duration
    pub cb_initial_backoff: Duration,

    /// Circuit breaker: maximum backoff duration
    pub cb_max_backoff: Duration,

    /// Circuit breaker: consecutive failures before opening
    pub cb_failure_threshold: u32,

    /// Retry: maximum number of attempts
    pub retry_max_attempts: usize,

    /// Retry: minimum number of attempts
    pub retry_min_backoff: Duration,

    /// Retry: maximum backoff duration
    pub retry_max_backoff: Duration,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            rate_limit_rps: 1,
            cb_initial_backoff: Duration::from_secs(10),
            cb_max_backoff: Duration::from_secs(60),
            cb_failure_threshold: 3,
            retry_max_attempts: 5,
            retry_min_backoff: Duration::from_millis(100),
            retry_max_backoff: Duration::from_secs(30),
        }
    }
}
