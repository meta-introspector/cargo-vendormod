//! # Command-Line Arguments
//!
//! Defines the CLI interface for cargo-vendormod using Clap's derive API.
//!
//! ## Overview
//!
//! The `Args` struct represents all configurable options available through
//! the command line. Options are organized into global flags and subcommands
//! that implement specific vendoring workflows.
//!
//! ## Global Options
//!
//! These apply to all commands:
//!
//! - `--verbose, -v`: Enable debug logging
//! - `--root-dir`: Workspace root directory
//! - `--manifest-path`: Cargo.toml file path
//! - `--submodules-path`: Vendored sources directory
//! - `--mirrors-path`: Bare repository mirrors directory
//! - `--vendor-dir`: Registry crate vendor directory
//! - `--target-branch`: Default branch for operations
//! - `--create-version-branches`: Auto-create version branches
//! - `--version-branch-format`: Format string for version branches
//! - `--dry-run`: Preview mode without changes
//! - `--output-file`: Write JSON plan to file
//!
//! ## Subcommands
//!
//! ### Vendoring
//! - `vendoring` - Convert vendored crates to git submodules
//!
//! ### Git Operations
//! - `fetch-upstream` - Fetch latest upstream changes
//! - `rebase` - Rebase submodules onto latest upstream
//! - `releases` - Fetch tags and create version branches
//! - `status` - Show submodule status
//! - `sync` - Full sync (fetch, rebase, patches)
//!
//! ### Workspace Management
//! - `edit` - Edit workspace Cargo.toml
//! - `workspace` - Process workspace members
//!
//! ### Processing
//! - `process-crates` - Process crates from file list
//! - `run-workflow` - Execute complete workflow
//!
//! ### Graph Analysis
//! - `global-graph build` - Build dependency graph
//! - `global-graph analyze` - Analyze graph structure
//! - `global-graph visualize` - Generate DOT file
//! - `global-graph toml-structure` - Export TOML schema
//! - `global-graph partition` - Partition graph
//!
//! ### Utility
//! - `patch` - Update .cargo/config.toml patches
//!
//! ## Usage Example
//!
//! ```text
//! # Process all crates from a list
//! cargo-vendormod process-crates /path/to/crates.txt \
//!     --output-dir ./output \
//!     --layered-processing \
//!     --max-parallel 8
//!
//! # Run full workflow
//! cargo-vendormod run-workflow \
//!     --workspace-path ./my-workspace \
//!     --output-dir ./output
//!
//! # Generate dependency graph
//! cargo-vendormod global-graph build \
//!     --workspace-path ./workspace \
//!     --output-format json
//! ```

use clap::{ArgAction, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Enable verbose output for debugging.
    #[arg(long)]
    pub verbose: bool,

    /// The root directory of the cargo project.
    #[arg(long, default_value = ".")]
    pub root_dir: PathBuf,

    /// Path to Cargo.toml manifest file. If not provided, auto-detects (Cargo.toml or *Cargo.toml).
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Path to central submodules directory (vendored sources).
    #[arg(long)]
    pub submodules_path: Option<PathBuf>,

    /// Path to local git mirrors directory (bare repositories).
    #[arg(long)]
    pub mirrors_path: Option<PathBuf>,

    /// Path to the vendor directory containing vendored crate sources (for registry crates).
    #[arg(long)]
    pub vendor_dir: Option<PathBuf>,

    /// The branch to checkout and use for dependencies.
    #[arg(long)]
    pub target_branch: Option<String>,

    /// Create a branch in the bare mirror for each version used (format: e.g., v{version} or {version}).
    #[arg(long, action = ArgAction::SetTrue)]
    pub create_version_branches: Option<bool>,

    /// Format string for version branches in bare mirrors. Use {} as placeholder for version.
    #[arg(long)]
    pub version_branch_format: Option<String>,

    /// Perform a dry run without making actual changes.
    #[arg(long)]
    pub dry_run: bool,

    /// Optional: Write the JSON plan to this file instead of stdout.
    #[arg(long)]
    pub output_file: Option<PathBuf>,

    /// Source repository to discover submodules from (for bootstrapping).
    #[arg(long, default_value = ".")]
    pub source_repo: PathBuf,

    /// Include dev dependencies when discovering from source.
    #[arg(long, default_value = "true")]
    pub include_dev: bool,

    /// Include build dependencies when discovering from source.
    #[arg(long, default_value = "true")]
    pub include_build: bool,

    /// Include optional dependencies when discovering from source.
    #[arg(long, default_value = "true")]
    pub include_optional: bool,

    /// Fetch repository URLs from crates.io API for registry crates.
    #[arg(long, default_value = "false")]
    pub fetch_crates_io_repos: bool,

    /// Subcommand to execute.
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(clap::Args, Debug, Clone)]
pub struct EditArgs {
    /// Sort dependencies alphabetically by name
    #[arg(long)]
    pub sort: bool,
    /// Add missing dependencies that are present in submodules but not listed
    #[arg(long)]
    pub add_missing: bool,
    /// Remove dependencies whose path no longer exists on disk
    #[arg(long)]
    pub remove_unused: bool,
    /// Update version fields to match package versions from Cargo.toml files
    #[arg(long)]
    pub update_versions: bool,
}

