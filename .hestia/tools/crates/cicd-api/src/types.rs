use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Backend {
    GithubActions,
    GitlabCi,
    LocalPipeline,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StageCondition {
    Always,
    OnSuccess,
    OnFailure,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineJob {
    pub name: String,
    pub commands: Vec<String>,
    pub environment: std::collections::HashMap<String, String>,
    pub artifacts: Vec<String>,
    pub retry_count: u32,
    pub retry_interval_secs: u64,
    pub timeout_secs: u64,
    pub cache_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineStage {
    pub name: String,
    pub jobs: Vec<PipelineJob>,
    pub parallel: bool,
    pub condition: StageCondition,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PipelineDefinition {
    pub name: String,
    pub backend: Backend,
    pub stages: Vec<PipelineStage>,
    pub artifact_retention_days: u32,
}