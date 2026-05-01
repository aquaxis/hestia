use petgraph::graph::{Graph, NodeIndex};
use petgraph::Direction;
use std::collections::{HashMap, VecDeque};
use crate::WorkflowError;

/// A directed acyclic graph for representing workflow dependencies.
pub struct WorkflowDag {
    graph: Graph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl WorkflowDag {
    /// Create a new empty workflow DAG.
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
        }
    }

    /// Add a named node to the DAG. Returns the node index.
    /// If the node already exists, returns the existing index.
    pub fn add_node(&mut self, name: &str) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(name) {
            return idx;
        }
        let idx = self.graph.add_node(name.to_string());
        self.node_map.insert(name.to_string(), idx);
        idx
    }

    /// Add a directed edge from `from` to `to`.
    /// Returns an error if either node does not exist.
    pub fn add_edge(&mut self, from: &str, to: &str) -> Result<(), WorkflowError> {
        let from_idx = self
            .node_map
            .get(from)
            .copied()
            .ok_or_else(|| WorkflowError::NodeNotFound(from.to_string()))?;
        let to_idx = self
            .node_map
            .get(to)
            .copied()
            .ok_or_else(|| WorkflowError::NodeNotFound(to.to_string()))?;
        self.graph.add_edge(from_idx, to_idx, ());
        Ok(())
    }

    /// Perform a topological sort using Kahn's algorithm.
    /// Returns nodes in execution order (dependencies first).
    /// Returns an error if a cycle is detected.
    pub fn topological_sort(&self) -> Result<Vec<String>, WorkflowError> {
        let node_count = self.graph.node_count();
        if node_count == 0 {
            return Ok(Vec::new());
        }

        // Compute in-degrees
        let mut in_degree: Vec<usize> = vec![0; node_count];
        for idx in self.graph.node_indices() {
            in_degree[idx.index()] = self
                .graph
                .neighbors_directed(idx, Direction::Incoming)
                .count();
        }

        // Seed queue with all zero in-degree nodes
        let mut queue: VecDeque<NodeIndex> = self
            .graph
            .node_indices()
            .filter(|idx| in_degree[idx.index()] == 0)
            .collect();

        let mut sorted = Vec::with_capacity(node_count);
        let mut visited = 0usize;

        while let Some(node) = queue.pop_front() {
            sorted.push(self.graph[node].clone());
            visited += 1;

            for neighbor in self.graph.neighbors_directed(node, Direction::Outgoing) {
                in_degree[neighbor.index()] -= 1;
                if in_degree[neighbor.index()] == 0 {
                    queue.push_back(neighbor);
                }
            }
        }

        if visited != node_count {
            return Err(WorkflowError::CycleDetected);
        }

        Ok(sorted)
    }

    /// Validate that the DAG has no cycles.
    pub fn validate(&self) -> Result<(), WorkflowError> {
        self.topological_sort()?;
        Ok(())
    }
}

impl Default for WorkflowDag {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_sort() {
        let mut dag = WorkflowDag::new();
        dag.add_node("a");
        dag.add_node("b");
        dag.add_node("c");
        dag.add_edge("a", "b").unwrap();
        dag.add_edge("b", "c").unwrap();
        let order = dag.topological_sort().unwrap();
        assert_eq!(order, vec!["a", "b", "c"]);
    }

    #[test]
    fn test_cycle_detection() {
        let mut dag = WorkflowDag::new();
        dag.add_node("x");
        dag.add_node("y");
        dag.add_edge("x", "y").unwrap();
        dag.add_edge("y", "x").unwrap();
        assert!(dag.validate().is_err());
    }
}