/******************************************************************************/
/* SELinux Commands */
/******************************************************************************/

#[derive(Subcommand, Debug)]
pub enum SelinuxCommands {
    /// Parse .service files and generate access reports
    Analyze {
        /// Input .service file(s) or directory
        inputs: Vec<PathBuf>,
        /// Output directory for results
        #[arg(long, default_value = "./selinux_output")]
        output_dir: PathBuf,
    },
    /// Generate SELinux .te policy file
    Policy {
        /// Input .service file(s) or directory
        inputs: Vec<PathBuf>,
        /// Output .te policy file
        #[arg(long, default_value = "./selinux_output/zkperf_services.te")]
        output: PathBuf,
    },
    /// Generate Lean4 formal verification model
    Lean4 {
        /// Input .service file(s) or directory
        inputs: Vec<PathBuf>,
        /// Output Lean4 model file
        #[arg(long, default_value = "./selinux_output/model.lean")]
        output: PathBuf,
    },
}

/******************************************************************************/

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Convert vendored crates to git submodules (original functionality)
    Vendoring,
    /// Fetch latest upstream changes into all bare mirrors in parallel (24 CPUs)
    FetchUpstream,
    /// Rebase all submodule changes onto latest upstream head
    Rebase,
    /// Fetch all release tags and create version branches in bare mirrors
    Releases,
    /// Generate/update .cargo/config.toml patches for vendored crates
    Patch,
    /// Show status of all submodules vs upstream/bare
    Status,
    /// Show workspace status (git state, build health, dependency status)
    WorkspaceStatus {
        /// Path to workspace root directory
        #[arg(long)]
        workspace_path: PathBuf,
        /// Include dependency health check
        #[arg(long, action = ArgAction::SetTrue)]
        check_deps: bool,
        /// Include build status
        #[arg(long, action = ArgAction::SetTrue)]
        check_build: bool,
    },
    /// Analyze workspace: count vendored crates, submodules, branches, Nix build status
    WorkspaceAnalyze {
        /// Path to cargo2nix workspace root
        #[arg(long)]
        workspace_path: PathBuf,
    },
    /// Full sync: fetch upstream, rebase, update patches (all in parallel)
    Sync,
    /// Edit the workspace Cargo.toml: sort, add missing, remove unused, update versions
    Edit(EditArgs),
    /// Process workspace members recursively and extract git dependencies
    Workspace {
        /// Path to workspace root directory
        workspace_path: PathBuf,
        /// Process all workspace members recursively
        #[arg(long, action = ArgAction::SetTrue)]
        recursive: bool,
    },
    /// Generic workload vendoring with local forking and optional zkperf annotations
    Workload {
        /// Path to workspace root directory
        workspace_path: PathBuf,
        /// Directory to store local forks of repositories
        #[arg(long, default_value = "./forks")]
        fork_dir: PathBuf,
        /// Apply cargo zkperf annotations to all dependencies
        #[arg(long, action = ArgAction::SetTrue)]
        zkperf: bool,
    },
    /// Global dependency graph operations for Solana universe analysis
    GlobalGraph {
        #[command(subcommand)]
        command: GlobalGraphCommands,
    },
    /// Process crates with layered topological processing
    ProcessCrates {
        /// Path to workspace root directory
        workspace_path: PathBuf,

        /// Output directory for processed crates
        #[arg(long, default_value = "./processed")]
        output_dir: PathBuf,

        /// Generate Nix flakes for each crate
        #[arg(long, action = ArgAction::SetTrue)]
        generate_flakes: bool,

        /// Compile crates standalone
        #[arg(long, action = ArgAction::SetTrue)]
        compile_standalone: bool,

        /// Use layered processing (external deps first, then workspace)
        #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
        layered_processing: bool,

        /// Process only specific layer (1 for external deps, 2 for workspace members)
        #[arg(long)]
        layer: Option<u32>,
    },

    /// Onboard a new repository, crate, or workspace
    Onboard {
        /// Git repository URL to onboard
        #[arg(long)]
        git_repo: Option<String>,

        /// Crate name to onboard from crates.io
        #[arg(long)]
        crate_name: Option<String>,

        /// Local workspace directory to onboard
        #[arg(long)]
        workdir: Option<PathBuf>,

        /// Git branch to use (default: main)
        #[arg(long, default_value = "main")]
        branch: String,

        /// Workflow name (default: full_onboarding)
        #[arg(long, default_value = "full_onboarding")]
        workflow: String,

        /// Output directory for onboarded workspace
        #[arg(long, default_value = "./workload/workspaces")]
        output_dir: PathBuf,
    },

    /// Fix Cargo.toml files for Rust toolchain components
    FixCargoToml,

    /// Fix edition configuration in existing workspaces
    FixEdition {
        /// Base directory containing workspaces to fix
        base_dir: PathBuf,

        /// Rust edition to set (e.g., "2021")
        #[arg(long, default_value = "2021")]
        edition: String,
    },

    /// Run a complete workflow (dependency analysis + crate processing + scripts)
    RunWorkflow {
        /// Path to the workspace directory
        workspace_path: PathBuf,

        /// Output directory for processed crates
        #[arg(long, default_value = "./processed")]
        output_dir: PathBuf,

        /// Workflow type: standard, minimal, or ci
        #[arg(long, value_enum, default_value = "standard")]
        workflow_type: Option<String>,
    },

    /// Process all crates from a list of Cargo.toml files
    ProcessAllCrates {
        /// Path to file containing list of Cargo.toml paths
        input_file: PathBuf,

        /// Output directory for processed crates
        #[arg(long, default_value = "./processed_all")]
        output_dir: PathBuf,

        /// Maximum number of parallel processes
        #[arg(long, default_value = "4")]
        max_parallel: usize,
    },

    /// Generate a comprehensive processing report
    GenerateReport {
        /// Path to file containing list of Cargo.toml paths (for input stats)
        input_file: PathBuf,

        /// Output directory to analyze
        #[arg(long, default_value = "./processed_all")]
        output_dir: PathBuf,
    },
    /// Split a Lean 4 mathlib-style project into per-declaration flakes via lean-split-tool
    SplitLean4 {
        /// Path to mathlib source (directory containing Mathlib/)
        #[arg(long)]
        mathlib_src: PathBuf,

        /// Output directory for split flakes
        #[arg(long, default_value = "./mathlib-split")]
        output_dir: PathBuf,

        /// Branch to push to in the target repo
        #[arg(long, default_value = "feature/split")]
        branch: String,

        /// Path to the lean-split-tool split script
        #[arg(long, default_value = "/home/mdupont/projects/lean-split-tool/split-mathlib.sh")]
        split_tool: PathBuf,

        /// Dry-run: only print what would be executed
        #[arg(long)]
        dry_run: bool,
    },
        /// Path to directory containing Nix flakes
        flake_dir: PathBuf,
        /// Maximum number of parallel builds
        #[arg(long, default_value = "8")]
        max_parallel: usize,
        /// Maximum number of retries for failed builds
        #[arg(long, default_value = "2")]
        max_retries: usize,
        /// Timeout for individual builds in seconds
        #[arg(long, default_value = "3600")]
        timeout_seconds: u64,
        /// Output directory for build artifacts
        #[arg(long, default_value = "./nix_builds")]
        output_dir: PathBuf,
        /// Directory for build logs
        #[arg(long, default_value = "./build_logs")]
        log_dir: PathBuf,
        /// Path to workspace for dependency analysis
        #[arg(long)]
        workspace_path: Option<PathBuf>,
    },
    /// Analyze build errors and generate LLM-friendly report
    GenerateErrorReport {
        /// Path to build results JSON file
        results_file: PathBuf,
        /// Output path for LLM error report
        #[arg(long, default_value = "./llm_error_report.md")]
        output_path: PathBuf,
    },
