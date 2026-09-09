//! Bounded job queue - chống анти-задержки (см. ARCHITECTURE.md Policy).
//! Semaphore 100 = MaxHandlerConcurrency, channel 256 = backpressure.

use std::sync::Arc;
use tokio::sync::{Semaphore, mpsc};
use tracing::warn;

pub struct JobQueue {
    tx: mpsc::Sender<Job>,
    sem: Arc<Semaphore>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl JobQueue {
    pub fn new(cap: usize, concurrency: usize) -> (Self, mpsc::Receiver<Job>) {
        let (tx, rx) = mpsc::channel(cap);
        let sem = Arc::new(Semaphore::new(concurrency));
        (Self { tx, sem }, rx)
    }

    /// Submit without blocking gateway. Returns error only if queue full (backpressure → 429).
    pub fn try_submit<F>(&self, job: F) -> Result<(), &'static str>
    where
        F: FnOnce() + Send + 'static,
    {
        self.tx.try_send(Box::new(job)).map_err(|_| {
            warn!("job queue full - backpressure, drop job");
            "queue full"
        })
    }

    pub fn semaphore(&self) -> Arc<Semaphore> {
        self.sem.clone()
    }
}
