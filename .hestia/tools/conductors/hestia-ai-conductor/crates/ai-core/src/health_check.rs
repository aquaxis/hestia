//! Periodic conductor health checking

use conductor_sdk::agent::{ConductorId, ConductorInfo, ConductorStatus};
use tracing::{debug, warn};

/// A single conductor's health check result.
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub conductor_id: ConductorId,
    pub healthy: bool,
    pub status: ConductorStatus,
    pub latency_ms: Option<u64>,
}

/// Performs health checks across all registered conductors.
#[derive(Debug)]
pub struct HealthChecker {
    /// Interval in seconds between automatic health-check rounds.
    pub interval_secs: u64,
}

impl HealthChecker {
    /// Create a new health checker with the given check interval.
    pub fn new(interval_secs: u64) -> Self {
        Self { interval_secs }
    }

    /// Check the health of every conductor in the provided list.
    ///
    /// For now the implementation returns a simple `healthy = true` for every
    /// conductor whose status is `Online` or `Degraded`, and `false` otherwise.
    /// A real implementation would issue a ping/heartbeat request and measure
    /// latency.
    pub fn check_all(&self, conductors: &[ConductorInfo]) -> Vec<HealthStatus> {
        conductors
            .iter()
            .map(|info| {
                let healthy = matches!(info.status, ConductorStatus::Online | ConductorStatus::Degraded);
                if !healthy {
                    warn!(conductor = %info.id, status = %info.status, "conductor unhealthy");
                } else {
                    debug!(conductor = %info.id, status = %info.status, "conductor healthy");
                }
                HealthStatus {
                    conductor_id: info.id,
                    healthy,
                    status: info.status,
                    latency_ms: None,
                }
            })
            .collect()
    }
}

impl Default for HealthChecker {
    fn default() -> Self {
        Self::new(30)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_all_online() {
        let checker = HealthChecker::new(10);
        let conductors = vec![ConductorInfo {
            id: ConductorId::Ai,
            status: ConductorStatus::Online,
            version: "0.1.0".into(),
            uptime_secs: 100,
        }];
        let results = checker.check_all(&conductors);
        assert_eq!(results.len(), 1);
        assert!(results[0].healthy);
    }

    #[test]
    fn check_all_offline() {
        let checker = HealthChecker::new(10);
        let conductors = vec![ConductorInfo {
            id: ConductorId::Rtl,
            status: ConductorStatus::Offline,
            version: "0.1.0".into(),
            uptime_secs: 0,
        }];
        let results = checker.check_all(&conductors);
        assert_eq!(results.len(), 1);
        assert!(!results[0].healthy);
    }
}