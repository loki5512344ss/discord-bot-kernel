use async_trait::async_trait;

/// Minimal lifecycle - replaces Java Lifecycle interface (core/Lifecycle.java:6).
/// Used by `db::pool` and `core::bot` for start/shutdown ordering.
#[async_trait]
pub trait Lifecycle: Send + Sync {
    /// Starts the component (e.g., connect DB, register commands).
    async fn start(&self) -> anyhow::Result<()>;
    /// Gracefully shuts down (e.g., close pool, shutdown shards).
    async fn shutdown(&self) -> anyhow::Result<()>;
}

// Ensure the trait is used (avoid dead_code) - implemented for `()`.
#[async_trait]
impl Lifecycle for () {
    async fn start(&self) -> anyhow::Result<()> {
        Ok(())
    }
    async fn shutdown(&self) -> anyhow::Result<()> {
        Ok(())
    }
}
