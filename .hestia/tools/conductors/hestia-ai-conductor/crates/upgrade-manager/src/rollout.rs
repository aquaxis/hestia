use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::UpgradeError;

/// Status of an ongoing rollout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RolloutStatus {
    Pending,
    InProgress { percent: u8 },
    Completed,
    Failed(String),
}

/// Manages version rollouts with progress tracking.
#[derive(Debug)]
pub struct RolloutManager {
    status: RolloutStatus,
    started_at: Option<DateTime<Utc>>,
}

impl RolloutManager {
    /// Create a new rollout manager in the Pending state.
    pub fn new() -> Self {
        Self {
            status: RolloutStatus::Pending,
            started_at: None,
        }
    }

    /// Start a rollout. Transitions from Pending to InProgress at 0%.
    pub fn start_rollout(&mut self) -> Result<(), UpgradeError> {
        match &self.status {
            RolloutStatus::Pending => {
                self.started_at = Some(Utc::now());
                self.status = RolloutStatus::InProgress { percent: 0 };
                tracing::info!("rollout started");
                Ok(())
            }
            RolloutStatus::InProgress { .. } => Err(UpgradeError::RolloutFailed(
                "rollout already in progress".into(),
            )),
            RolloutStatus::Completed => Err(UpgradeError::RolloutFailed(
                "rollout already completed".into(),
            )),
            RolloutStatus::Failed(_) => Err(UpgradeError::RolloutFailed(
                "previous rollout failed; reset first".into(),
            )),
        }
    }

    /// Advance the rollout to a given percentage.
    pub fn advance(&mut self, percent: u8) -> Result<(), UpgradeError> {
        match &self.status {
            RolloutStatus::InProgress { .. } => {
                if percent >= 100 {
                    self.status = RolloutStatus::Completed;
                    tracing::info!("rollout completed");
                } else {
                    self.status = RolloutStatus::InProgress { percent };
                    tracing::info!(percent, "rollout advanced");
                }
                Ok(())
            }
            _ => Err(UpgradeError::RolloutFailed(
                "no rollout in progress".into(),
            )),
        }
    }

    /// Mark the rollout as failed.
    pub fn fail(&mut self, reason: &str) {
        self.status = RolloutStatus::Failed(reason.to_string());
        tracing::error!(reason, "rollout failed");
    }

    /// Get the current rollout status.
    pub fn get_status(&self) -> &RolloutStatus {
        &self.status
    }

    /// Get the timestamp when the rollout was started, if any.
    pub fn started_at(&self) -> Option<DateTime<Utc>> {
        self.started_at
    }
}

impl Default for RolloutManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rollout_lifecycle() {
        let mut mgr = RolloutManager::new();
        assert!(matches!(mgr.get_status(), RolloutStatus::Pending));

        mgr.start_rollout().unwrap();
        assert!(matches!(mgr.get_status(), RolloutStatus::InProgress { percent: 0 }));

        mgr.advance(50).unwrap();
        assert!(matches!(mgr.get_status(), RolloutStatus::InProgress { percent: 50 }));

        mgr.advance(100).unwrap();
        assert!(matches!(mgr.get_status(), RolloutStatus::Completed));
    }

    #[test]
    fn test_rollout_failure() {
        let mut mgr = RolloutManager::new();
        mgr.start_rollout().unwrap();
        mgr.fail("something broke");
        assert!(matches!(mgr.get_status(), RolloutStatus::Failed(_)));
    }
}
