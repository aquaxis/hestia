use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::SkillError;

/// Metadata describing a skill.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillMeta {
    pub id: String,
    pub name: String,
    pub description: String,
}

/// A skill that can be discovered and executed by the conductor.
#[async_trait]
pub trait Skill: Send + Sync {
    /// Unique identifier for this skill.
    fn id(&self) -> &str;

    /// Human-readable name.
    fn name(&self) -> &str;

    /// Description of what this skill does.
    fn description(&self) -> &str;

    /// Execute the skill with the given parameters.
    /// Returns a JSON value on success or an error on failure.
    async fn execute(&self, params: &serde_json::Value) -> Result<serde_json::Value, SkillError>;

    /// Return the skill metadata.
    fn meta(&self) -> SkillMeta {
        SkillMeta {
            id: self.id().to_string(),
            name: self.name().to_string(),
            description: self.description().to_string(),
        }
    }
}
