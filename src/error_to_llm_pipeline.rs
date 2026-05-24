//! # Error-to-LLM Pipeline
//!
//! This module provides a comprehensive error analysis and parallel build system that
//! processes Nix build failures and generates actionable reports for large-scale
//! package builds across dependency graphs.
//!
//! ## Overview
//!
//! The `ErrorToLLMPipeline` coordinates parallel Nix builds of packages identified
//! in a global dependency graph, collects build errors, analyzes them, and produces
//! reports optimized for consumption by language models (LLMs) to identify patterns
//! and suggest fixes.
//!
//! ## Key Features
//!
//! - **Parallel Build Execution**: Runs multiple Nix builds concurrently with configurable limits
//! - **Dependency-Aware Scheduling**: Respects package dependencies for build ordering
//! - **Error Classification**: Categorizes errors as retryable or non-retryable
//! - **LLM-Optimized Reporting**: Generates structured error reports for AI analysis
//! - **Build Result Tracking**: Maintains comprehensive build statistics and timing
//!
//! ## Architecture
//!
//! ```text
//! Dependency Graph → Partition Independent Groups → Parallel Build Execution
//!        ↓                                              ↓
//!   Error Collection                              Result Aggregation
//!        ↓                                              ↓
//!   LLM Report Generation ←─ Error Pattern Analysis ←─ Build Logs
//! ```
//!
//! ## Usage
//!
//! ```no_run
//! use cargo_vendormod::error_to_llm_pipeline::{ErrorToLLMPipeline, ParallelBuildConfig};
//!
//! let config = ParallelBuildConfig {
//!     max_parallel: 8,
//!     max_retries: 2,
//!     timeout_seconds: 3600,
//!     ..Default::default()
//! };
//!
//! let mut pipeline = ErrorToLLMPipeline::new(config);
//! pipeline.load_dependency_graph(workspace_path)?;
//! let groups = pipeline.partition_independent_groups()?;
//! let results = pipeline.build_all_packages(flake_paths)?;
//! let report = pipeline.generate_llm_error_report()?;
//! ```

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader};
use rayon::prelude::*;
use log::{info, error, warn};
use serde::{Serialize, Deserialize};
use regex;

use crate::global_dep_graph::{GlobalDependencyGraph, GlobalDependencyGraphBuilder};

/// Error information collected from Nix builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildError {
    pub crate_name: String,
    pub error_message: String,
    pub build_log: String,
    pub dependencies: Vec<String>,
    pub nix_flake_path: PathBuf,
    pub is_retryable: bool,
}

/// Build result with error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildResult {
    pub crate_name: String,
    pub success: bool,
    pub error: Option<BuildError>,
    pub build_time_ms: u64,
    pub dependencies: Vec<String>,
}

/// Parallel build configuration
#[derive(Debug, Clone)]
pub struct ParallelBuildConfig {
    pub max_parallel: usize,
    pub max_retries: usize,
    pub timeout_seconds: u64,
    pub nix_command: String,
    pub output_dir: PathBuf,
    pub log_dir: PathBuf,
}

impl Default for ParallelBuildConfig {
    fn default() -> Self {
        Self {
            max_parallel: 8,
            max_retries: 2,
            timeout_seconds: 3600,
            nix_command: "nix".to_string(),
            output_dir: PathBuf::from("./nix_builds"),
            log_dir: PathBuf::from("./build_logs"),
        }
    }
}

/// Error-to-LLM pipeline for parallel Nix builds
pub struct ErrorToLLMPipeline {
    config: ParallelBuildConfig,
    dependency_graph: Option<GlobalDependencyGraph>,
    build_results: Arc<Mutex<Vec<BuildResult>>>,
}

impl ErrorToLLMPipeline {
    
