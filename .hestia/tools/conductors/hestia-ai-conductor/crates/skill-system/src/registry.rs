use std::collections::HashMap;
use crate::skill::{Skill, SkillMeta};
use crate::SkillError;

/// A registry that stores and retrieves skills by their ID.
pub struct SkillRegistry {
    skills: HashMap<String, Box<dyn Skill>>,
}

impl SkillRegistry {
    /// Create a new empty skill registry.
    pub fn new() -> Self {
        Self {
            skills: HashMap::new(),
        }
    }

    /// Register a skill. Returns an error if a skill with the same ID already exists.
    pub fn register(&mut self, skill: Box<dyn Skill>) -> Result<(), SkillError> {
        let id = skill.id().to_string();
        if self.skills.contains_key(&id) {
            return Err(SkillError::AlreadyRegistered(id));
        }
        self.skills.insert(id, skill);
        Ok(())
    }

    /// Unregister a skill by ID. Returns true if the skill was present.
    pub fn unregister(&mut self, id: &str) -> bool {
        self.skills.remove(id).is_some()
    }

    /// Look up a skill by ID.
    pub fn get(&self, id: &str) -> Option<&dyn Skill> {
        self.skills.get(id).map(|s| s.as_ref())
    }

    /// List all registered skill metadata.
    pub fn list(&self) -> Vec<SkillMeta> {
        self.skills.values().map(|s| s.meta()).collect()
    }
}

impl Default for SkillRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;

    struct DummySkill {
        id: String,
        name: String,
        description: String,
    }

    #[async_trait]
    impl Skill for DummySkill {
        fn id(&self) -> &str { &self.id }
        fn name(&self) -> &str { &self.name }
        fn description(&self) -> &str { &self.description }
        async fn execute(&self, _params: &serde_json::Value) -> Result<serde_json::Value, SkillError> {
            Ok(serde_json::json!({"result": "ok"}))
        }
    }

    #[tokio::test]
    async fn test_register_and_get() {
        let mut registry = SkillRegistry::new();
        let skill = DummySkill {
            id: "test-skill".into(),
            name: "Test".into(),
            description: "A test skill".into(),
        };
        registry.register(Box::new(skill)).unwrap();
        assert!(registry.get("test-skill").is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[tokio::test]
    async fn test_unregister() {
        let mut registry = SkillRegistry::new();
        let skill = DummySkill {
            id: "rm-skill".into(),
            name: "Remove".into(),
            description: "To be removed".into(),
        };
        registry.register(Box::new(skill)).unwrap();
        assert!(registry.unregister("rm-skill"));
        assert!(registry.get("rm-skill").is_none());
    }
}
