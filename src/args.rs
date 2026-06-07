
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use crate::warm_args::WarmCmd;
use crate::nora_indexer::NoraIndexArgs;

#[derive(Parser, Debug)]
#[command(name = "cargo-vendormod")]
#[command(version = "0.2.0")]
#[command(about = "Vendor git dependencies as submodules with local mirrors", long_about = None)]
pub struct MainArgs {
    /// Path to vendormod config file
    #[arg(long)]
    pub config: Option<PathBuf>,

    /// Path to Cargo.toml manifest file. If not provided, auto-detects (Cargo.toml or *Cargo.toml).
    #[arg(long)]
    pub manifest_path: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long, short)]
    pub verbose: bool,

    /// Root directory of the cargo project
    #[arg(long, default_value = ".")]
    pub root_dir: PathBuf,

    /// Submodules directory
    #[arg(long)]
    pub submodules_path: Option<PathBuf>,

    /// Mirrors directory
    #[arg(long)]
    pub mirrors_path: Option<PathBuf>,

    /// Vendor directory
    #[arg(long)]
    pub vendor_dir: Option<PathBuf>,

    /// Target branch
    #[arg(long)]
    pub target_branch: Option<String>,

    /// Create version branches
    #[arg(long)]
    pub create_version_branches: Option<bool>,

    /// Format string for version branches in bare mirrors. Use {} as placeholder for version.
    #[arg(long)]
    pub version_branch_format: Option<String>,

    /// Dry run mode
    #[arg(long)]
    pub dry_run: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Git submodule vendoring operations
    Vendoring(VendoringSubCmd),
    /// Dependency graph operations
    Graph(GraphSubCmd),
    /// Crate processing operations
    Process(ProcessSubCmd),
    /// Initialize vendoring from source
    Init(InitArgs),
    /// Workload performance analysis
    Workload(WorkloadArgs),
    /// List defined workloads
    WorkloadList,
    /// Create worktree for a workload
    WorkloadWorktree(WorkloadWorktreeArgs),
    /// Fetch from upstream
    FetchUpstream,
    /// Rebase onto upstream
    Rebase,
    /// Create version branches
    Releases,
    /// Show status
    Status,
    /// Full sync
    Sync,
    /// Generate patches
    Patch,
    /// Build dependency graph
    BuildGraph(BuildArgs),
    /// Analyze graph
    AnalyzeGraph(AnalyzeArgs),
    /// Visualize graph
    VisualizeGraph(VisualizeArgs),
    /// Partition graph
    PartitionGraph(PartitionArgs),
    /// Process crates
    ProcessCrates(ProcessCratesArgs),
    /// Process all crates from file
    ProcessAll(ProcessAllArgs),
    /// Run workflow
    RunWorkflow(WorkflowArgs),
    /// Generate report
    Report(ReportArgs),
    /// Create config file
    InitConfig,
    /// Onboard new repository
    Onboard(OnboardArgs),
    /// Edit workspace manifest
    Edit(EditArgs),
    /// Generate NUR flake from repos.json
    NurFlake(NurFlakeArgs),
    /// Check flake coverage for all target CBOR libs, fuzz tools, and test suites
    FlakeCheck(FlakeCheckArgs),
    /// Generate Lean4 formal verification model
    Lean4(Lean4Args),
    /// Split a Lean 4 mathlib-style project into per-declaration flakes
    SplitLean4(SplitLean4Args),
    /// Create a virtual workspace from a directory of crates
    CreateVirtualWorkspace(CreateVirtualWorkspaceArgs),
    /// Detect projects in a directory
    DetectProjects(DetectProjectsArgs),
    /// Analyze submodules
    AnalyzeSubmodules,
    /// Index crates for Nora with extended IPLD metadata
    NoraIndex(NoraIndexArgs),
    /// Split a file
    Split(SplitArgs),
    /// Ingest old submodules directory into the new vendormod registry
    Ingest(IngestArgs),
    /// Force-upgrade all crate dependencies to latest versions
    MemecacheUpgrade(MemecacheUpgradeArgs),
    /// Clean stale crate versions from the memecache
    MemecacheGc(MemecacheGcArgs),
    /// Scan index files (text lists, .gitmodules, parquet, plocate)
    ScanIndex(ScanIndexArgs),
    /// Fast change detection: snapshot and diff to find new/changed/deleted files
    ScanDelta(ScanDeltaArgs),
    /// Manage directories for cache warming
    Warm(WarmCmd),
}

#[derive(Parser, Debug)]
pub struct VendoringSubCmd {
    #[command(subcommand)]
    pub cmd: Option<VendoringCmd>,
}

