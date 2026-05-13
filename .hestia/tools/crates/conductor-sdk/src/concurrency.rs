//! Phase 126 — Hierarchical semaphore for sub-agent concurrency + acquire timeout control.
//!
//! `ConductorLimiter` is a thin helper that wraps tokio::sync::Semaphore with a timeout.
//! It is used commonly across three levels: ai-conductor / each domain conductor / AgentManager.
//! When the timeout elapses, it returns `ConcurrencyError::AcquireTimeout` to detect and
//! resolve deadlocks caused by exceeding the cap.

use std::sync::Arc;
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tokio::time::timeout;

#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    #[error("acquire timeout after {0}s")]
    AcquireTimeout(u64),
    #[error("semaphore closed")]
    Closed,
}

#[derive(Clone, Debug)]
pub struct ConductorLimiter {
    sem: Arc<Semaphore>,
    timeout_secs: u64,
    capacity: usize,
}

impl ConductorLimiter {
    pub fn new(max: usize, timeout_secs: u64) -> Self {
        let capacity = max.max(1);
        Self {
            sem: Arc::new(Semaphore::new(capacity)),
            timeout_secs,
            capacity,
        }
    }

    /// Build with env-driven defaults (for per-conductor use).
    /// - `HESTIA_PER_CONDUCTOR_MAX` (default 4)
    /// - `HESTIA_ACQUIRE_TIMEOUT_SECS` (default 600)
    pub fn from_env() -> Self {
        let max = std::env::var("HESTIA_PER_CONDUCTOR_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(4);
        let to = std::env::var("HESTIA_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);
        Self::new(max, to)
    }

    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, ConcurrencyError> {
        match timeout(
            Duration::from_secs(self.timeout_secs),
            self.sem.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => Ok(permit),
            Ok(Err(_)) => Err(ConcurrencyError::Closed),
            Err(_) => Err(ConcurrencyError::AcquireTimeout(self.timeout_secs)),
        }
    }

    pub async fn run_with_slot<F, Fut, T>(&self, f: F) -> Result<T, ConcurrencyError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = T>,
    {
        let _permit = self.acquire().await?;
        Ok(f().await)
    }

    pub fn available(&self) -> usize {
        self.sem.available_permits()
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn limiter_caps_concurrency() {
        let lim = ConductorLimiter::new(2, 60);
        let p1 = lim.acquire().await.unwrap();
        let p2 = lim.acquire().await.unwrap();
        assert_eq!(lim.available(), 0);
        drop(p1);
        let _p3 = lim.acquire().await.unwrap();
        drop(p2);
    }

    #[tokio::test]
    async fn limiter_times_out() {
        let lim = ConductorLimiter::new(1, 1);
        let _p = lim.acquire().await.unwrap();
        let r = lim.acquire().await;
        assert!(matches!(r, Err(ConcurrencyError::AcquireTimeout(_))));
    }

    #[tokio::test]
    async fn run_with_slot_executes_and_releases() {
        let lim = ConductorLimiter::new(1, 5);
        let v = lim.run_with_slot(|| async { 42 }).await.unwrap();
        assert_eq!(v, 42);
        assert_eq!(lim.available(), 1);
    }

    #[tokio::test]
    async fn from_env_uses_default_when_unset() {
        std::env::remove_var("HESTIA_PER_CONDUCTOR_MAX");
        std::env::remove_var("HESTIA_ACQUIRE_TIMEOUT_SECS");
        let lim = ConductorLimiter::from_env();
        assert_eq!(lim.capacity(), 4);
        assert_eq!(lim.timeout_secs(), 600);
    }
}