use serde::{Deserialize, Serialize};
use crate::UpgradeError;

/// Stages of the upgrade pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeStage {
    Watcher,
    Probe,
    Patcher,
    Validator,
    HumanReview,
    Complete,
}

impl std::fmt::Display for UpgradeStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeStage::Watcher => write!(f, "Watcher"),
            UpgradeStage::Probe => write!(f, "Probe"),
            UpgradeStage::Patcher => write!(f, "Patcher"),
            UpgradeStage::Validator => write!(f, "Validator"),
            UpgradeStage::HumanReview => write!(f, "HumanReview"),
            UpgradeStage::Complete => write!(f, "Complete"),
        }
    }
}

/// The full sequence of upgrade stages.
const PIPELINE_STAGES: [UpgradeStage; 6] = [
    UpgradeStage::Watcher,
    UpgradeStage::Probe,
    UpgradeStage::Patcher,
    UpgradeStage::Validator,
    UpgradeStage::HumanReview,
    UpgradeStage::Complete,
];

/// Result of a single pipeline stage execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub stage: UpgradeStage,
    pub success: bool,
    pub message: String,
}

/// The upgrade pipeline that orchestrates the staged rollout.
#[derive(Debug)]
pub struct UpgradePipeline {
    current_stage: usize,
    results: Vec<StageResult>,
}

impl UpgradePipeline {
    /// Create a new upgrade pipeline starting at the first stage.
    pub fn new() -> Self {
        Self {
            current_stage: 0,
            results: Vec::new(),
        }
    }

    /// Get the current stage.
    pub fn current_stage(&self) -> UpgradeStage {
        PIPELINE_STAGES[self.current_stage]
    }

    /// Advance the pipeline by executing the current stage and moving to the next.
    /// Returns the result of the executed stage.
    pub async fn advance(&mut self) -> Result<StageResult, UpgradeError> {
        if self.current_stage >= PIPELINE_STAGES.len() {
            return Err(UpgradeError::PipelineError(
                "pipeline already completed".into(),
            ));
        }

        let stage = PIPELINE_STAGES[self.current_stage];
        tracing::info!(stage = %stage, "executing upgrade pipeline stage");

        // TODO: Dispatch to actual stage handlers via conductor-sdk.
        // Each stage will perform its specific work:
        //   Watcher    -> monitor for new versions
        //   Probe      -> validate compatibility
        //   Patcher    -> apply changes
        //   Validator  -> verify correctness
        //   HumanReview -> pause for human approval
        //   Complete   -> finalize the upgrade

        let result = StageResult {
            stage,
            success: true,
            message: format!("{} stage completed successfully", stage),
        };

        self.current_stage += 1;
        self.results.push(result.clone());
        Ok(result)
    }

    /// Run the entire pipeline from the current stage to completion.
    pub async fn run(&mut self) -> Result<Vec<StageResult>, UpgradeError> {
        let mut all_results = Vec::new();
        while self.current_stage < PIPELINE_STAGES.len() {
            let result = self.advance().await?;
            all_results.push(result);
        }
        Ok(all_results)
    }

    /// Get all stage results collected so far.
    pub fn results(&self) -> &[StageResult] {
        &self.results
    }

    /// Check if the pipeline has reached the Complete stage.
    pub fn is_complete(&self) -> bool {
        self.current_stage >= PIPELINE_STAGES.len()
    }
}

impl Default for UpgradePipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pipeline_advance() {
        let mut pipeline = UpgradePipeline::new();
        assert_eq!(pipeline.current_stage(), UpgradeStage::Watcher);

        let result = pipeline.advance().await.unwrap();
        assert_eq!(result.stage, UpgradeStage::Watcher);
        assert!(result.success);
        assert_eq!(pipeline.current_stage(), UpgradeStage::Probe);
    }

    #[tokio::test]
    async fn test_pipeline_full_run() {
        let mut pipeline = UpgradePipeline::new();
        let results = pipeline.run().await.unwrap();
        assert_eq!(results.len(), 6);
        assert!(pipeline.is_complete());
    }
}
