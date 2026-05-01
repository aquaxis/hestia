//! Agent-level health checking

use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

/// Result of a single agent health check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentHealth {
    pub agent_id: String,
    pub healthy: bool,
    pub detail: Option<String>,
}

/// Performs health checks on individual agents.
#[derive(Debug)]
pub struct AgentHealthChecker {
    /// Timeout in seconds for each individual health-check probe.
    pub timeout_secs: u64,
}

impl AgentHealthChecker {
    /// Create a new health checker with the given per-agent timeout.
    pub fn new(timeout_secs: u64) -> Self {
        Self { timeout_secs }
    }

    /// Check the health of a single agent.
    ///
    /// In a real implementation this would issue a heartbeat or readiness
    /// probe.  For now it accepts the agent's self-reported status string
    /// and returns healthy when it is `"running"`.
    pub fn check_agent(&self, agent_id: &str, status: &str) -> AgentHealth {
        let healthy = status == "running";
        if healthy {
            debug!(agent = %agent_id, "agent health check passed");
        } else {
            warn!(agent = %agent_id, status = %status, "agent health check failed");
        }
        AgentHealth {
            agent_id: agent_id.to_string(),
            healthy,
            detail: if healthy {
                None
            } else {
                Some(format!("agent status is {status}"))
            },
        }
    }
}

impl Default for AgentHealthChecker {
    fn default() -> Self {
        Self::new(10)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn running_agent_is_healthy() {
        let checker = AgentHealthChecker::new(5);
        let result = checker.check_agent("a1", "running");
        assert!(result.healthy);
        assert!(result.detail.is_none());
    }

    #[test]
    fn stopped_agent_is_unhealthy() {
        let checker = AgentHealthChecker::new(5);
        let result = checker.check_agent("a1", "stopped");
        assert!(!result.healthy);
        assert!(result.detail.is_some());
    }
}