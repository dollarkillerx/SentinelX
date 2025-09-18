use governor::{DefaultDirectRateLimiter, Quota, RateLimiter as GovernorRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct RateLimiter {
    limiter: Arc<DefaultDirectRateLimiter>,
}

impl RateLimiter {
    #[allow(dead_code)]
    pub fn new(bytes_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(bytes_per_second.max(1)).unwrap());
        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self { limiter }
    }

    pub async fn wait_for_capacity(&self, bytes: usize) {
        let permits = (bytes / 1024).max(1) as u32;
        for _ in 0..permits {
            self.limiter.until_ready().await;
        }
    }

    #[allow(dead_code)]
    pub fn try_consume(&self, bytes: usize) -> bool {
        let permits = (bytes / 1024).max(1) as u32;
        if let Some(n) = NonZeroU32::new(permits) {
            self.limiter.check_n(n).is_ok()
        } else {
            true
        }
    }
}