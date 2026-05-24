//! # Workflow System
//!
//! High-level workflow orchestration that integrates all major components of cargo-vendormod
//! into cohesive, automated processing pipelines.
//!
//! ## Overview
//!
//! The `WorkflowRunner` coordinates the complete vendoring workflow:
//! 1. Build the global dependency graph
//! 2. Process crates in layered topological order
//! 3. Execute post-processing scripts
//! 4. Generate Git operations and patches
//!
//! ## Components
//!
//! - **Dependency Analysis**: `GlobalDependencyGraphBuilder` constructs the full dependency tree
//! - **Layered Processing**: `LayerProcessor` handles crates in correct dependency order
//! - **Git Integration**: `GitWrapper` manages repository operations
//! - **Error Handling**: Comprehensive `anyhow::Result`-based error propagation
//!
//! ## Usage
//!
//! ```no_run
//! use cargo_vendormod::workflow::{WorkflowRunner, WorkflowConfig};
//! use std::path::PathBuf;
//!
//! let config = WorkflowConfig {
//!     workspace_path: PathBuf::from("/my/workspace"),
//!     output_dir: PathBuf::from("./output"),
//!     git_exe: PathBuf::from("git"),
//!     generate_flakes: true,
//!     compile_standalone: true,
//! };
//!
//! let runner = WorkflowRunner::new(config);
//! runner.run()?;
//! ```

use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

use crate::git_wrapper::GitWrapper;
use crate::global_dep_graph::{GlobalDependencyGraph, GlobalDependencyGraphBuilder};
use crate::layer_processor::LayerProcessor;

/// Workflow configuration
pub struct WorkflowConfig {
    pub workspace_path: PathBuf,
    pub output_dir: PathBuf,
    pub git_exe: PathBuf,
    pub generate_flakes: bool,
    pub compile_standalone: bool,
}

/// Workflow runner
pub struct WorkflowRunner {
    config: WorkflowConfig,
    git_wrapper: GitWrapper,
}

impl WorkflowRunner {
    /// Create a new workflow runner
    pub fn new(config: WorkflowConfig) -> Self {
        let git_wrapper = GitWrapper::new(config.git_exe.clone());

        Self {
            config,
            git_wrapper,
        }
    }

    /// Run the complete workflow
    pub fn run(&self) -> Result<()> {
        println!("🚀 Starting cargo-vendormod workflow");

        // Step 1: Build dependency graph
        let graph = self.build_dependency_graph()?;

        // Step 2: Process crates in layers
        self.process_crates(&graph)?;

        // Step 3: Run post-processing scripts
        self.run_post_processing_scripts()?;

        println!("✅ Workflow completed successfully!");
        Ok(())
    }

    /// Build the global dependency graph
    fn build_dependency_graph(&self) -> Result<GlobalDependencyGraph> {
        println!("📊 Building dependency graph...");

        let mut builder = GlobalDependencyGraphBuilder::new(self.config.workspace_path.clone());
        builder.set_options(true, true, true);

        let graph = builder.build_global_graph()?;

        println!("📊 Dependency graph built successfully");
        println!("   - Total nodes: {}", graph.nodes.len());
        println!("   - Total edges: {}", graph.edges.len());

        // Count workspace members and external dependencies
        let workspace_members = graph.nodes.iter().filter(|n| n.is_workspace_member).count();
        let external_deps = graph
            .nodes
            .iter()
            .filter(|n| !n.is_workspace_member)
            .count();

        println!("   - Workspace members: {}", workspace_members);
        println!("   - External dependencies: {}", external_deps);

        Ok(graph)
    }

    /// Process crates using the layer processor
    fn process_crates(&self, graph: &GlobalDependencyGraph) -> Result<()> {
        println!("🏗️  Processing crates...");

        let processor = LayerProcessor::new(
            graph.clone(),
            self.config.workspace_path.clone(),
            self.config.output_dir.clone(),
            self.git_wrapper.git_exe().clone(),
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")),
        );

        processor.process_layers()?;

        println!("🏗️  Crate processing completed");
        Ok(())
    }

    /// Run post-processing scripts
    fn run_post_processing_scripts(&self) -> Result<()> {
        println!("🧪 Running post-processing scripts...");

        // Run quality checks
        self.run_script("check.sh")?;

        // Run tests
        self.run_script("test.sh")?;

        println!("🧪 Post-processing scripts completed");
        Ok(())
    }

    /// Run a script from the scripts directory
    fn run_script(&self, script_name: &str) -> Result<()> {
        let script_path = self.config.workspace_path.join("scripts").join(script_name);

        if !script_path.exists() {
            println!("   ⚠️  Script {} not found, skipping", script_name);
            return Ok(());
        }

        println!("   📜 Running {}...", script_name);

        let output = Command::new("bash")
            .arg(script_path)
            .current_dir(&self.config.workspace_path)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Script {} failed: {}",
                script_name,
                error_msg
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("   ✅ {} completed successfully", script_name);

        Ok(())
    }
}

/// Workflow factory for creating different types of workflows
pub struct WorkflowFactory;

impl WorkflowFactory {
    /// Create a standard workflow
    pub fn create_standard_workflow(
        workspace_path: PathBuf,
        output_dir: PathBuf,
    ) -> WorkflowConfig {
        WorkflowConfig {
            workspace_path,
            output_dir,
            git_exe: PathBuf::from("git"),
            generate_flakes: true,
            compile_standalone: true,
        }
    }

    /// Create a minimal workflow (no compilation, no flakes)
    pub fn create_minimal_workflow(workspace_path: PathBuf, output_dir: PathBuf) -> WorkflowConfig {
        WorkflowConfig {
            workspace_path,
            output_dir,
            git_exe: PathBuf::from("git"),
            generate_flakes: false,
            compile_standalone: false,
        }
    }

    /// Create a CI workflow (full validation)
    pub fn create_ci_workflow(workspace_path: PathBuf, output_dir: PathBuf) -> WorkflowConfig {
        WorkflowConfig {
            workspace_path,
            output_dir,
            git_exe: PathBuf::from("git"),
            generate_flakes: true,
            compile_standalone: true,
        }
    }
}
