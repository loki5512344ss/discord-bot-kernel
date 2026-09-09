//! POLICY - анти-задержки (ARCHITECTURE.md).
//! Запрещено в `src/feature/*`: `tokio::time::sleep` / `std::thread::sleep` без `// SAFETY:`.
//! Разрешённые задержки - только с обоснованием:
//!   // SAFETY: retry backoff 2s + jitter for 429 Retry-After
//!   tokio::time::sleep(Duration::from_millis(2000 + rand 0..500)).await;
//! CI: grep -R 'sleep' src/feature --include='*.rs' | grep -v 'SAFETY:' && exit 1
//!
//! Для тяжёлых задач: `ctx.defer().await?` + `tokio::spawn` + `JobQueue` (semaphore 100).

pub const MAX_HANDLER_CONCURRENCY: usize = 100;
pub const QUEUE_CAPACITY: usize = 256;
pub const RATE_LIMIT_WINDOW_SECS: u64 = 5;
