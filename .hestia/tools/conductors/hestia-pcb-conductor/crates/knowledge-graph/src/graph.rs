//! Datasheet knowledge graph using petgraph

use crate::{KgEdge, KgNode};
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;

/// Datasheet knowledge graph.
pub struct DatasheetGraph {
    graph: DiGraph<KgNode, KgEdge>,
    index_map: HashMap<String, NodeIndex>,
}

impl DatasheetGraph {
    /// Create a new empty knowledge graph.
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            index_map: HashMap::new(),
        }
    }

    /// Add a node to the graph.
    pub fn add_node(&mut self, node: KgNode) -> NodeIndex {
        let id = node.id.clone();
        let idx = self.graph.add_node(node);
        self.index_map.insert(id, idx);
        idx
    }

    /// Add an edge between two nodes.
    pub fn add_edge(&mut self, edge: KgEdge) -> bool {
        let source_idx = self.index_map.get(&edge.source);
        let target_idx = self.index_map.get(&edge.target);
        match (source_idx, target_idx) {
            (Some(&s), Some(&t)) => {
                self.graph.add_edge(s, t, edge);
                true
            }
            _ => false,
        }
    }

    /// Get a node by its ID.
    pub fn get_node(&self, id: &str) -> Option<&KgNode> {
        self.index_map.get(id).map(|&idx| &self.graph[idx])
    }

    /// Find all nodes of a given type.
    pub fn find_by_type(&self, node_type: &crate::NodeType) -> Vec<&KgNode> {
        self.graph
            .node_indices()
            .filter_map(|idx| {
                let node = &self.graph[idx];
                if &node.node_type == node_type {
                    Some(node)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get the number of nodes.
    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Get the number of edges.
    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }
}

impl Default for DatasheetGraph {
    fn default() -> Self {
        Self::new()
    }
}