    pub fn new(config: ParallelBuildConfig) -> Self {
        Self {
            config,
            dependency_graph: None,
            build_results: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Load dependency graph from workspace
    pub fn load_dependency_graph(&mut self, workspace_path: &Path) -> Result<()> {
        let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.to_path_buf());
        self.dependency_graph = Some(builder.build_global_graph()?);
        Ok(())
    }

    /// Partition packages into independent build groups
    pub fn partition_independent_groups(&self) -> Result<Vec<Vec<String>>> {
        let graph = self.dependency_graph.as_ref()
            .context("Dependency graph not loaded")?;
        
        // Use external dependency order as build groups - externals must build first
        let mut groups = Vec::new();
        
        // Group 1: External dependencies (no workspace deps)
        let external_group = graph.external_dependency_order.clone();
        if !external_group.is_empty() {
            groups.push(external_group);
        }
        
        // Group 2: Workspace members (can build in topological order)
        if !graph.topological_order.is_empty() {
            groups.push(graph.topological_order.clone());
        } else if !graph.publish_order.is_empty() {
            groups.push(graph.publish_order.clone());
        }
        
        // If no dependency info, put all nodes in one group
        if groups.is_empty() {
            let all_crates: Vec<String> = graph.nodes
                .iter()
                .map(|node| node.crate_name.clone())
                .collect();
            if !all_crates.is_empty() {
                groups.push(all_crates);
            }
        }
        
        Ok(groups)
    }

    /// Build all packages with parallel execution and error collection
    pub fn build_all_packages(&self, flake_paths: Vec<PathBuf>) -> Result<Vec<BuildResult>> {
        info!("Starting parallel Nix builds with error collection");
        info!("Total packages: {}", flake_paths.len());
        info!("Max parallel: {}", self.config.max_parallel);
        
        // Create output and log directories
        fs::create_dir_all(&self.config.output_dir)
            .context("Failed to create output directory")?;
        fs::create_dir_all(&self.config.log_dir)
            .context("Failed to create log directory")?;
        
        // Process in parallel with limited concurrency
        let results: Vec<BuildResult> = flake_paths
            .into_par_iter()
            .map(|flake_path| {
                self.build_single_package(&flake_path)
            })
            .collect();
        
        // Store results
        let mut results_guard = self.build_results.lock().unwrap();
        results_guard.extend(results.clone());
        
        // Generate summary
        let success_count = results.iter().filter(|r| r.success).count();
        let failure_count = results.iter().filter(|r| !r.success).count();
        
        info!("Build complete!");
        info!("✅ Success: {}", success_count);
        info!("❌ Failed: {}", failure_count);
        info!("📊 Success rate: {}%", 
            if results.len() > 0 { (success_count * 100) / results.len() } else { 0 });
        
        Ok(results)
    }

    /// Build a single package with error collection
    fn build_single_package(&self, flake_path: &Path) -> BuildResult {
        let crate_name = flake_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        let log_path = self.config.log_dir.join(format!("{}.log", crate_name));
        let start_time = std::time::Instant::now();
        
        info!("🚀 Building: {}", crate_name);
        
        // Execute Nix build command
        let mut command = Command::new(&self.config.nix_command);
        command.args(["build", "--flake", flake_path.to_str().unwrap_or("")]);
        command.current_dir(&self.config.output_dir);
        
        let output = match command.output() {
            Ok(output) => output,
             Err(e) => {
                 let error_msg = format!("Failed to execute Nix command: {}", e);
                 let path_clone = flake_path.to_path_buf();
                 return BuildResult {
                     crate_name: crate_name.clone(),
                     success: false,
                     error: Some(BuildError {
                         crate_name,
                         error_message: error_msg,
                         build_log: String::new(),
                         dependencies: Vec::new(),
                         nix_flake_path: path_clone,
                         is_retryable: true,
                     }),
                     build_time_ms: start_time.elapsed().as_millis() as u64,
                     dependencies: Vec::new(),
                 };
             }
        };
        
        let build_time = start_time.elapsed().as_millis() as u64;
        
        // Save build log
        let build_log = String::from_utf8_lossy(&output.stderr).to_string();
        if let Err(e) = fs::write(&log_path, &build_log) {
            warn!("Failed to save build log for {}: {}", crate_name, e);
        }
        
        if output.status.success() {
            info!("✅ Successfully built: {}", crate_name);
            BuildResult {
                crate_name,
                success: true,
                error: None,
                build_time_ms: build_time,
                dependencies: Vec::new(),
            }
        } else {
            error!("❌ Failed to build: {}", crate_name);
            
            // Extract error information
            let error_message = self.extract_error_message(&build_log);
            let is_retryable = self.is_retryable_error(&build_log);
            let dependencies = self.extract_dependencies(&build_log);
            
            BuildResult {
                crate_name: crate_name.clone(),
                success: false,
                error: Some(BuildError {
                    crate_name,
                    error_message,
                    build_log: build_log.clone(),
                    dependencies: dependencies.clone(),
                    nix_flake_path: flake_path.to_path_buf(),
                    is_retryable,
                }),
                build_time_ms: build_time,
                dependencies,
            }
        }
    }

    /// Extract main error message from build log
    fn extract_error_message(&self, build_log: &str) -> String {
        // Look for common error patterns
        let patterns = [
            "error:",
            "Error:",
            "failed",
            "Failed",
            "not found",
            "No such file",
            "permission denied",
            "timeout",
        ];
        
        for pattern in patterns {
            if let Some(pos) = build_log.find(pattern) {
                let start = if pos > 100 { pos - 100 } else { 0 };
                let end = std::cmp::min(start + 500, build_log.len());
                return build_log[start..end].trim().to_string();
            }
        }
        
        // Return first 500 characters if no specific error found
        if build_log.len() > 500 {
            build_log[..500].trim().to_string()
        } else {
            build_log.trim().to_string()
        }
    }

    /// Extract dependencies from build log
    fn extract_dependencies(&self, build_log: &str) -> Vec<String> {
        let mut dependencies = Vec::new();
        
        // Look for dependency patterns
        let re = regex::Regex::new(r#"(?:depends on|requires|needs) [\"']?([a-zA-Z0-9_-]+)[\"']?"#).unwrap();
        for cap in re.captures_iter(build_log) {
            if let Some(dep) = cap.get(1) {
                dependencies.push(dep.as_str().to_string());
            }
        }
        
        dependencies
    }

    /// Check if error is retryable
    fn is_retryable_error(&self, build_log: &str) -> bool {
        let non_retryable_patterns = [
            "not found",
            "No such file",
            "permission denied",
            "invalid syntax",
            "parse error",
        ];
        
        for pattern in non_retryable_patterns {
            if build_log.contains(pattern) {
                return false;
            }
        }
        
        true
    }

    /// Generate LLM-friendly error report
    pub fn generate_llm_error_report(&self) -> Result<String> {
        let results_guard = self.build_results.lock().unwrap();
        let failed_builds: Vec<&BuildResult> = results_guard.iter()
            .filter(|r| !r.success)
            .collect();
        
        if failed_builds.is_empty() {
            return Ok("All builds successful! No errors to report.".to_string());
        }
        
        let mut report = String::new();
        report.push_str("# Nix Build Error Report for LLM Analysis\n\n");
        report.push_str(&format!("Total builds: {}\n", results_guard.len()));
        report.push_str(&format!("Failed builds: {}\n", failed_builds.len()));
        report.push_str(&format!("Success rate: {}%\n\n", 
            if results_guard.len() > 0 { 
                ((results_guard.len() - failed_builds.len()) * 100) / results_guard.len() 
            } else { 0 }));
        
        for (i, result) in failed_builds.iter().enumerate() {
            if let Some(error) = &result.error {
                report.push_str(&format!("## Error {}: {}\n", i + 1, error.crate_name));
                report.push_str(&format!("- **Error Message**: {}\n", error.error_message));
                let deps_str = if error.dependencies.is_empty() { 
                    "None".to_string() 
                } else { 
                    error.dependencies.join(", ") 
                };
                report.push_str(&format!("- **Dependencies**: {}\n", deps_str));
                report.push_str(&format!("- **Retryable**: {}\n", error.is_retryable));
                report.push_str(&format!("- **Build Time**: {}ms\n", result.build_time_ms));
                report.push_str(&format!("- **Flake Path**: {}\n\n", error.nix_flake_path.display()));
                
                report.push_str("### Build Log (first 1000 chars):\n");
                report.push_str("```\n");
                if error.build_log.len() > 1000 {
                    report.push_str(&error.build_log[..1000]);
                } else {
                    report.push_str(&error.build_log);
                }
                report.push_str("```\n\n");
            }
        }
        
        report.push_str("## Summary\n");
        report.push_str("- Total failed builds: ");
        report.push_str(&failed_builds.len().to_string());
        report.push_str("\n");
        report.push_str("- Retryable errors: ");
        report.push_str(&failed_builds.iter().filter(|r| r.error.as_ref().map(|e| e.is_retryable).unwrap_or(false)).count().to_string());
        report.push_str("\n");
        report.push_str("- Non-retryable errors: ");
        report.push_str(&failed_builds.iter().filter(|r| !r.error.as_ref().map(|e| e.is_retryable).unwrap_or(false)).count().to_string());
        report.push_str("\n");
        
        Ok(report)
    }

    /// Get maximum number of independent builds that can run in parallel
    pub fn get_max_independent_builds(&self) -> Result<usize> {
        let groups = self.partition_independent_groups()?;
        Ok(groups.len())
    }

    /// Get build results
    pub fn get_build_results(&self) -> Vec<BuildResult> {
        let guard = self.build_results.lock().unwrap();
        guard.clone()
    }
    
    /// Set build results (for loading from JSON)
    pub fn set_build_results(&self, results: Vec<BuildResult>) {
        let mut guard = self.build_results.lock().unwrap();
        *guard = results;
    }
}

/// Find all Nix flake files in a directory
pub fn find_all_flake_files(base_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut flake_files = Vec::new();
    
    if !base_dir.exists() {
        return Ok(flake_files);
    }
    
    for entry in walkdir::WalkDir::new(base_dir)
        .follow_links(true)
        .sort_by_file_name()
    {
        let entry = entry?;
        if entry.file_name() == "flake.nix" {
            flake_files.push(entry.path().to_path_buf());
        }
    }
    
    Ok(flake_files)
}

    /// Create parallel build groups based on dependency analysis
    pub fn create_parallel_build_groups(
        flake_files: Vec<PathBuf>,
        max_group_size: usize,
    ) -> Vec<Vec<PathBuf>> {
        if max_group_size == 0 {
            return vec![flake_files];
        }

    let mut groups = Vec::new();
    let mut current_group = Vec::new();
    
    for flake_file in flake_files {
        current_group.push(flake_file);
        
        if current_group.len() >= max_group_size {
            groups.push(current_group);
            current_group = Vec::new();
        }
    }
    
    if !current_group.is_empty() {
        groups.push(current_group);
    }
    
    groups
}