use serde::{Deserialize, Serialize};
use crate::WorkflowError;

/// A single step within a pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStep {
    pub name: String,
    pub conductor: String,
    pub method: String,
    pub params: serde_json::Value,
}

/// A named pipeline of sequential steps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pipeline {
    pub name: String,
    pub steps: Vec<PipelineStep>,
}

impl Pipeline {
    /// Create a new empty pipeline with the given name.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            steps: Vec::new(),
        }
    }

    /// Add a step to the pipeline.
    pub fn add_step(&mut self, step: PipelineStep) {
        self.steps.push(step);
    }

    /// Run the pipeline sequentially, executing each step in order.
    /// Returns an error if any step fails.
    pub async fn run(&self) -> Result<Vec<serde_json::Value>, WorkflowError> {
        let mut results = Vec::with_capacity(self.steps.len());

        for (i, step) in self.steps.iter().enumerate() {
            tracing::info!(
                pipeline = %self.name,
                step = %step.name,
                conductor = %step.conductor,
                method = %step.method,
                "executing pipeline step {}/{}",
                i + 1,
                self.steps.len()
            );

            // TODO: Dispatch to conductor-sdk for actual execution.
            // For now we record a placeholder result.
            let result = serde_json::json!({
                "step": step.name,
                "conductor": step.conductor,
                "method": step.method,
                "status": "ok"
            });

            results.push(result);
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipeline_construction() {
        let mut pipeline = Pipeline::new("test-pipeline");
        pipeline.add_step(PipelineStep {
            name: "step-1".into(),
            conductor: "ai-conductor".into(),
            method: "generate".into(),
            params: serde_json::json!({}),
        });
        assert_eq!(pipeline.steps.len(), 1);
        assert_eq!(pipeline.name, "test-pipeline");
    }
}
