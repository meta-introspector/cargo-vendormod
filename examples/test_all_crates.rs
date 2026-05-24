use std::process::Command;
use std::path::PathBuf;
use std::fs;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing topological sorting on all crates in the repository...");
    
    // Create a temporary workspace that includes all crates
    let temp_workspace = "/tmp/all_crates_workspace";
    fs::create_dir_all(temp_workspace)?;
    
    // Copy our main Cargo.toml to the temp workspace
    fs::copy("Cargo.toml", format!("{}/Cargo.toml", temp_workspace))?;
    
    // Find all potential crate directories
    let crate_dirs = vec![
        "cargo-edit",
        "krates", 
        "zkperf",
        "krates_example",
        "solana_analyzer"
    ];
    
    // Create symlinks to include all crates in the workspace
    let mut workspace_members = Vec::new();
    
    for crate_dir in &crate_dirs {
        let source = PathBuf::from(crate_dir);
        let dest = PathBuf::from(temp_workspace).join(crate_dir);
        
        if source.exists() && source.join("Cargo.toml").exists() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(&source, &dest)?;
            
            #[cfg(windows)]
            std::os::windows::fs::symlink_dir(&source, &dest)?;
            
            workspace_members.push(format!("{}/Cargo.toml", crate_dir));
            println!("Added crate: {}", crate_dir);
        }
    }
    
    // Update the workspace Cargo.toml to include all members
    let mut cargo_toml = fs::read_to_string(format!("{}/Cargo.toml", temp_workspace))?;
    
    // Add workspace section if not present
    if !cargo_toml.contains("[workspace]") {
        cargo_toml.push_str("\n[workspace]\n");
    }
    
    // Add members
    cargo_toml.push_str("members = [\n");
    for member in &workspace_members {
        cargo_toml.push_str(&format!("    \"{}\",\n", member));
    }
    cargo_toml.push_str("]\n");
    
    fs::write(format!("{}/Cargo.toml", temp_workspace), cargo_toml)?;
    
    println!("Created workspace with {} members", workspace_members.len());
    
    // Build the dependency graph for the comprehensive workspace
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--", "global-graph", "build", temp_workspace, "--output-dir", "/tmp/all_crates_graph"])
        .current_dir(".")
        .output()?;
    
    if !output.status.success() {
        println!("Graph build output: {}", String::from_utf8_lossy(&output.stdout));
        println!("Graph build errors: {}", String::from_utf8_lossy(&output.stderr));
        return Err("Failed to build comprehensive graph".into());
    }
    
    println!("Comprehensive dependency graph built successfully!");
    
    // Read and analyze the comprehensive graph
    let graph_json = fs::read_to_string("/tmp/all_crates_graph/graph.json")?;
    let graph_data: serde_json::Value = serde_json::from_str(&graph_json)?;
    
    println!("Comprehensive Graph Metrics:");
    println!("- Nodes: {}", graph_data["nodes"].as_array().unwrap().len());
    println!("- Edges: {}", graph_data["edges"].as_array().unwrap().len());
    
    // Categorize nodes
    let mut workspace_nodes = Vec::new();
    let mut external_nodes = Vec::new();
    
    for node in graph_data["nodes"].as_array().unwrap() {
        let node_id = node["id"].as_str().unwrap();
        if node_id.starts_with("workspace:") {
            workspace_nodes.push(node_id.to_string());
        } else if node_id.starts_with("crate:") {
            external_nodes.push(node_id.to_string());
        }
    }
    
    println!("- Workspace members: {}", workspace_nodes.len());
    println!("- External dependencies: {}", external_nodes.len());
    
    println!("\n=== Workspace Members ===");
    for member in &workspace_nodes {
        println!("- {}", member);
    }
    
    println!("\n=== Top External Dependencies ===");
    for dep in external_nodes.iter().take(10) {
        println!("- {}", dep);
    }
    
    if external_nodes.len() > 10 {
        println!("... and {} more", external_nodes.len() - 10);
    }
    
    // Test the process-crates command on the comprehensive workspace
    println!("\n=== Testing Layered Processing ===");
    let process_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "process-crates", temp_workspace, "--output-dir", "/tmp/all_crates_process", "--dry-run"])
        .current_dir(".")
        .output()?;
    
    println!("Process output: {}", String::from_utf8_lossy(&process_output.stdout));
    if !process_output.stderr.is_empty() {
        println!("Process errors: {}", String::from_utf8_lossy(&process_output.stderr));
    }
    
    Ok(())
}