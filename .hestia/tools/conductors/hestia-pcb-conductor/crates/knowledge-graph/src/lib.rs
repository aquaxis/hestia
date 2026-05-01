//! knowledge-graph -- Datasheet knowledge graph: nodes, edges

pub mod graph;
pub mod node;
pub mod edge;

pub use graph::DatasheetGraph;
pub use node::KgNode;
pub use edge::KgEdge;

use serde::{Deserialize, Serialize};

/// Knowledge graph node types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeType {
    Component,
    Parameter,
    Interface,
    Manufacturer,
    Application,
}

/// Knowledge graph edge types.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeType {
    HasParameter,
    ManufacturedBy,
    CompatibleWith,
    RecommendedFor,
    SimilarTo,
    Requires,
}