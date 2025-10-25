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
    pub fn new(requests_per_second: u32) -> Result<Self, crate::error::PaperFetchError> {
        let quota = NonZeroU32::new(requests_per_second)
            .ok_or_else(|| {
                crate::error::PaperFetchError::InvalidInput(String::from(
                    "requests_per_second must be greater than 0",
                ))
            })
            .map(|n| Quota::per_second(n))?;

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
