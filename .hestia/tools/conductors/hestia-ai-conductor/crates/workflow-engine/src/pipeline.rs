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
    ///
    /// Each step is dispatched to the specified conductor via the provided
    /// closure. The closure receives `(conductor_name, method, params)` and
    /// returns the response from the conductor.
    pub async fn run<F, Fut>(
        &self,
        dispatch_fn: F,
    ) -> Result<Vec<serde_json::Value>, WorkflowError>
    where
        F: Fn(String, String, serde_json::Value) -> Fut,
        Fut: std::future::Future<Output = Result<serde_json::Value, String>>,
    {
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

            match dispatch_fn(
                step.conductor.clone(),
                step.method.clone(),
                step.params.clone(),
            )
            .await
            {
                Ok(response) => {
                    results.push(serde_json::json!({
                        "step": step.name,
                        "conductor": step.conductor,
                        "method": step.method,
                        "status": "ok",
                        "response": response,
                    }));
                }
                Err(e) => {
                    tracing::error!(
                        pipeline = %self.name,
                        step = %step.name,
                        error = %e,
                        "pipeline step failed"
                    );
                    results.push(serde_json::json!({
                        "step": step.name,
                        "conductor": step.conductor,
                        "method": step.method,
                        "status": "error",
                        "error": e,
                    }));
                    return Err(WorkflowError::PipelineFailed(format!(
                        "step '{}' on conductor '{}' failed: {}",
                        step.name, step.conductor, e
                    )));
                }
            }
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