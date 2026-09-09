//! Prefix dispatcher - virtual-thread analogue (Java EventDispatcher:96).
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
pub struct RateLimiter {
    inner: Arc<Mutex<HashMap<String, Instant>>>,
    window: Duration,
}

impl RateLimiter {
    pub fn new(window_secs: u64) -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
            window: Duration::from_secs(window_secs),
        }
    }

    pub async fn check(&self, user_id: &str) -> Option<Duration> {
        let mut map = self.inner.lock().await;
        let now = Instant::now();
        if let Some(last) = map.get(user_id) {
            let elapsed = now.duration_since(*last);
            if elapsed < self.window {
                return Some(self.window - elapsed);
            }
        }
        map.insert(user_id.to_owned(), now);
        None
    }
}