#[derive(Subcommand, Debug)]
pub enum VendoringCmd {
    Init,
    FetchUpstream,
    Rebase,
    Releases,
    Status,
    Sync,
    Patch,
}

#[derive(Parser, Debug)]
pub struct GraphSubCmd {
    #[command(subcommand)]
    pub cmd: Option<GraphCmd>,
}

#[derive(Subcommand, Debug)]
pub enum GraphCmd {
    Build(BuildArgs),
    Analyze(AnalyzeArgs),
    Visualize(VisualizeArgs),
    Partition(PartitionArgs),
}

#[derive(Parser, Debug)]
pub struct ProcessSubCmd {
    #[command(subcommand)]
    pub cmd: Option<ProcessCmd>,
}

#[derive(Subcommand, Debug)]
pub enum ProcessCmd {
    Crates(ProcessCratesArgs),
    All(ProcessAllArgs),
    Workflow(WorkflowArgs),
}

#[derive(Parser, Debug)]
pub struct InitArgs {
    /// Source repository
    #[arg(long, default_value = ".")]
    pub source: PathBuf,
    /// Include dev deps
    #[arg(long)]
    pub include_dev: bool,
    /// Include build deps
    #[arg(long)]
    pub include_build: bool,
    /// Include optional deps
    #[arg(long)]
    pub include_optional: bool,
}

#[derive(Parser, Debug)]
pub struct BuildArgs {
    /// Workspace path
    #[arg(long)]
    pub workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./analysis")]
    pub output_dir: PathBuf,
    /// Include dev deps
    #[arg(long)]
    pub include_dev: bool,
    /// Include build deps
    #[arg(long)]
    pub include_build: bool,
    /// Expand features
    #[arg(long)]
    pub expand_features: bool,
}

