use std::process::Command;
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing topological sorting on our own dependency graph...");
    
    // Run the global-graph build command
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "global-graph", "build", ".", "--output-dir", "/tmp/test_topo"])
        .output()?;
    
    if !output.status.success() {
        return Err(format!("Failed to build graph: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    
    println!("Graph built successfully!");
    
    // Read the generated graph
    let graph_json = std::fs::read_to_string("/tmp/test_topo/graph.json")?;
    let graph_data: serde_json::Value = serde_json::from_str(&graph_json)?;
    
    println!("Nodes: {}", graph_data["nodes"].as_array().unwrap().len());
    println!("Edges: {}", graph_data["edges"].as_array().unwrap().len());
    
    // Extract node IDs for topological analysis
    let nodes: Vec<String> = graph_data["nodes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|n| n["id"].as_str().unwrap().to_string())
        .collect();
    
    let edges: Vec<(String, String)> = graph_data["edges"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| (
            e["from"].as_str().unwrap().to_string(),
            e["to"].as_str().unwrap().to_string()
        ))
        .collect();
    
    println!("\n=== Dependency Relationships ===");
    for (from, to) in &edges {
        println!("{} -> {}", from, to);
    }
    
    println!("\n=== Workspace Members ===");
    for node in &nodes {
        if node.starts_with("workspace:") {
            println!("- {}", node);
        }
    }
    
    println!("\n=== External Dependencies ===");
    for node in &nodes {
        if node.starts_with("crate:") {
            println!("- {}", node);
        }
    }
    
    Ok(())
}