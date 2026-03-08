use super::config::ResilienceConfig;
use governor::{DefaultDirectRateLimiter, Quota, RateLimiter};
use std::collections::HashMap;
use std::num::NonZeroU32;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ProviderRateLimiter {
    limiters: Arc<RwLock<HashMap<String, Arc<DefaultDirectRateLimiter>>>>,
    default_quota: Quota,
}

impl ProviderRateLimiter {
    pub fn new(config: &ResilienceConfig) -> Result<Self, crate::error::PaperFetchError> {
        let quota = NonZeroU32::new(config.rate_limit_rps)
            .ok_or_else(|| {
                crate::error::PaperFetchError::InvalidInput(String::from(
                    "requests_per_second must be greater than 0",
                ))
            })
            .map(Quota::per_second)?;

        Ok(Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
            default_quota: quota,
        })
    }

    pub async fn acquire(&self, provider: &str) {
        let mut limiters = self.limiters.write().await;
        let limiter = limiters
            .entry(String::from(provider))
            .or_insert_with(|| Arc::new(RateLimiter::direct(self.default_quota)));

        limiter.until_ready().await;
    }
}