/// Generate comprehensive workload status report
    Report {
        /// Path to workspace root directory
        #[arg(long)]
        workspace_path: PathBuf,
        /// Output directory for report
        #[arg(long, default_value = "./crate_report")]
        output_dir: PathBuf,
    },
    /// Workload discovery, creation, and management
    Workloads {
        /// Path to workspace root directory
        #[arg(long)]
        workspace_path: PathBuf,
        /// Workload name (for specific workload operations)
        #[arg(long)]
        workload: Option<String>,
        /// Output path for matrix/discover results
        #[arg(long)]
        output: Option<PathBuf>,
        /// Create all workloads (for create-all command)
        #[arg(long, action = ArgAction::SetTrue)]
        create_all: bool,
        /// Aggressive optimization mode
        #[arg(long, action = ArgAction::SetTrue)]
        aggressive: bool,
        /// Run analysis on workload
        #[arg(long, action = ArgAction::SetTrue)]
        analyze: bool,
        /// Correlate multiple analysis tools
        #[arg(long, action = ArgAction::SetTrue)]
        correlate: bool,
    },
    /// Extract ALL dependencies (git + crates.io) as submodules in newroot/host/owner/repo
    ExtractAll {
        /// Path to workspace root directory
        #[arg(long)]
        workspace_path: PathBuf,
        /// Output directory for extracted submodules
        #[arg(long, default_value = "./newroot")]
        newroot: PathBuf,
        /// Mirrors path for bare git repos
        #[arg(long, default_value = "/home/mdupont/git/host")]
        mirrors_path: PathBuf,
        /// Process registry (crates.io) dependencies
        #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
        include_registry: bool,
        /// Process git dependencies
        #[arg(long, action = ArgAction::SetTrue, default_value = "true")]
        include_git: bool,
    },
}

