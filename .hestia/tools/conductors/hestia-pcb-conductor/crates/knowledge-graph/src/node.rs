//! Knowledge graph nodes

use crate::NodeType;
use serde::{Deserialize, Serialize};

/// A node in the datasheet knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgNode {
    /// Unique identifier.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Node type.
    pub node_type: NodeType,
    /// Key-value properties.
    #[serde(default)]
    pub properties: std::collections::HashMap<String, String>,
}

impl KgNode {
    /// Create a new node.
    pub fn new(id: &str, label: &str, node_type: NodeType) -> Self {
        Self {
            id: id.to_string(),
            label: label.to_string(),
            node_type,
            properties: std::collections::HashMap::new(),
        }
    }

    /// Add a property to the node.
    pub fn with_property(mut self, key: &str, value: &str) -> Self {
        self.properties.insert(key.to_string(), value.to_string());
        self
    }

    /// Create a component node.
    pub fn component(id: &str, label: &str) -> Self {
        Self::new(id, label, NodeType::Component)
    }

    /// Create a parameter node.
    pub fn parameter(id: &str, label: &str) -> Self {
        Self::new(id, label, NodeType::Parameter)
    }
}