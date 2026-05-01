use std::collections::HashMap;

use crate::types::{ConductorName, HealthStatus};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub conductor: ConductorName,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub timestamp: DateTime<Utc>,
}

pub struct HealthManager {
    checks: HashMap<ConductorName, HealthCheck>,
}

impl HealthManager {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }

    pub fn update(&mut self, conductor: ConductorName, status: HealthStatus, message: Option<String>) {
        self.checks.insert(
            conductor.clone(),
            HealthCheck {
                conductor,
                status,
                message,
                timestamp: Utc::now(),
            },
        );
    }

    pub fn get_status(&self, conductor: &ConductorName) -> Option<&HealthCheck> {
        self.checks.get(conductor)
    }

    pub fn overall_status(&self) -> HealthStatus {
        if self.checks.is_empty() {
            return HealthStatus::Unhealthy;
        }
        if self.checks.values().any(|c| c.status == HealthStatus::Unhealthy) {
            return HealthStatus::Unhealthy;
        }
        if self.checks.values().any(|c| c.status == HealthStatus::Degraded) {
            return HealthStatus::Degraded;
        }
        HealthStatus::Healthy
    }
}

impl Default for HealthManager {
    fn default() -> Self {
        Self::new()
    }
}