/// Subcommands for global dependency graph operations
#[derive(Subcommand, Debug)]
pub enum GlobalGraphCommands {
    /// Build the global dependency graph from workspace
    Build {
        /// Path to workspace root directory
        workspace_path: PathBuf,
        /// Include dev dependencies in the graph
        #[arg(long, action = ArgAction::SetTrue)]
        include_dev: bool,
        /// Include build dependencies in the graph
        #[arg(long, action = ArgAction::SetTrue)]
        include_build: bool,
        /// Expand all feature dependencies
        #[arg(long, action = ArgAction::SetTrue)]
        expand_features: bool,
        /// Output directory for graph files
        #[arg(long, default_value = "./analysis/global_graph")]
        output_dir: PathBuf,
    },
    /// Analyze the global dependency graph for Solana patterns
    Analyze {
        /// Path to the JSON graph file
        input_path: PathBuf,
        /// Output directory for analysis results
        #[arg(long, default_value = "./analysis/global_graph")]
        output_dir: PathBuf,
    },
    /// Generate visualization of the dependency graph
    Visualize {
        /// Path to the JSON graph file
        input_path: PathBuf,
        /// Output path for visualization file
        #[arg(long, default_value = "./analysis/global_graph/graph.dot")]
        output_path: PathBuf,
    },
    /// Generate TOML structure analysis and visualization
    TomlStructure {
        /// Path to the JSON graph file
        input_path: PathBuf,
        /// Output directory for TOML structure analysis
        #[arg(long, default_value = "./analysis/toml_structure")]
        output_dir: PathBuf,
    },
    /// Perform graph partitioning using KaMinPar
    Partition {
        /// Path to the JSON graph file
        input_path: PathBuf,
        /// Number of partitions to create
        #[arg(long, default_value = "8")]
        partition_count: usize,
        /// Balance factor for partitioning (0.0-1.0)
        #[arg(long, default_value = "0.03")]
        balance_factor: f64,
        /// Partitioning algorithm to use
        #[arg(long, default_value = "kaminpar")]
        algorithm: String,
        /// Output directory for partition results
        #[arg(long, default_value = "./analysis/partitions")]
        output_dir: PathBuf,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::global_dep_graph::PartitioningAlgorithm;

    #[test]
    fn test_args_default_values() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["cargo-vendormod"])?;
        assert!(!args.verbose);
        assert_eq!(args.root_dir, PathBuf::from("."));
        assert!(args.manifest_path.is_none());
        assert!(args.submodules_path.is_none());
        assert!(args.mirrors_path.is_none());
        assert!(args.vendor_dir.is_none());
        assert!(args.target_branch.is_none());
        assert!(!args.dry_run);
        assert!(args.output_file.is_none());
        Ok(())
    }

    #[test]
    fn test_args_with_verbose_flag() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["cargo-vendormod", "--verbose"])?;
        assert!(args.verbose);
        Ok(())
    }

    #[test]
    fn test_args_with_custom_root_dir() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["cargo-vendormod", "--root-dir", "/tmp/test"])?;
        assert_eq!(args.root_dir, PathBuf::from("/tmp/test"));
        Ok(())
    }

    #[test]
    fn test_args_with_dry_run() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["cargo-vendormod", "--dry-run"])?;
        assert!(args.dry_run);
        Ok(())
    }

    #[test]
    fn test_args_with_output_file() -> Result<(), clap::Error> {
        let args = Args::try_parse_from(["cargo-vendormod", "--output-file", "/tmp/output.json"])?;
        assert_eq!(args.output_file, Some(PathBuf::from("/tmp/output.json")));
        Ok(())
    }

    #[test]
    fn test_edit_args_defaults() {
        // EditArgs is a subcommand args struct, test default values
        let args = EditArgs {
            sort: false,
            add_missing: false,
            remove_unused: false,
            update_versions: false,
        };
        assert!(!args.sort);
        assert!(!args.add_missing);
        assert!(!args.remove_unused);
        assert!(!args.update_versions);
    }

    #[test]
    fn test_partitioning_algorithm_from_str() {
        use std::str::FromStr;
        assert_eq!(PartitioningAlgorithm::from_str("kaminpar").unwrap(), PartitioningAlgorithm::KaMinPar);
        assert_eq!(PartitioningAlgorithm::from_str("KAMINPAR").unwrap(), PartitioningAlgorithm::KaMinPar);
        assert_eq!(PartitioningAlgorithm::from_str("metis").unwrap(), PartitioningAlgorithm::Metis);
        assert_eq!(PartitioningAlgorithm::from_str("louvain").unwrap(), PartitioningAlgorithm::Louvain);
        assert_eq!(PartitioningAlgorithm::from_str("kernighanlin").unwrap(), PartitioningAlgorithm::KernighanLin);
        assert_eq!(PartitioningAlgorithm::from_str("spectral").unwrap(), PartitioningAlgorithm::Spectral);
        assert_eq!(PartitioningAlgorithm::from_str("greedy").unwrap(), PartitioningAlgorithm::Greedy);
        assert!(PartitioningAlgorithm::from_str("unknown").is_err());
    }
}
