//! Shared index test for the DMZ tag 42 graph
//!
//! This creates a unified test index that can be used across Rust, Python, Go, JS

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DMZTag42Node {
    pub id: String,
    pub label: String,
    pub node_type: String,
    pub domain: String,
    pub count: Option<usize>,
    pub file: Option<String>,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DMZTag42Edge {
    pub from: String,
    pub to: String,
    pub edge_type: String,
    pub count: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DMZTag42Graph {
    pub nodes: Vec<DMZTag42Node>,
    pub edges: Vec<DMZTag42Edge>,
    pub metadata: HashMap<String, serde_json::Value>,
}

pub fn compute_transition_matrix(graph: &DMZTag42Graph) -> HashMap<(String, String), f64> {
    let mut transition_counts: HashMap<(String, String), usize> = HashMap::new();
    let mut outgoing_totals: HashMap<String, usize> = HashMap::new();

    // Count transitions
    for edge in &graph.edges {
        let key = (edge.from.clone(), edge.to.clone());
        let count = edge.count.unwrap_or(1);
        *transition_counts.entry(key).or_insert(0) += count;
        *outgoing_totals.entry(edge.from.clone()).or_insert(0) += count;
    }

    // Convert to probabilities
    let mut matrix: HashMap<(String, String), f64> = HashMap::new();
    for ((from, to), count) in transition_counts {
        let total = outgoing_totals.get(&from).unwrap_or(&1);
        matrix.insert((from, to), count as f64 / *total as f64);
    }

    matrix
}

pub fn find_all_paths_to_target(graph: &DMZTag42Graph, target: &str, max_depth: usize) -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut visited = HashSet::new();

    // Build adjacency list
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for edge in &graph.edges {
        adj.entry(edge.from.clone()).or_default().push(edge.to.clone());
    }

    fn dfs(
        current: &str,
        target: &str,
        path: &mut Vec<String>,
        visited: &mut HashSet<String>,
        adj: &HashMap<String, Vec<String>>,
        paths: &mut Vec<Vec<String>>,
        depth: usize,
    ) {
        if depth > max_depth {
            return;
        }
        path.push(current.to_string());
        visited.insert(current.to_string());

        if current == target {
            paths.push(path.clone());
        } else if let Some(neighbors) = adj.get(current) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    dfs(neighbor, target, path, visited, adj, paths, depth + 1);
                }
            }
        }

        path.pop();
        visited.remove(current);
    }

    // Find all nodes that have the target as reachable
    for node in &graph.nodes {
        if node.id != target && !visited.contains(&node.id) {
            dfs(&node.id, target, &mut Vec::new(), &mut visited, &adj, &mut paths, 0);
        }
    }

    paths
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_dmZ_graph() {
        // Will load the DMZ graph JSON and verify structure
        println!("DMZ Tag 42 graph test placeholder");
    }
}