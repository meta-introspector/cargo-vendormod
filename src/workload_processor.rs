//! # Workload Processor
//!
//! Uses existing workspace and workload modules to process git submodules
//! and Cargo.toml files with performance measurement.

use anyhow::Result;
use crate::workspace;
use std::path::PathBuf;
use std::time::Instant;

/// Performance metrics for workload processing
#[derive(Debug, serde::Serialize)]
pub struct WorkloadMetrics {
    pub total_repos: usize,
    pub total_cargoTOMs: usize,
    pub elapsed_ms: u128,
    pub repos_per_second: f64,
    pub details: Vec<RepoProcessDetail>,
}

#[derive(Debug, serde::Serialize)]
pub struct RepoProcessDetail {
    pub name: String,
    pub cargoTOMs: usize,
    pub elapsed_ms: u128,
}

impl Default for WorkloadMetrics {
    fn default() -> Self {
        Self {
            total_repos: 0,
            total_cargoTOMs: 0,
            elapsed_ms: 0,
            repos_per_second: 0.0,
            details: Vec::new(),
        }
    }
}

/// Process git submodules and Cargo.toml files recursively
pub fn process_workload_recursive(root: &PathBuf) -> Result<WorkloadMetrics> {
    let start = Instant::now();
    let mut metrics = WorkloadMetrics::default();
    
    println!("🚀 Starting workload processing from: {}", root.display());
    
    // Use existing function from workspace.rs
    let git_repos = workspace::discover_all_git_repositories(root)?;
    println!("📦 Found {} git repositories", git_repos.len());
    
    for repo_path in &git_repos {
        let repo_start = Instant::now();
        let repo_name = repo_path.file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let cargo_files = workspace::find_workspace_cargo_files(repo_path, true)?;
        let elapsed = repo_start.elapsed().as_millis();
        
        metrics.details.push(RepoProcessDetail {
            name: repo_name.to_string(),
            cargoTOMs: cargo_files.len(),
            elapsed_ms: elapsed,
        });
        
        metrics.total_cargoTOMs += cargo_files.len();
    }
    
    metrics.total_repos = git_repos.len();
    metrics.elapsed_ms = start.elapsed().as_millis();
    
    if metrics.elapsed_ms > 0 {
        metrics.repos_per_second = metrics.total_repos as f64 / (metrics.elapsed_ms as f64 / 1000.0);
    }
    
    println!("✅ Completed in {}ms ({} repos/sec)", 
        metrics.elapsed_ms, 
        metrics.repos_per_second);
    
    Ok(metrics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workload_metrics_default() {
        let metrics = WorkloadMetrics::default();
        assert_eq!(metrics.total_repos, 0);
        assert_eq!(metrics.total_cargoTOMs, 0);
        assert_eq!(metrics.details.len(), 0);
    }

    #[test]
    fn test_workload_metrics_serialization() {
        let mut metrics = WorkloadMetrics::default();
        metrics.total_repos = 5;
        metrics.total_cargoTOMs = 10;
        metrics.elapsed_ms = 100;
        metrics.details.push(RepoProcessDetail {
            name: "test".to_string(),
            cargoTOMs: 2,
            elapsed_ms: 20,
        });
        
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("5"));
    }
}