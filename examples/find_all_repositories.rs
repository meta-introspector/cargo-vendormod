use std::process::Command;
use std::collections::HashMap;
use serde_json::Value;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Finding all git repositories in dependency metadata...");
    
    // Get comprehensive cargo metadata
    let metadata_output = Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .output()?;
    
    if !metadata_output.status.success() {
        return Err("Failed to get cargo metadata".into());
    }
    
    let metadata: Value = serde_json::from_slice(&metadata_output.stdout)?;
    
    // Extract all packages with repository information
    let mut repositories = HashMap::new();
    let mut crates_without_repo = Vec::new();
    
    if let Some(packages) = metadata["packages"].as_array() {
        for package in packages {
            let name = package["name"].as_str().unwrap().to_string();
            
            if let Some(repo_url) = package["repository"].as_str() {
                repositories.insert(name.clone(), repo_url.to_string());
            } else {
                crates_without_repo.push(name);
            }
        }
    }
    
    println!("Found {} crates with repository information", repositories.len());
    println!("Found {} crates without repository information", crates_without_repo.len());
    
    // Build comprehensive dependency graph
    let graph_output = Command::new("cargo")
        .args(["run", "--quiet", "--", "global-graph", "build", ".", 
               "--output-dir", "/tmp/repo_analysis",
               "--include-dev", "--include-build"])
        .output()?;
    
    if graph_output.status.success() {
        println!("\n=== Comprehensive Dependency Graph ===");
        let graph_json = std::fs::read_to_string("/tmp/repo_analysis/graph.json")?;
        let graph_data: Value = serde_json::from_str(&graph_json)?;
        
        println!("Total nodes: {}", graph_data["nodes"].as_array().unwrap().len());
        println!("Total edges: {}", graph_data["edges"].as_array().unwrap().len());
        
        // Analyze dependency types
        let mut regular_deps = 0;
        let mut dev_deps = 0;
        let mut build_deps = 0;
        
        if let Some(edges) = graph_data["edges"].as_array() {
            for edge in edges {
                if edge["is_dev_dependency"].as_bool().unwrap_or(false) {
                    dev_deps += 1;
                } else if edge["is_build_dependency"].as_bool().unwrap_or(false) {
                    build_deps += 1;
                } else {
                    regular_deps += 1;
                }
            }
        }
        
        println!("Dependency breakdown:");
        println!("  Regular: {}", regular_deps);
        println!("  Dev: {}", dev_deps);
        println!("  Build: {}", build_deps);
    }
    
    println!("\n=== Crates with Git Repositories ===");
    let mut repo_list: Vec<_> = repositories.iter().collect();
    repo_list.sort_by(|a, b| a.0.cmp(b.0));
    
    for (crate_name, repo_url) in repo_list {
        println!("{}: {}", crate_name, repo_url);
    }
    
    println!("\n=== Crates without Repository Information ===");
    crates_without_repo.sort();
    for crate_name in crates_without_repo {
        println!("- {}", crate_name);
    }
    
    // Analyze repository hosts
    println!("\n=== Repository Host Analysis ===");
    let mut hosts = HashMap::new();
    
    for repo_url in repositories.values() {
        if repo_url.starts_with("https://github.com/") {
            *hosts.entry("GitHub").or_insert(0) += 1;
        } else if repo_url.starts_with("https://gitlab.com/") {
            *hosts.entry("GitLab").or_insert(0) += 1;
        } else if repo_url.starts_with("https://") {
            let host = repo_url.split('/').nth(2).unwrap_or("unknown");
            *hosts.entry(host).or_insert(0) += 1;
        } else {
            *hosts.entry("Other").or_insert(0) += 1;
        }
    }
    
    for (host, count) in hosts {
        println!("{}: {} crates", host, count);
    }
    
    // GitHub-specific analysis
    println!("\n=== GitHub Organization Analysis ===");
    let mut github_orgs = HashMap::new();
    
    for repo_url in repositories.values() {
        if repo_url.starts_with("https://github.com/") {
            let parts: Vec<&str> = repo_url.split('/').collect();
            if parts.len() >= 4 {
                let org = parts[3];
                *github_orgs.entry(org).or_insert(0) += 1;
            }
        }
    }
    
    let mut org_list: Vec<_> = github_orgs.iter().collect();
    org_list.sort_by(|a, b| b.1.cmp(a.1)); // Sort by count descending
    
    for (org, count) in org_list {
        println!("{}: {} crates", org, count);
    }
    
    Ok(())
}