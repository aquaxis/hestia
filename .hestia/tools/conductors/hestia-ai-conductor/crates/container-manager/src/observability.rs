//! Container observability — emit metrics for running containers

use chrono::Utc;
use tracing::info;

/// Metric snapshot for a container.
#[derive(Debug, Clone)]
pub struct ContainerMetrics {
    pub name: String,
    pub cpu_usage_percent: f64,
    pub memory_usage_mb: f64,
    pub timestamp: String,
}

pub struct ContainerObservability;

impl ContainerObservability {
    pub fn new() -> Self {
        Self
    }

    /// Emit a metrics snapshot for the given container name.
    pub fn emit_metrics(&self, container_name: &str) -> ContainerMetrics {
        info!("emitting metrics for container={container_name}");
        // Minimal implementation — in production this would query podman stats
        ContainerMetrics {
            name: container_name.to_string(),
            cpu_usage_percent: 0.0,
            memory_usage_mb: 0.0,
            timestamp: Utc::now().to_rfc3339(),
        }
    }
}

impl Default for ContainerObservability {
    fn default() -> Self {
        Self::new()
    }
}