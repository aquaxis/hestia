//! Knowledge graph edges

use crate::EdgeType;
use serde::{Deserialize, Serialize};

/// An edge in the datasheet knowledge graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KgEdge {
    /// Source node ID.
    pub source: String,
    /// Target node ID.
    pub target: String,
    /// Edge type.
    pub edge_type: EdgeType,
    /// Edge weight (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weight: Option<f64>,
    /// Key-value metadata.
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, String>,
}

impl KgEdge {
    /// Create a new edge.
    pub fn new(source: &str, target: &str, edge_type: EdgeType) -> Self {
        Self {
            source: source.to_string(),
            target: target.to_string(),
            edge_type,
            weight: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Create an edge with a weight.
    pub fn with_weight(mut self, weight: f64) -> Self {
        self.weight = Some(weight);
        self
    }
}