#[derive(Parser, Debug)]
pub struct AnalyzeArgs {
    /// Input graph file
    #[arg(long)]
    pub input_path: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./analysis")]
    pub output_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkloadDef {
    pub name: String,
    pub path: String,
    pub layer1_count: usize,
    pub layer2_count: usize,
    pub description: String,
}



#[derive(Parser, Debug)]
pub struct Lean4Args {
    /// Input .service file(s) or directory
    pub inputs: Vec<PathBuf>,
    /// Output Lean4 model file
    #[arg(long, default_value = "./lean4_output/model.lean")]
    pub output: PathBuf,
}

#[derive(Parser, Debug)]
pub struct VisualizeArgs {
    /// Input graph file
    #[arg(long)]
    pub input_path: PathBuf,
    /// Output file
    #[arg(long)]
    pub output_path: PathBuf,
}

#[derive(Parser, Debug)]
pub struct PartitionArgs {
    /// Input graph file
    #[arg(long)]
    pub input_path: PathBuf,
    /// Partition count
    #[arg(long, default_value = "8")]
    pub partition_count: usize,
    /// Output directory
    #[arg(long, default_value = "./analysis/partitions")]
    pub output_dir: PathBuf,
    /// Algorithm
    #[arg(long, default_value = "kaminpar")]
    pub algorithm: String,
}

#[derive(Parser, Debug)]
pub struct ProcessCratesArgs {
    /// Workspace path
    #[arg(long)]
    pub workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./processed")]
    pub output_dir: PathBuf,
    /// Generate flakes
    #[arg(long)]
    pub generate_flakes: bool,
    /// Standalone compile
    #[arg(long)]
    pub compile_standalone: bool,
    /// Layered processing
    #[arg(long)]
    pub layered_processing: bool,
    /// Max parallel
    #[arg(long, default_value = "4")]
    pub max_parallel: usize,
}

#[derive(Parser, Debug)]
pub struct ProcessAllArgs {
    /// Input file with crate paths
    #[arg(long)]
    pub input_file: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./processed_all")]
    pub output_dir: PathBuf,
    /// Max parallel
    #[arg(long, default_value = "4")]
    pub max_parallel: usize,
}

#[derive(Parser, Debug)]
pub struct WorkflowArgs {
    /// Workspace path
    #[arg(long)]
    pub workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./output")]
    pub output_dir: PathBuf,
    /// Workflow type
    #[arg(long, default_value = "standard")]
    pub workflow_type: String,
}

#[derive(Parser, Debug)]
pub struct ReportArgs {
    /// Workspace path
    #[arg(long)]
    pub workspace_path: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./crate_report")]
    pub output_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct OnboardArgs {
    /// Git repo URL
    #[arg(long)]
    pub git_repo: Option<String>,
    /// Crate name
    #[arg(long)]
    pub crate_name: Option<String>,
    /// Local workspace
    #[arg(long)]
    pub workdir: Option<PathBuf>,
    /// Branch
    #[arg(long, default_value = "main")]
    pub branch: String,
    /// Workflow
    #[arg(long, default_value = "full_onboarding")]
    pub workflow: String,
    /// Output directory
    #[arg(long, default_value = "./workload/workspaces")]
    pub output_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct EditArgs {
    /// Sort dependencies
    #[arg(long)]
    pub sort: bool,
    /// Add missing
    #[arg(long)]
    pub add_missing: bool,
    /// Remove unused
    #[arg(long)]
    pub remove_unused: bool,
    /// Update versions
    #[arg(long)]
    pub update_versions: bool,
}

#[derive(Parser, Debug)]
pub struct NurFlakeArgs {
    /// Path to repos.json
    #[arg(long, default_value = "repos.json")]
    pub repos_json: PathBuf,

    /// Path to repos.json.lock
    #[arg(long, default_value = "repos.json.lock")]
    pub lock_json: PathBuf,

    /// Output path for generated flake.nix
    #[arg(long, default_value = "flake.nix")]
    pub output: PathBuf,

    /// Check existing flake.nix without overwriting
    #[arg(long)]
    pub check: bool,

    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct FlakeCheckArgs {
    /// Directory containing flake subdirectories
    #[arg(long, default_value = "flakes")]
    pub flakes_dir: PathBuf,
    /// Output as JSON
    #[arg(long)]
    pub json: bool,
}

#[derive(Parser, Debug)]
pub struct WorkloadArgs {
    /// Workspace path to analyze
    #[arg(long, default_value = ".")]
    pub workspace_path: PathBuf,
    /// Output format (json, text)
    #[arg(long, default_value = "text")]
    pub format: String,
}

/// Workload worktree creation arguments
#[derive(Parser, Debug)]
pub struct WorkloadWorktreeArgs {
    /// Workload name from list
    pub name: String,
    /// Branch name for worktree
    #[arg(long)]
    pub branch: Option<String>,
    /// Worktree output directory
    #[arg(long, default_value = "./worktrees")]
    pub output_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct SplitLean4Args {
    /// Path to mathlib source (directory containing Mathlib/)
    #[arg(long)]
    pub mathlib_src: PathBuf,

    /// Output directory for split flakes
    #[arg(long, default_value = "./mathlib-split")]
    pub output_dir: PathBuf,

    /// Branch to push to in the target repo
    #[arg(long, default_value = "feature/split")]
    pub branch: String,

    /// Path to the lean-split-tool split script
    #[arg(long, default_value = "/home/mdupont/projects/lean-split-tool/split-mathlib.sh")]
    pub split_tool: PathBuf,

    /// Dry-run: only print what would be executed
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Parser, Debug)]
pub struct CreateVirtualWorkspaceArgs {
    /// Path to the directory containing the crates
    #[arg(long)]
    pub input_dir: PathBuf,
    /// Path to the output directory for the virtual workspace
    #[arg(long)]
    pub output_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct DetectProjectsArgs {
    /// Path to the directory to scan for projects
    #[arg(long)]
    pub input_dir: PathBuf,
}

#[derive(Parser, Debug)]
pub struct SplitArgs {
    /// Path to the file to split
    #[arg(long)]
    pub input_file: PathBuf,
}

#[derive(Parser, Debug)]
pub struct NixBuildArgs {
    /// Path to directory containing Nix flakes
    pub flake_dir: PathBuf,
    /// Maximum number of parallel builds
    #[arg(long, default_value = "8")]
    pub max_parallel: usize,
    /// Maximum number of retries for failed builds
    #[arg(long, default_value = "2")]
    pub max_retries: usize,
    /// Timeout for individual builds in seconds
    #[arg(long, default_value = "3600")]
    pub timeout_seconds: u64,
    /// Output directory for build artifacts
    #[arg(long, default_value = "./nix_builds")]
    pub output_dir: PathBuf,
    /// Directory for build logs
    #[arg(long, default_value = "./build_logs")]
    pub log_dir: PathBuf,
    /// Path to workspace for dependency analysis
    #[arg(long)]
    pub workspace_path: Option<PathBuf>,
}

#[derive(Parser, Debug)]
pub struct IngestArgs {
    /// Path to old submodules directory to ingest
    #[arg(long)]
    pub source_dir: PathBuf,
    /// Path to .gitmodules file (default: source_dir/.gitmodules)
    #[arg(long)]
    pub gitmodules_path: Option<PathBuf>,
    /// Output directory for the new vendormod registry
    #[arg(long, default_value = "./vendormod-registry")]
    pub output_dir: PathBuf,
    /// Store crate metadata in shmem by CID
    #[arg(long)]
    pub shmem: bool,
    /// Shmem socket path
    #[arg(long, default_value = "@ipld_car_shmem")]
    pub shmem_socket: String,
    /// Only scan, don't write anything
    #[arg(long)]
    pub dry_run: bool,
    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
    /// Skip per-submodule git status/commit checks (much faster for large repos)
    #[arg(long)]
    pub no_git_status: bool,
    /// Max parallel submodules to process (0 = sequential)
    #[arg(long, default_value = "0")]
    pub max_parallel: usize,
}

#[derive(Parser, Debug)]
pub struct MemecacheUpgradeArgs {
    /// Path to workspace or submodules directory
    #[arg(long, default_value = ".")]
    pub workspace_path: PathBuf,
    /// Upgrade strategy: conservative (semver), aggressive (latest compatible), force (latest unconditional)
    #[arg(long, default_value = "conservative")]
    pub strategy: String,
    /// Upgrade specific crate only
    #[arg(long)]
    pub crate_name: Option<String>,
    /// Only show what would change
    #[arg(long)]
    pub dry_run: bool,
    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
    /// Upgrade all workspaces found recursively
    #[arg(long)]
    pub all: bool,
    /// Maximum parallel upgrades
    #[arg(long, default_value = "8")]
    pub max_parallel: usize,
}

#[derive(Parser, Debug)]
pub struct MemecacheGcArgs {
    /// Path to cargo registry cache
    #[arg(long, default_value = "~/.cargo/registry")]
    pub registry_path: PathBuf,
    /// Only show what would be deleted
    #[arg(long)]
    pub dry_run: bool,
    /// Also clean git checkouts
    #[arg(long)]
    pub clean_git: bool,
    /// Also clean target directories
    #[arg(long)]
    pub clean_targets: bool,
    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
}





#[derive(Parser, Debug)]
pub struct ScanIndexArgs {
    /// Text file list(s) to scan (one path per line)
    #[arg(long)]
    pub file_list: Vec<PathBuf>,
    /// Directory containing text file lists (e.g. ~/nix/index, ~/dasl/index)
    #[arg(long)]
    pub index_dir: Vec<PathBuf>,
    /// .gitmodules file(s) to parse and ingest
    #[arg(long)]
    pub gitmodules: Vec<PathBuf>,
    /// Find .gitmodules via plocate (pattern to search)
    #[arg(long)]
    pub plocate_pattern: Option<String>,
    /// Parquet file(s) to read as index
    #[arg(long)]
    pub parquet: Vec<PathBuf>,
    /// Base directory for resolving relative paths
    #[arg(long, default_value = ".")]
    pub base_dir: PathBuf,
    /// Output directory for scanned results
    #[arg(long, default_value = "./scan-results")]
    pub output_dir: PathBuf,
    /// Filter: only show files matching extension (e.g. .rs, .toml, .nix)
    #[arg(long)]
    pub ext: Vec<String>,
    /// Filter: only show files matching glob pattern
    #[arg(long)]
    pub glob: Vec<String>,
    /// Maximum number of lines to read per file list (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_lines: usize,
    /// Only scan, don't write results
    #[arg(long)]
    pub dry_run: bool,
    /// Store results in IPLD shmem (1MB cap per file)
    #[arg(long)]
    pub shmem: bool,
    /// Sample large files (>1MB) instead of skipping: head/tail/middle/conformal
    #[arg(long)]
    pub sample: bool,
    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
    /// Output format (json, text, summary)
    #[arg(long, default_value = "summary")]
    pub format: String,
}

// ============================================================
// scan-delta: Fast change detection via snapshot diffing
// ============================================================

#[derive(Parser, Debug)]
pub struct ScanDeltaArgs {
    /// Directory to scan for changes
    #[arg(long, default_value = ".")]
    pub dir: PathBuf,
    /// Snapshot name (stored in IPLD shmem)
    #[arg(long, default_value = "default")]
    pub snapshot: String,
    /// Max depth to walk (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_depth: usize,
    /// Skip directories matching these patterns (comma-separated)
    #[arg(long, default_value = ".git,target,node_modules,.cargo,build,dist,__pycache__")]
    pub skip_dirs: String,
    /// Only show new files (don't report changed/deleted)
    #[arg(long)]
    pub new_only: bool,
    /// Store new snapshot in IPLD shmem
    #[arg(long)]
    pub shmem: bool,
    /// Verbose output
    #[arg(long, short)]
    pub verbose: bool,
}
