//! # Main CLI Dispatcher
//!
//! Cargo-vendormod main entry point — all command logic runs in-process.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use cargo_vendormod::config::Config;
use cargo_vendormod::nur_flake::NurFlakeGenerator;
use cargo_vendormod::flake_check;
use cargo_vendormod::workload_processor::process_workload_recursive;
use cargo_vendormod::vendoring_cmds;
use std::process::Command;
use cargo_vendormod::global_dep_graph::{self, GlobalDependencyGraph};
use walkdir::WalkDir;
use toml;
use project_detect;
use git_config;
use git2;
use serde_ipld_dagcbor;
use cid::{Cid, codec::Codec};
use multihash::{Code, Multihash};
use std::io::{Read, Write};
use std::io::{Read, Write};

#[derive(Parser, Debug)]
#[command(name = "cargo-vendormod")]
#[command(version = "0.2.0")]
#[command(about = "Vendor git dependencies as submodules with local mirrors", long_about = None)]
struct MainArgs {
    /// Path to vendormod config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Enable verbose output
    #[arg(long, short)]
    verbose: bool,

    /// Root directory of the cargo project
    #[arg(long, default_value = ".")]
    root_dir: PathBuf,

    /// Submodules directory
    #[arg(long)]
    submodules_path: Option<PathBuf>,

    /// Mirrors directory
    #[arg(long)]
    mirrors_path: Option<PathBuf>,

    /// Vendor directory
    #[arg(long)]
    vendor_dir: Option<PathBuf>,

    /// Target branch
    #[arg(long)]
    target_branch: Option<String>,

    /// Create version branches
    #[arg(long)]
    create_version_branches: Option<bool>,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
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
    /// Split a monolithic crate into workspace sub-crates
    Split(SplitArgs),
}

#[derive(Parser, Debug)]
struct VendoringSubCmd {
    #[command(subcommand)]
    cmd: Option<VendoringCmd>,
}

#[derive(Subcommand, Debug)]
enum VendoringCmd {
    Init,
    FetchUpstream,
    Rebase,
    Releases,
    Status,
    Sync,
    Patch,
}

#[derive(Parser, Debug)]
struct GraphSubCmd {
    #[command(subcommand)]
    cmd: Option<GraphCmd>,
}

#[derive(Subcommand, Debug)]
enum GraphCmd {
    Build(BuildArgs),
    Analyze(AnalyzeArgs),
    Visualize(VisualizeArgs),
    Partition(PartitionArgs),
}

#[derive(Parser, Debug)]
struct ProcessSubCmd {
    #[command(subcommand)]
    cmd: Option<ProcessCmd>,
}

#[derive(Subcommand, Debug)]
enum ProcessCmd {
    Crates(ProcessCratesArgs),
    All(ProcessAllArgs),
    Workflow(WorkflowArgs),
}

#[derive(Parser, Debug)]
struct InitArgs {
    /// Source repository
    #[arg(long, default_value = ".")]
    source: PathBuf,
    /// Include dev deps
    #[arg(long)]
    include_dev: bool,
    /// Include build deps
    #[arg(long)]
    include_build: bool,
    /// Include optional deps
    #[arg(long)]
    include_optional: bool,
}

#[derive(Parser, Debug)]
struct BuildArgs {
    /// Workspace path
    #[arg(long)]
    workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./analysis")]
    output_dir: PathBuf,
    /// Include dev deps
    #[arg(long)]
    include_dev: bool,
    /// Include build deps
    #[arg(long)]
    include_build: bool,
    /// Expand features
    #[arg(long)]
    expand_features: bool,
}

#[derive(Parser, Debug)]
struct AnalyzeArgs {
    /// Input graph file
    #[arg(long)]
    input_path: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./analysis")]
    output_dir: PathBuf,
}

#[derive(Debug, Clone)]
struct WorkloadDef {
    name: String,
    path: String,
    layer1_count: usize,
    layer2_count: usize,
    description: String,
}

#[derive(Parser, Debug)]
struct Lean4Args {
    /// Input .service file(s) or directory
    inputs: Vec<PathBuf>,
    /// Output Lean4 model file
    #[arg(long, default_value = "./lean4_output/model.lean")]
    output: PathBuf,
}

#[derive(Parser, Debug)]
struct VisualizeArgs {
    /// Input graph file
    #[arg(long)]
    input_path: PathBuf,
    /// Output file
    #[arg(long)]
    output_path: PathBuf,
}

#[derive(Parser, Debug)]
struct PartitionArgs {
    /// Input graph file
    #[arg(long)]
    input_path: PathBuf,
    /// Partition count
    #[arg(long, default_value = "8")]
    partition_count: usize,
    /// Output directory
    #[arg(long, default_value = "./analysis/partitions")]
    output_dir: PathBuf,
    /// Algorithm
    #[arg(long, default_value = "kaminpar")]
    algorithm: String,
}

#[derive(Parser, Debug)]
struct ProcessCratesArgs {
    /// Workspace path
    #[arg(long)]
    workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./processed")]
    output_dir: PathBuf,
    /// Generate flakes
    #[arg(long)]
    generate_flakes: bool,
    /// Standalone compile
    #[arg(long)]
    compile_standalone: bool,
    /// Layered processing
    #[arg(long)]
    layered_processing: bool,
    /// Max parallel
    #[arg(long, default_value = "4")]
    max_parallel: usize,
}

#[derive(Parser, Debug)]
struct ProcessAllArgs {
    /// Input file with crate paths
    #[arg(long)]
    input_file: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./processed_all")]
    output_dir: PathBuf,
    /// Max parallel
    #[arg(long, default_value = "4")]
    max_parallel: usize,
}

#[derive(Parser, Debug)]
struct WorkflowArgs {
    /// Workspace path
    #[arg(long)]
    workspace_path: Option<PathBuf>,
    /// Output directory
    #[arg(long, default_value = "./output")]
    output_dir: PathBuf,
    /// Workflow type
    #[arg(long, default_value = "standard")]
    workflow_type: String,
}

#[derive(Parser, Debug)]
struct ReportArgs {
    /// Workspace path
    #[arg(long)]
    workspace_path: PathBuf,
    /// Output directory
    #[arg(long, default_value = "./crate_report")]
    output_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct OnboardArgs {
    /// Git repo URL
    #[arg(long)]
    git_repo: Option<String>,
    /// Crate name
    #[arg(long)]
    crate_name: Option<String>,
    /// Local workspace
    #[arg(long)]
    workdir: Option<PathBuf>,
    /// Branch
    #[arg(long, default_value = "main")]
    branch: String,
    /// Workflow
    #[arg(long, default_value = "full_onboarding")]
    workflow: String,
    /// Output directory
    #[arg(long, default_value = "./workload/workspaces")]
    output_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct EditArgs {
    /// Sort dependencies
    #[arg(long)]
    sort: bool,
    /// Add missing
    #[arg(long)]
    add_missing: bool,
    /// Remove unused
    #[arg(long)]
    remove_unused: bool,
    /// Update versions
    #[arg(long)]
    update_versions: bool,
}

#[derive(Parser, Debug)]
struct NurFlakeArgs {
    /// Path to repos.json
    #[arg(long, default_value = "repos.json")]
    repos_json: PathBuf,

    /// Path to repos.json.lock
    #[arg(long, default_value = "repos.json.lock")]
    lock_json: PathBuf,

    /// Output path for generated flake.nix
    #[arg(long, default_value = "flake.nix")]
    output: PathBuf,

    /// Check existing flake.nix without overwriting
    #[arg(long)]
    check: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Parser, Debug)]
struct FlakeCheckArgs {
    /// Directory containing flake subdirectories
    #[arg(long, default_value = "flakes")]
    flakes_dir: PathBuf,
    /// Output as JSON
    #[arg(long)]
    json: bool,
}

#[derive(Parser, Debug)]
struct WorkloadArgs {
    /// Workspace path to analyze
    #[arg(long, default_value = ".")]
    workspace_path: PathBuf,
    /// Output format (json, text)
    #[arg(long, default_value = "text")]
    format: String,
}

/// Workload worktree creation arguments
#[derive(Parser, Debug)]
struct WorkloadWorktreeArgs {
    /// Workload name from list
    name: String,
    /// Branch name for worktree
    #[arg(long)]
    branch: Option<String>,
    /// Worktree output directory
    #[arg(long, default_value = "./worktrees")]
    output_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct SplitLean4Args {
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
}

#[derive(Parser, Debug)]
struct CreateVirtualWorkspaceArgs {
    /// Path to the directory containing the crates
    #[arg(long)]
    input_dir: PathBuf,
    /// Path to the output directory for the virtual workspace
    #[arg(long)]
    output_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct DetectProjectsArgs {
    /// Path to the directory to scan for projects
    #[arg(long)]
    input_dir: PathBuf,
}

#[derive(Parser, Debug)]
struct NoraIndexArgs {
    /// Path to workspace root directory
    #[arg(long)]
    workspace_path: PathBuf,

    /// Shmem server socket path (default: @ipld_car_shmem)
    #[arg(long, default_value = "@ipld_car_shmem")]
    shmem_socket: String,

    /// Cache directory for CAR pages (default: /mnt/data1/dasl-cache)
    #[arg(long, default_value = "/mnt/data1/dasl-cache")]
    cache_dir: PathBuf,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

#[derive(Parser, Debug)]
struct SplitArgs {
    /// Path to the file to split
    #[arg(long)]
    input_file: PathBuf,
}

#[derive(Parser, Debug)]
struct NixBuildArgs {
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
}

#[derive(Parser, Debug)]
struct IngestArgs {
    /// Path to old submodules directory to ingest
    #[arg(long)]
    source_dir: PathBuf,
    /// Path to .gitmodules file (default: source_dir/.gitmodules)
    #[arg(long)]
    gitmodules_path: Option<PathBuf>,
    /// Output directory for the new vendormod registry
    #[arg(long, default_value = "./vendormod-registry")]
    output_dir: PathBuf,
    /// Store crate metadata in shmem by CID
    #[arg(long)]
    shmem: bool,
    /// Shmem socket path
    #[arg(long, default_value = "@ipld_car_shmem")]
    shmem_socket: String,
    /// Only scan, don't write anything
    #[arg(long)]
    dry_run: bool,
    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
    /// Skip per-submodule git status/commit checks (much faster for large repos)
    #[arg(long)]
    no_git_status: bool,
    /// Max parallel submodules to process (0 = sequential)
    #[arg(long, default_value = "0")]
    max_parallel: usize,
}

#[derive(Parser, Debug)]
struct MemecacheUpgradeArgs {
    /// Path to workspace or submodules directory
    #[arg(long, default_value = ".")]
    workspace_path: PathBuf,
    /// Upgrade strategy: conservative (semver), aggressive (latest compatible), force (latest unconditional)
    #[arg(long, default_value = "conservative")]
    strategy: String,
    /// Upgrade specific crate only
    #[arg(long)]
    crate_name: Option<String>,
    /// Only show what would change
    #[arg(long)]
    dry_run: bool,
    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
    /// Upgrade all workspaces found recursively
    #[arg(long)]
    all: bool,
    /// Maximum parallel upgrades
    #[arg(long, default_value = "8")]
    max_parallel: usize,
}

#[derive(Parser, Debug)]
struct MemecacheGcArgs {
    /// Path to cargo registry cache
    #[arg(long, default_value = "~/.cargo/registry")]
    registry_path: PathBuf,
    /// Only show what would be deleted
    #[arg(long)]
    dry_run: bool,
    /// Also clean git checkouts
    #[arg(long)]
    clean_git: bool,
    /// Also clean target directories
    #[arg(long)]
    clean_targets: bool,
    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}





#[derive(Parser, Debug)]
struct ScanIndexArgs {
    /// Text file list(s) to scan (one path per line)
    #[arg(long)]
    file_list: Vec<PathBuf>,
    /// Directory containing text file lists (e.g. ~/nix/index, ~/dasl/index)
    #[arg(long)]
    index_dir: Vec<PathBuf>,
    /// .gitmodules file(s) to parse and ingest
    #[arg(long)]
    gitmodules: Vec<PathBuf>,
    /// Find .gitmodules via plocate (pattern to search)
    #[arg(long)]
    plocate_pattern: Option<String>,
    /// Parquet file(s) to read as index
    #[arg(long)]
    parquet: Vec<PathBuf>,
    /// Base directory for resolving relative paths
    #[arg(long, default_value = ".")]
    base_dir: PathBuf,
    /// Output directory for scanned results
    #[arg(long, default_value = "./scan-results")]
    output_dir: PathBuf,
    /// Filter: only show files matching extension (e.g. .rs, .toml, .nix)
    #[arg(long)]
    ext: Vec<String>,
    /// Filter: only show files matching glob pattern
    #[arg(long)]
    glob: Vec<String>,
    /// Maximum number of lines to read per file list (0 = unlimited)
    #[arg(long, default_value = "0")]
    max_lines: usize,
    /// Only scan, don't write results
    #[arg(long)]
    dry_run: bool,
    /// Store results in IPLD shmem (1MB cap per file)
    #[arg(long)]
    shmem: bool,
    /// Sample large files (>1MB) instead of skipping: head/tail/middle/conformal
    #[arg(long)]
    sample: bool,
    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
    /// Output format (json, text, summary)
    #[arg(long, default_value = "summary")]
    format: String,
}

#[derive(Parser, Debug)]
struct SplitArgs {
    /// Path to the file to split
    #[arg(long)]
    input_file: PathBuf,
}










// ============================================================
// scan-delta: Fast change detection via snapshot diffing
// ============================================================

#[derive(Parser, Debug)]
struct ScanDeltaArgs {
    /// Directory to scan for changes
    #[arg(long, default_value = ".")]
    dir: PathBuf,
    /// Snapshot name (stored in IPLD shmem)
    #[arg(long, default_value = "default")]
    snapshot: String,
    /// Max depth to walk (0 = unlimited)
    #[arg(long, default_value = "0")]
    max_depth: usize,
    /// Skip directories matching these patterns (comma-separated)
    #[arg(long, default_value = ".git,target,node_modules,.cargo,build,dist,__pycache__")]
    skip_dirs: String,
    /// Only show new files (don't report changed/deleted)
    #[arg(long)]
    new_only: bool,
    /// Store new snapshot in IPLD shmem
    #[arg(long)]
    shmem: bool,
    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

/// Lightweight file fingerprint: path + size + mtime_secs + mtime_nanos
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
struct FileFingerprint {
    path: String,
    size: u64,
    mtime_secs: i64,
    mtime_nanos: i32,
    ext: String,
}

/// A snapshot of all file fingerprints at a point in time
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Snapshot {
    name: String,
    dir: String,
    timestamp: String,
    total_files: usize,
    total_size: u64,
    fingerprints: Vec<FileFingerprint>,
}

/// Delta between two snapshots
#[derive(Debug, serde::Serialize)]
struct Delta {
    snapshot_name: String,
    dir: String,
    new_files: Vec<FileFingerprint>,
    changed_files: Vec<(FileFingerprint, FileFingerprint)>,  // (old, new)
    deleted_files: Vec<FileFingerprint>,
    new_count: usize,
    changed_count: usize,
    deleted_count: usize,
    unchanged_count: usize,
    scan_duration_ms: u64,
}

fn handle_scan_delta(args: &ScanDeltaArgs) -> Result<()> {
    let start = std::time::Instant::now();
    let skip_dirs: Vec<&str> = args.skip_dirs.split(',').map(|s| s.trim()).collect();
    let max_depth = if args.max_depth == 0 { usize::MAX } else { args.max_depth };

    // 1. Load previous snapshot from IPLD shmem
    let snapshot_path = format!("vendormod/snapshots/{}", args.snapshot);
    let prev_snapshot = load_snapshot(&snapshot_path);

    if args.verbose {
        match &prev_snapshot {
            Some(s) => println!("Loaded previous snapshot: {} ({} files, {})",
                s.name, s.total_files, s.timestamp),
            None => println!("No previous snapshot found — creating baseline"),
        }
    }

    // 2. Walk the directory and build new fingerprints
    let mut new_fingerprints: Vec<FileFingerprint> = Vec::new();
    let mut total_size: u64 = 0;

    for entry in walkdir::WalkDir::new(&args.dir)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip hidden and excluded directories
            if let Some(name) = e.file_name().to_str() {
                if name.starts_with('.') { return false; }
                if skip_dirs.iter().any(|&skip| name == skip) { return false; }
            }
            true
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };

        if !entry.file_type().is_file() { continue; }

        let path = entry.path();
        let metadata = match std::fs::metadata(path) {
            Ok(m) => m,
            Err(_) => continue,
        };

        let mtime = metadata.modified().ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .unwrap_or_default();

        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        let fp = FileFingerprint {
            path: path.to_string_lossy().to_string(),
            size: metadata.len(),
            mtime_secs: mtime.as_secs() as i64,
            mtime_nanos: mtime.subsec_nanos() as i32,
            ext,
        };

        total_size += metadata.len();
        new_fingerprints.push(fp);
    }

    let scan_duration = start.elapsed();

    // 3. Compute delta
    let mut new_files = Vec::new();
    let mut changed_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut unchanged_count = 0usize;

    if let Some(prev) = &prev_snapshot {
        // Build a map from the previous snapshot
        let prev_map: HashMap<String, &FileFingerprint> = prev.fingerprints.iter()
            .map(|fp| (fp.path.clone(), fp))
            .collect();

        let new_map: HashMap<String, &FileFingerprint> = new_fingerprints.iter()
            .map(|fp| (fp.path.clone(), fp))
            .collect();

        // Find new and changed files
        for fp in &new_fingerprints {
            match prev_map.get(&fp.path) {
                None => new_files.push(fp.clone()),
                Some(old) if *old != *fp => changed_files.push(((*old).clone(), fp.clone())),
                Some(_) => unchanged_count += 1,
            }
        }

        // Find deleted files
        for fp in &prev.fingerprints {
            if !new_map.contains_key(&fp.path) {
                deleted_files.push(fp.clone());
            }
        }
    } else {
        // No previous snapshot — everything is "new"
        new_files = new_fingerprints.clone();
    }

    // 4. Print results
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Scan Delta: {}", args.dir.display());
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Snapshot:     {}", args.snapshot);
    println!("Scan time:    {:.1}s", scan_duration.as_secs_f64());
    println!("Total files:  {} ({:.1} MB)", new_fingerprints.len(), total_size as f64 / 1_048_576.0);
    println!();

    if !new_files.is_empty() {
        println!("NEW FILES ({}):", new_files.len());
        for fp in new_files.iter().take(50) {
            println!("  + {} ({:.1} KB, .{})", fp.path, fp.size as f64 / 1024.0, fp.ext);
        }
        if new_files.len() > 50 {
            println!("  ... and {} more", new_files.len() - 50);
        }
        println!();
    }

    if !args.new_only {
        if !changed_files.is_empty() {
            println!("CHANGED FILES ({}):", changed_files.len());
            for (old, new) in changed_files.iter().take(30) {
                let size_diff = new.size as i64 - old.size as i64;
                let size_sign = if size_diff >= 0 { "+" } else { "" };
                println!("  M {} ({}{} bytes, .{})", new.path, size_sign, size_diff, new.ext);
            }
            if changed_files.len() > 30 {
                println!("  ... and {} more", changed_files.len() - 30);
            }
            println!();
        }

        if !deleted_files.is_empty() {
            println!("DELETED FILES ({}):", deleted_files.len());
            for fp in deleted_files.iter().take(30) {
                println!("  - {} ({:.1} KB, .{})", fp.path, fp.size as f64 / 1024.0, fp.ext);
            }
            if deleted_files.len() > 30 {
                println!("  ... and {} more", deleted_files.len() - 30);
            }
            println!();
        }
    }

    println!("Summary: {} new, {} changed, {} deleted, {} unchanged",
        new_files.len(), changed_files.len(), deleted_files.len(), unchanged_count);

    // 5. Store new snapshot in shmem
    if args.shmem {
        let snapshot = Snapshot {
            name: args.snapshot.clone(),
            dir: args.dir.to_string_lossy().to_string(),
            timestamp: chrono_now(),
            total_files: new_fingerprints.len(),
            total_size,
            fingerprints: new_fingerprints,
        };

        // Store in chunks if needed (1MB cap)
        let snapshot_json = serde_json::to_vec(&snapshot)?;
        if snapshot_json.len() <= SHMEM_MAX_SIZE {
            let _ = shmem_put("", &snapshot_path, &format!("Snapshot {} ({} files)", args.snapshot, snapshot.total_files), &snapshot_json);
        } else {
            // Store fingerprints in chunks, metadata separately
            let meta = serde_json::json!({
                "name": snapshot.name,
                "dir": snapshot.dir,
                "timestamp": snapshot.timestamp,
                "total_files": snapshot.total_files,
                "total_size": snapshot.total_size,
            });
            let meta_bytes = serde_json::to_vec(&meta)?;
            let _ = shmem_put("", &format!("{}.meta", snapshot_path), &format!("Snapshot {} meta", args.snapshot), &meta_bytes);

            for (chunk_idx, chunk) in snapshot.fingerprints.chunks(5000).enumerate() {
                let chunk_bytes = serde_json::to_vec(chunk)?;
                let _ = shmem_put("", &format!("{}.fps-{}", snapshot_path, chunk_idx),
                    &format!("Snapshot {} fingerprints chunk {}", args.snapshot, chunk_idx), &chunk_bytes);
            }
        }

        println!("Snapshot stored in IPLD shmem: {}", snapshot_path);
    }

    // 6. Store delta in shmem too
    if args.shmem && prev_snapshot.is_some() {
        let delta = Delta {
            snapshot_name: args.snapshot.clone(),
            dir: args.dir.to_string_lossy().to_string(),
            new_files,
            changed_files,
            deleted_files,
            new_count: 0,
            changed_count: 0,
            deleted_count: 0,
            unchanged_count,
            scan_duration_ms: scan_duration.as_millis() as u64,
        };
        let delta_json = serde_json::to_vec(&delta)?;
        let delta_path = format!("vendormod/deltas/{}", args.snapshot);
        let _ = shmem_put("", &delta_path, &format!("Delta for {}", args.snapshot), &delta_json);
    }

    Ok(())
}

/// Load a snapshot from IPLD shmem
fn load_snapshot(path: &str) -> Option<Snapshot> {
    let output = std::process::Command::new(IPLD_MEMORY_BIN)
        .args(["get", path])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout == "not_found" {
        return None;
    }

    // Try to parse as full snapshot first
    if let Ok(snap) = serde_json::from_str::<Snapshot>(&stdout) {
        return Some(snap);
    }

    // Otherwise, try to reconstruct from meta + fingerprint chunks
    let meta_output = std::process::Command::new(IPLD_MEMORY_BIN)
        .args(["get", &format!("{}.meta", path)])
        .output()
        .ok()?;

    if !meta_output.status.success() {
        return None;
    }

    let meta_stdout = String::from_utf8_lossy(&meta_output.stdout);
    if meta_stdout == "not_found" {
        return None;
    }

    let meta: serde_json::Value = serde_json::from_str(&meta_stdout).ok()?;

    let mut fingerprints = Vec::new();
    for chunk_idx in 0..100 {
        let chunk_output = std::process::Command::new(IPLD_MEMORY_BIN)
            .args(["get", &format!("{}.fps-{}", path, chunk_idx)])
            .output()
            .ok()?;

        if !chunk_output.status.success() {
            break;
        }

        let chunk_stdout = String::from_utf8_lossy(&chunk_output.stdout);
        if chunk_stdout == "not_found" {
            break;
        }

        let chunk_fps: Vec<FileFingerprint> = match serde_json::from_str(&chunk_stdout) {
            Ok(fps) => fps,
            Err(_) => break,
        };

        if chunk_fps.is_empty() {
            break;
        }

        fingerprints.extend(chunk_fps);
    }

    Some(Snapshot {
        name: meta.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        dir: meta.get("dir").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        timestamp: meta.get("timestamp").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        total_files: meta.get("total_files").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
        total_size: meta.get("total_size").and_then(|v| v.as_u64()).unwrap_or(0),
        fingerprints,
    })
}

/// Simple timestamp
fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
    }
}

fn main() -> Result<()> {
    let args = MainArgs::parse();

    // Load configuration
    let config = match cargo_vendormod::config::Config::load(args.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not load config: {}", e);
            cargo_vendormod::config::Config::default()
        }
    };

    match args.command {
        // Vendoring sub-commands
        Some(Commands::Vendoring(ref vcmd)) => {
            match &vcmd.cmd {
                Some(VendoringCmd::Init) => handle_init(&args, &config, None),
                Some(VendoringCmd::FetchUpstream) => handle_fetch_upstream(&args, &config),
                Some(VendoringCmd::Rebase) => handle_rebase(&args, &config),
                Some(VendoringCmd::Releases) => handle_releases(&args, &config),
                Some(VendoringCmd::Status) => handle_status(&args, &config),
                Some(VendoringCmd::Sync) => handle_sync(&args, &config),
                Some(VendoringCmd::Patch) => handle_patch(&args, &config),
                None => handle_init(&args, &config, None),
            }
        }
        Some(Commands::Graph(_gcmd)) => {
            println!("Graph commands not yet implemented in-process");
            Ok(())
        }
        Some(Commands::Process(_pcmd)) => {
            println!("Process commands not yet implemented in-process");
            Ok(())
        }

        // Inline commands — call vendoring_cmds directly
        Some(Commands::Init(ref a)) => handle_init(&args, &config, Some(&a.source)),
        Some(Commands::FetchUpstream) => handle_fetch_upstream(&args, &config),
        Some(Commands::Rebase) => handle_rebase(&args, &config),
        Some(Commands::Releases) => handle_releases(&args, &config),
        Some(Commands::Status) => handle_status(&args, &config),
        Some(Commands::Sync) => handle_sync(&args, &config),
        Some(Commands::Patch) => handle_patch(&args, &config),

        Some(Commands::BuildGraph(_)) => {
            println!("BuildGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::AnalyzeGraph(_)) => {
            println!("AnalyzeGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::VisualizeGraph(_)) => {
            println!("VisualizeGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::PartitionGraph(_)) => {
            println!("PartitionGraph not yet implemented in-process");
            Ok(())
        }

        Some(Commands::ProcessCrates(_)) => {
            println!("ProcessCrates not yet implemented in-process");
            Ok(())
        }
        Some(Commands::ProcessAll(_)) => {
            println!("ProcessAll not yet implemented in-process");
            Ok(())
        }
        Some(Commands::RunWorkflow(_)) => {
            println!("RunWorkflow not yet implemented in-process");
            Ok(())
        }

        Some(Commands::Report(_)) => {
            println!("Report not yet implemented in-process");
            Ok(())
        }

        Some(Commands::InitConfig) => {
            let sample = cargo_vendormod::config::generate_sample_config();
            println!("{}", sample);
            println!("\nCopy the above to ./vendormod.toml or ~/.config/cargo-vendormod/config.toml");
            Ok(())
        }

        Some(Commands::Onboard(_)) => {
            println!("Onboarding not yet implemented in standalone binary");
            Ok(())
        }

        Some(Commands::Edit(_)) => {
            println!("Edit not yet implemented in standalone binary");
            Ok(())
        }

        Some(Commands::Workload(args)) => {
            run_workload_analysis(&args.workspace_path, &args.format)
        }

        Some(Commands::WorkloadList) => {
            let workloads = get_defined_workloads();
            println!("═══════════════════════════════════════════════════════════════════");
            println!("  Available Workloads");
            println!("═══════════════════════════════════════════════════════════════════\n");
            for w in &workloads {
                println!("📦 {} ({})", w.name, w.path.split('/').last().unwrap_or(&w.name));
                println!("   {} (L1: {} crates, L2: {} crates)", 
                    w.description, w.layer1_count, w.layer2_count);
                println!();
            }
            println!("Run: cargo-vendormod workload --workspace-path /path/to/workload");
            Ok(())
        }

        Some(Commands::WorkloadWorktree(args)) => {
            run_workload_worktree(&args.name, args.branch.as_deref(), &args.output_dir)
        }

        Some(Commands::NurFlake(args)) => {
            let generator = cargo_vendormod::nur_flake::NurFlakeGenerator::new(
                args.repos_json.clone(),
                args.lock_json.clone(),
                args.output.clone(),
            );

            if args.check {
                let is_up_to_date = generator
                    .check()
                    .context("Failed to check flake.nix status")?;
                if is_up_to_date {
                    println!("\u{2713} {} is up to date", args.output.display());
                } else {
                    eprintln!("\u{2717} {} is out of date \u{2014} regenerate needed", args.output.display());
                    eprintln!("  Run: cargo-vendormod nur-flake --repos-json {} --lock-json {} --output {}",
                        args.repos_json.display(),
                        args.lock_json.display(),
                        args.output.display());
                    std::process::exit(1);
                }
            } else {
                let repo_count = generator
                    .generate()
                    .context("Failed to generate flake.nix")?;
                println!("\u{2713} Generated {} with {} repos", args.output.display(), repo_count);
                if args.verbose {
                    eprintln!("  Run: nix flake check --impure  # to validate");
                }
            }

            Ok(())
        }

        Some(Commands::SplitLean4(args)) => {
            eprintln!("[info] SplitLean4: src={} out={} branch={}",
                args.mathlib_src.display(), args.output_dir.display(), args.branch);
            if args.dry_run {
                eprintln!("[dry-run] Would run: {} from {}", args.split_tool.display(), args.mathlib_src.display());
            } else {
                let status = std::process::Command::new(&args.split_tool)
                    .current_dir(&args.mathlib_src)
                    .arg(&args.output_dir)
                    .arg(&args.branch)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("lean-split-tool failed with status {}", status);
                }
            }
            Ok(())
        }

        Some(Commands::Lean4(args)) => {
            eprintln!("[info] Lean4 model generation: inputs={:?} output={}", args.inputs, args.output.display());
            Ok(())
        }

        Some(Commands::FlakeCheck(args)) => {
            let report = cargo_vendormod::flake_check::check_flake_coverage(&args.flakes_dir)?;
            if args.json {
                cargo_vendormod::flake_check::print_json(&report)?;
            } else {
                cargo_vendormod::flake_check::print_report(&report);
            }
            Ok(())
        }

        Some(Commands::Ingest(args)) => {
            handle_ingest(&args)
        }

        Some(Commands::MemecacheUpgrade(args)) => {
            handle_memecache_upgrade(&args)
        }

        Some(Commands::MemecacheGc(args)) => {
            handle_memecache_gc(&args)
        }

        Some(Commands::Split(args)) => {
            println!("Split not yet implemented: {}", args.input_file.display());
            Ok(())
        }

        Some(Commands::ScanIndex(args)) => {
            handle_scan_index(&args)
        }
        Some(Commands::ScanDelta(args)) => {
            handle_scan_delta(&args)
        }

        None => {
            // Show help
            println!("Cargo-vendormod - Git submodule vendoring for Cargo");
            println!("\nUsage: cargo-vendormod <command>");
            println!("\nCommands:");
            println!("  vendoring    - Git submodule operations");
            println!("  graph        - Dependency graph analysis");
            println!("  process      - Crate processing");
            println!("  workload     - Workload performance analysis");
            println!("  init-config  - Generate sample config");
            println!("  nur-flake    - Generate flake.nix for NUR workspace");
            println!("  flake-check  - Check flake coverage for CBOR libs, fuzz, tests");
            Ok(())
        }
    }
}

fn get_defined_workloads() -> Vec<WorkloadDef> {
    vec![
        WorkloadDef {
            name: "cargo2nix".to_string(),
            path: "/home/mdupont/nix/vendor/rust/cargo2nix".to_string(),
            layer1_count: 597,
            layer2_count: 0,
            description: "Cargo2nix 597-submodule workspace".to_string(),
        },
        WorkloadDef {
            name: "dasl".to_string(),
            path: "/home/mdupont/dasl".to_string(),
            layer1_count: 2993,
            layer2_count: 0,
            description: "DASL 2993-submodule IPLD ecosystem".to_string(),
        },
        WorkloadDef {
            name: "erdfa".to_string(),
            path: "/home/mdupont/git/erdfa-plugins".to_string(),
            layer1_count: 25,
            layer2_count: 170,
            description: "Escaped-RDFa plugin workspace (~170 Cargo.toml)".to_string(),
        },
    ]
}

fn run_workload_worktree(name: &str, _branch: Option<&str>, output_dir: &PathBuf) -> Result<()> {
    println!("🎯 Creating worktree for workload: {}", name);
    
    // Find the workload by name
    let workloads = get_defined_workloads();
    if let Some(workload) = workloads.iter().find(|w| w.name == name) {
        let worktree_path = output_dir.join(&workload.name);
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&worktree_path)?;
        
        // Create worktree using git
        let output = std::process::Command::new("git")
            .args(&["worktree", "add", worktree_path.to_str().unwrap()])
            .output()?;
        
        if output.status.success() {
            println!("✅ Worktree created at: {}", worktree_path.display());
        } else {
            eprintln!("❌ Failed to create worktree: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        eprintln!("❌ Unknown workload: {}", name);
        println!("Available workloads:");
        for w in workloads {
            println!("  - {}", w.name);
        }
    }
    
    Ok(())
}

fn run_workload_analysis(path: &PathBuf, format: &str) -> Result<()> {
    use cargo_vendormod::workload_processor::process_workload_recursive;
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Workload Performance Report");
    println!("═══════════════════════════════════════════════════════════════════\n");
    
    let metrics = process_workload_recursive(path)?;
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&metrics)?;
            println!("{}", json);
        }
        _ => {
            println!("═══════════════════════════════════════════════════════════════════");
            println!("  Performance Metrics");
            println!("═══════════════════════════════════════════════════════════════════");
            println!("Total git repositories: {}", metrics.total_repos);
            println!("Total Cargo.toml files: {}", metrics.total_cargoTOMs);
            println!("Processing time: {}ms", metrics.elapsed_ms);
            println!("Repositories per second: {:.2}", metrics.repos_per_second);
        }
    }
    Ok(())
}

// ── Vendoring handler functions (in-process) ──────────────────────────────

fn handle_init(args: &MainArgs, config: &cargo_vendormod::config::Config, source: Option<&PathBuf>) -> Result<()> {
    let source_repo = source.cloned()
        .or_else(|| args.root_dir.canonicalize().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    cargo_vendormod::vendoring_cmds::cmd_init(&source_repo, &sub_path, &mir_path, args.dry_run)
}

fn handle_fetch_upstream(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_fetch_upstream(
        &sub_path, &mir_path, git_exe, args.dry_run, args.verbose, config.default_threads,
    )
}

fn handle_rebase(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_rebase(
        &sub_path, &config.target_branch, git_exe, args.dry_run, args.verbose,
    )
}

fn handle_releases(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_releases(
        &sub_path, &mir_path, git_exe, &config.version_branch_format,
        config.create_version_branches, args.dry_run,
    )
}

fn handle_status(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_status(&sub_path, git_exe)
}

fn handle_sync(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_sync(
        &sub_path, &mir_path, &config.target_branch, git_exe,
        args.dry_run, args.verbose, config.default_threads,
    )
}

fn handle_patch(args: &MainArgs, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    cargo_vendormod::vendoring_cmds::cmd_patch(&sub_path)
}

// ── Ingest: read old submodules directory and take control ─────────────

fn handle_ingest(args: &IngestArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Ingest: Reading old submodules");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Source:      {}", args.source_dir.display());
    println!("Output:      {}", args.output_dir.display());
    println!("Shmem:       {}", if args.shmem { "enabled" } else { "disabled" });
    println!("Dry run:     {}", args.dry_run);
    println!();

    // 1. Parse .gitmodules using git-config parser (handles git config format, not strict TOML)
    let gitmodules_path = args.gitmodules_path.clone()
        .unwrap_or_else(|| args.source_dir.join(".gitmodules"));

    if !gitmodules_path.exists() {
        anyhow::bail!(".gitmodules not found at {}", gitmodules_path.display());
    }

    let gitmodules_raw = std::fs::read_to_string(&gitmodules_path)
        .context("Failed to read .gitmodules")?;

    // Parse using git-config crate (handles submodule "path/with/slashes" correctly)
    let git_config: git_config::File = gitmodules_raw.as_str().try_into()
        .context("Failed to parse .gitmodules as git config")?;

    // Collect all submodule sections
    let mut submodule_entries: Vec<(String, String, String, Option<String>)> = Vec::new(); // (name, path, url, branch)
    for section in git_config.sections() {
        let header = section.header();
        let section_name = header.name();
        if section_name != "submodule" {
            continue;
        }
        let sub_name = header.subsection_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let path_val = git_config.string("submodule", Some(sub_name.as_str()), "path")
            .unwrap_or_default()
            .to_string();
        let url_val = git_config.string("submodule", Some(sub_name.as_str()), "url")
            .unwrap_or_default()
            .to_string();
        let branch_val = git_config.string("submodule", Some(sub_name.as_str()), "branch")
            .map(|s| s.to_string());

        if !path_val.is_empty() && !url_val.is_empty() {
            submodule_entries.push((sub_name.clone(), path_val, url_val, branch_val));
        }
    }

    println!("Found {} registered submodules in .gitmodules", submodule_entries.len());

    // 2. Walk each submodule and collect state
    let mut registered: Vec<IngestedSubmodule> = Vec::new();
    let mut dirty_count = 0;
    let mut missing_count = 0;
    let mut clean_count = 0;
    let mut has_cargo_toml = 0;
    let mut has_flake_nix = 0;

    for (name, path_val, url_val, branch_val) in &submodule_entries {
        let sub_path = args.source_dir.join(path_val);
        let exists = sub_path.exists() && sub_path.is_dir();

        let git_status = if args.no_git_status {
            // Fast path: skip git status check, just check if dir exists
            if exists {
                clean_count += 1;
                "unknown (skipped)".to_string()
            } else {
                missing_count += 1;
                "missing".to_string()
            }
        } else if exists {
            let output = std::process::Command::new("git")
                .args(["-C", sub_path.to_str().unwrap_or(""), "status", "--porcelain"])
                .output()
                .ok();
            match output {
                Some(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    if stdout.trim().is_empty() {
                        clean_count += 1;
                        "clean".to_string()
                    } else {
                        dirty_count += 1;
                        format!("dirty ({} changes)", stdout.lines().count())
                    }
                }
                _ => {
                    dirty_count += 1;
                    "unknown".to_string()
                }
            }
        } else {
            missing_count += 1;
            "missing".to_string()
        };

        let version = if exists {
            std::fs::read_to_string(sub_path.join("Cargo.toml"))
                .ok()
                .and_then(|c| c.parse::<toml::Value>().ok())
                .and_then(|doc| {
                    doc.get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "N/A".to_string()
        };

        let has_cargo = exists && sub_path.join("Cargo.toml").exists();
        let has_flake = exists && sub_path.join("flake.nix").exists();

        if has_cargo { has_cargo_toml += 1; }
        if has_flake { has_flake_nix += 1; }

        let current_commit = if args.no_git_status || !exists {
            "N/A".to_string()
        } else {
            std::process::Command::new("git")
                .args(["-C", sub_path.to_str().unwrap_or(""), "rev-parse", "HEAD"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        };

        let ingested = IngestedSubmodule {
            name: name.clone(),
            path: path_val.clone(),
            url: url_val.clone(),
            branch: branch_val.clone(),
            version,
            git_status,
            current_commit,
            has_cargo_toml: has_cargo,
            has_flake_nix: has_flake,
        };

        if args.verbose {
            println!("  {} {} ({}) [{}]", 
                if ingested.has_cargo_toml { "📦" } else { "📁" },
                ingested.name, ingested.version, ingested.git_status);
        }

        registered.push(ingested);
    }

    // 3. Report summary
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Ingest Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Total submodules:    {}", registered.len());
    println!("Clean:               {}", clean_count);
    println!("Dirty:               {}", dirty_count);
    println!("Missing:             {}", missing_count);
    println!("Have Cargo.toml:     {}", has_cargo_toml);
    println!("Have flake.nix:      {}", has_flake_nix);
    println!();

    // 4. Write vendormod-registry.json
    if !args.dry_run {
        std::fs::create_dir_all(&args.output_dir)
            .context("Failed to create output directory")?;

        let registry = serde_json::json!({
            "version": "0.2.0",
            "source_dir": args.source_dir.to_string_lossy().to_string(),
            "total_submodules": registered.len(),
            "clean_count": clean_count,
            "dirty_count": dirty_count,
            "missing_count": missing_count,
            "has_cargo_toml": has_cargo_toml,
            "has_flake_nix": has_flake_nix,
            "submodules": registered,
        });

        let registry_path = args.output_dir.join("vendormod-registry.json");
        let json = serde_json::to_string_pretty(&registry)?;
        std::fs::write(&registry_path, &json)?;
        println!("Written registry to {}", registry_path.display());

        // 5. Write vendormod.lock (CID-indexed)
        let lock_path = args.output_dir.join("vendormod.lock");
        let mut lock_content = String::new();
        lock_content.push_str("# vendormod.lock — CID-indexed submodule registry\n");
        lock_content.push_str("# Generated by cargo-vendormod ingest\n");
        lock_content.push_str(&format!("# Total: {} submodules\n\n", registered.len()));

        for sm in &registered {
            let cid = compute_cid(&format!("{}:{}:{}", sm.url, sm.branch.as_deref().unwrap_or("main"), sm.current_commit));
            lock_content.push_str(&format!("{} {} {} {} {}\n",
                cid, sm.name, sm.version, sm.git_status, sm.url));
        }

        std::fs::write(&lock_path, &lock_content)?;
        println!("Written lock file to {}", lock_path.display());
    }

    // 6. Store in shmem if requested
    if args.shmem && !args.dry_run {
        println!();
        println!("Storing submodule metadata in shmem...");
        for sm in &registered {
            if sm.has_cargo_toml {
                let path = format!("vendormod/submodules/{}/metadata", sm.name);
                let description = format!("{} {} ({} bytes)", sm.name, sm.version, 0);
                // Store the Cargo.toml content in shmem
                let cargo_toml_path = args.source_dir.join(&sm.path).join("Cargo.toml");
                if let Ok(content) = std::fs::read(&cargo_toml_path) {
                    let _ = shmem_put(&args.shmem_socket, &path, &description, &content);
                    if args.verbose {
                        println!("  Stored {} ({} bytes)", sm.name, content.len());
                    }
                }
            }
        }
    }

    println!();
    println!("Ingest complete. {} submodules registered.", registered.len());
    Ok(())
}

/// Compute a simple CID-like hash for a string
fn compute_cid(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    format!("bafyri{}", hex::encode(hash)[..56].to_string())
}

/// Store a block in the IPLD CAR shmem
/// Max file size to store in shmem (1MB)
const SHMEM_MAX_SIZE: usize = 1_048_576;

/// Full path to letta-ipld-memory binary
const IPLD_MEMORY_BIN: &str = "/home/mdupont/dasl/ipld-car-ipc-shmem-linux/target/release/letta-ipld-memory";

fn shmem_put(_socket: &str, path: &str, description: &str, data: &[u8]) -> Result<()> {
    // Enforce 1MB cap
    if data.len() > SHMEM_MAX_SIZE {
        eprintln!("[shmem] Skipping {} ({} bytes > 1MB cap)", path, data.len());
        return Ok(());
    }

    // Call letta-ipld-memory put via stdin pipe
    let mut child = std::process::Command::new(IPLD_MEMORY_BIN)
        .arg("put")
        .arg(path)
        .arg(description)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn letta-ipld-memory put")?;

    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data)?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        eprintln!("[shmem] Failed to store {}: {}", path, String::from_utf8_lossy(&output.stderr));
    } else {
        let cid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cid.is_empty() {
            eprintln!("[shmem] Stored {} ({} bytes) -> CID {}", path, data.len(), cid);
        }
    }
    Ok(())
}

// ============================================================
// File Sampler — head, tail, middle, conformal field
// ============================================================

/// Sample lines from a large file for storage under the 1MB cap.
/// Strategy: head + tail + middle + conformal (evenly-spaced intervals)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct FileSample {
    /// Original file path
    path: String,
    /// Original file size in bytes
    original_size: u64,
    /// Original line count
    original_lines: usize,
    /// Number of head lines sampled
    head_lines: usize,
    /// Number of tail lines sampled
    tail_lines: usize,
    /// Number of middle lines sampled
    middle_lines: usize,
    /// Number of conformal lines sampled (evenly-spaced)
    conformal_lines: usize,
    /// Total lines in this sample
    sample_lines: usize,
    /// Sample size in bytes
    sample_size: usize,
    /// Byte-level entropy of the sample
    entropy: f64,
    /// Hecke spectral score (simplified)
    hecke_score: f64,
    /// Detected CIDs in sample (count)
    detected_cids: usize,
    /// The sampled content
    content: String,
}

/// Default sample sizes
const SAMPLE_HEAD_LINES: usize = 100;
const SAMPLE_TAIL_LINES: usize = 100;
const SAMPLE_MIDDLE_LINES: usize = 100;
const SAMPLE_CONFORMAL_LINES: usize = 200;

/// Sample a large file: head, tail, middle, conformal field
fn sample_file(path: &Path, max_sample_bytes: usize) -> Result<FileSample> {
    let metadata = std::fs::metadata(path)?;
    let original_size = metadata.len();

    // Read the file with lossy UTF-8
    let raw = std::fs::read(path)?;
    let content = String::from_utf8_lossy(&raw);
    let all_lines: Vec<&str> = content.lines().collect();
    let original_lines = all_lines.len();

    if original_lines == 0 {
        return Ok(FileSample {
            path: path.to_string_lossy().to_string(),
            original_size,
            original_lines: 0,
            head_lines: 0,
            tail_lines: 0,
            middle_lines: 0,
            conformal_lines: 0,
            sample_lines: 0,
            sample_size: 0,
            entropy: 0.0,
            hecke_score: 0.0,
            detected_cids: 0,
            content: String::new(),
        });
    }

    let mut sampled: Vec<(usize, &str)> = Vec::new(); // (line_number, line_text)
    let mut seen_lines: HashSet<usize> = HashSet::new();

    // 1. Head
    let head_n = SAMPLE_HEAD_LINES.min(original_lines);
    for i in 0..head_n {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 2. Tail
    let tail_n = SAMPLE_TAIL_LINES.min(original_lines);
    let tail_start = original_lines.saturating_sub(tail_n);
    for i in tail_start..original_lines {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 3. Middle (lines around the midpoint)
    let mid = original_lines / 2;
    let middle_n = SAMPLE_MIDDLE_LINES.min(original_lines);
    let middle_start = mid.saturating_sub(middle_n / 2);
    let middle_end = (mid + middle_n / 2).min(original_lines);
    for i in middle_start..middle_end {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 4. Conformal field — evenly-spaced lines across the entire file
    //    This is the "deep_scanner conformal" sampling: uniform intervals
    //    that preserve the file's spectral structure
    let conformal_n = SAMPLE_CONFORMAL_LINES.min(original_lines);
    if original_lines > 1 && conformal_n > 0 {
        let step = original_lines as f64 / conformal_n as f64;
        for k in 0..conformal_n {
            let i = (k as f64 * step) as usize;
            let i = i.min(original_lines - 1);
            if !seen_lines.contains(&i) {
                sampled.push((i, all_lines[i]));
                seen_lines.insert(i);
            }
        }
    }

    // Sort by line number for readability
    sampled.sort_by_key(|(i, _)| *i);

    // Build the sample content with line number markers
    let mut sample_content = String::new();
    sample_content.push_str(&format!("# FileSample: {} ({} bytes, {} lines)\n",
        path.to_string_lossy(), original_size, original_lines));
    sample_content.push_str(&format!("# head={} tail={} middle={} conformal={}\n",
        head_n, tail_n, middle_n, conformal_n));
    sample_content.push_str("# ---\n");

    for (line_num, line) in &sampled {
        // Truncate lines > 500 chars to keep sample small
        let truncated = if line.len() > 500 {
            format!("{}...[truncated, {} chars]", &line[..500], line.len())
        } else {
            line.to_string()
        };
        sample_content.push_str(&format!("{}:{}\n", line_num + 1, truncated));
    }

    // Trim to max_sample_bytes
    if sample_content.len() > max_sample_bytes {
        sample_content.truncate(max_sample_bytes);
        // Find last complete line
        if let Some(pos) = sample_content.rfind('\n') {
            sample_content.truncate(pos + 1);
        }
        sample_content.push_str("# [SAMPLE TRUNCATED TO FIT CAP]\n");
    }

    let sample_size = sample_content.len();
    let sample_lines_actual = sampled.len();

    // Compute entropy of the sample
    let entropy = compute_entropy(sample_content.as_bytes());

    // Simplified Hecke score: spectral density of line lengths
    let hecke_score = compute_hecke_score(&sampled);

    // Count potential CIDs (bafyrei, bafkrei, Qm prefixes)
    let detected_cids = count_cids(&sample_content);

    Ok(FileSample {
        path: path.to_string_lossy().to_string(),
        original_size,
        original_lines,
        head_lines: head_n,
        tail_lines: tail_n,
        middle_lines: middle_n,
        conformal_lines: conformal_n,
        sample_lines: sample_lines_actual,
        sample_size,
        entropy,
        hecke_score,
        detected_cids,
        content: sample_content,
    })
}

/// Compute Shannon entropy of a byte slice
fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let total = data.len() as f64;
    let mut entropy = 0.0;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Simplified Hecke spectral score: variance of line length differences
/// (captures structural rhythm of the file)
fn compute_hecke_score(lines: &[(usize, &str)]) -> f64 {
    if lines.len() < 3 {
        return 0.0;
    }
    let lengths: Vec<usize> = lines.iter().map(|(_, l)| l.len()).collect();
    let diffs: Vec<f64> = lengths.windows(2).map(|w| (w[1] as f64 - w[0] as f64).abs()).collect();
    if diffs.is_empty() {
        return 0.0;
    }
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    let variance = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / diffs.len() as f64;
    // Hecke-like spectral score: sqrt(variance) * ln(lines)
    variance.sqrt() * (lines.len() as f64).ln().max(1.0)
}

/// Count potential CIDs in text (bafyrei, bafkrei, Qm prefixes)
fn count_cids(text: &str) -> usize {
    let mut count = 0;
    for line in text.lines() {
        if line.contains("bafyrei") || line.contains("bafkrei") || line.contains("Qm") {
            count += 1;
        }
    }
    count
}

/// Ingested submodule record
#[derive(Debug, Clone, serde::Serialize)]
struct IngestedSubmodule {
    name: String,
    path: String,
    url: String,
    branch: Option<String>,
    version: String,
    git_status: String,
    current_commit: String,
    has_cargo_toml: bool,
    has_flake_nix: bool,
}

// ── Memecache Upgrade: force-upgrade all crates ────────────────────────

fn handle_memecache_upgrade(args: &MemecacheUpgradeArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Memecache Upgrade");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Workspace:   {}", args.workspace_path.display());
    println!("Strategy:    {}", args.strategy);
    println!("Dry run:     {}", args.dry_run);
    println!("All:         {}", args.all);
    println!();

    // Validate strategy
    if !["conservative", "aggressive", "force"].contains(&args.strategy.as_str()) {
        anyhow::bail!("Invalid strategy '{}'. Must be: conservative, aggressive, or force", args.strategy);
    }

    // Find all Cargo.toml files
    let mut cargo_tomls = Vec::new();
    if args.all {
        for entry in walkdir::WalkDir::new(&args.workspace_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml"))
                && !path.to_string_lossy().contains("/target/")
                && !path.to_string_lossy().contains("/.cargo/")
            {
                cargo_tomls.push(path.to_path_buf());
            }
        }
    } else {
        cargo_tomls.push(args.workspace_path.join("Cargo.toml"));
    }

    println!("Found {} Cargo.toml files to upgrade", cargo_tomls.len());

    // For each workspace, find dependencies and check for upgrades
    let mut total_upgrades = 0;
    let mut total_skipped = 0;
    let mut total_errors = 0;

    for cargo_toml in &cargo_tomls {
        if args.verbose {
            println!("  Scanning: {}", cargo_toml.display());
        }

        let content = match std::fs::read_to_string(cargo_toml) {
            Ok(c) => c,
            Err(_) => { total_errors += 1; continue; }
        };

        let doc: toml::Value = match content.parse() {
            Ok(d) => d,
            Err(_) => { total_errors += 1; continue; }
        };

        // Count dependencies
        let dep_count = count_dependencies(&doc);
        if args.verbose {
            println!("    {} dependencies", dep_count);
        }

        // For now, just report. The actual upgrade logic will call
        // cargo's upgrade_requirement function once we integrate with
        // the cargo source.
        if args.dry_run {
            total_skipped += dep_count;
        } else {
            // TODO: implement actual upgrade using cargo's upgrade_requirement
            total_skipped += dep_count;
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Upgrade Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Workspaces scanned:  {}", cargo_tomls.len());
    println!("Upgrades available:  {}", total_upgrades);
    println!("Skipped:             {}", total_skipped);
    println!("Errors:              {}", total_errors);
    println!();

    if args.dry_run {
        println!("Dry run — no changes made.");
    }

    Ok(())
}

fn count_dependencies(doc: &toml::Value) -> usize {
    let mut count = 0;
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get(section).and_then(|d| d.as_table()) {
            count += deps.len();
        }
    }
    count
}

// ── Memecache GC: clean stale crate versions ───────────────────────────

fn handle_memecache_gc(args: &MemecacheGcArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Memecache GC");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let registry_path = if args.registry_path.starts_with("~") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(args.registry_path.to_string_lossy().replacen("~", &home, 1))
    } else {
        args.registry_path.clone()
    };

    println!("Registry:    {}", registry_path.display());
    println!("Dry run:     {}", args.dry_run);
    println!("Clean git:   {}", args.clean_git);
    println!("Clean targets: {}", args.clean_targets);
    println!();

    // Walk the registry src directory
    let src_path = registry_path.join("src");
    if !src_path.exists() {
        anyhow::bail!("Registry src directory not found at {}", src_path.display());
    }

    // Group crate versions
    let mut crate_versions: HashMap<String, Vec<(String, u64)>> = HashMap::new();

    for entry in walkdir::WalkDir::new(&src_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Parse crate-version directory name
            if let Some((crate_name, version)) = parse_crate_version(dir_name) {
                let size = dir_size(path);
                crate_versions
                    .entry(crate_name)
                    .or_default()
                    .push((version, size));
            }
        }
    }

    // Find stale versions (keep only latest per semver range)
    let mut stale_count = 0;
    let mut stale_bytes: u64 = 0;
    let mut kept_count = 0;
    let mut kept_bytes: u64 = 0;

    let mut crate_names: Vec<&String> = crate_versions.keys().collect();
    crate_names.sort();

    for crate_name in &crate_names {
        let versions = crate_versions.get(crate_name.as_str()).unwrap();
        if versions.len() <= 1 {
            kept_count += versions.len();
            kept_bytes += versions.iter().map(|(_, s)| *s).sum::<u64>();
            continue;
        }

        // Find the latest version
        let mut sorted = versions.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0)); // string sort is close enough for semver
        let latest = sorted.last().unwrap().clone();

        if args.verbose {
            if versions.len() > 3 {
                println!("  {} has {} versions (latest: {})", crate_name, versions.len(), latest.0);
            }
        }

        for (version, size) in versions {
            if version == &latest.0 {
                kept_count += 1;
                kept_bytes += size;
            } else {
                stale_count += 1;
                stale_bytes += size;
            }
        }
    }

    // Report
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  GC Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Unique crates:       {}", crate_versions.len());
    println!("Total versions:      {}", crate_versions.values().map(|v| v.len()).sum::<usize>());
    println!("Versions to keep:    {} ({:.1} MB)", kept_count, kept_bytes as f64 / 1_048_576.0);
    println!("Versions to delete:  {} ({:.1} MB)", stale_count, stale_bytes as f64 / 1_048_576.0);
    println!();

    if args.dry_run {
        println!("Dry run — no changes made.");
    } else if stale_count > 0 {
        println!("Would delete {} stale versions ({:.1} MB) — not yet implemented", 
            stale_count, stale_bytes as f64 / 1_048_576.0);
        println!("Use --dry-run to see what would be deleted.");
    }

    Ok(())
}

/// Parse a crate-version directory name like "serde-1.0.228"
fn parse_crate_version(dir_name: &str) -> Option<(String, String)> {
    // Find the last '-' that's followed by a digit (crate-version pattern)
    for (i, c) in dir_name.char_indices().rev() {
        if c == '-' {
            let rest = &dir_name[i+1..];
            if rest.starts_with(|c: char| c.is_ascii_digit()) {
                return Some((dir_name[..i].to_string(), rest.to_string()));
            }
        }
    }
    None
}

/// Get directory size in bytes
fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

// ── Scan Index: read text file lists, .gitmodules, parquet, plocate ────

/// A scanned file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScannedFile {
    path: String,
    source: String,
    exists: bool,
    size: u64,
    ext: String,
}

/// A scanned .gitmodules entry
#[derive(Debug, Clone, serde::Serialize)]
struct ScannedGitmodule {
    gitmodules_path: String,
    submodule_name: String,
    submodule_path: String,
    submodule_url: String,
    submodule_branch: Option<String>,
}

fn handle_scan_index(args: &ScanIndexArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Scan Index");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let mut all_files: Vec<ScannedFile> = Vec::new();
    let mut all_gitmodules: Vec<ScannedGitmodule> = Vec::new();
    let mut source_stats: HashMap<String, usize> = HashMap::new();

    // 1. Read text file lists (--file-list)
    for file_list_path in &args.file_list {
        if args.verbose {
            println!("Reading file list: {}", file_list_path.display());
        }
        let count = read_file_list(file_list_path, &args.base_dir, &args.ext, &mut all_files, &args.max_lines)?;
        source_stats.insert(format!("file_list:{}", file_list_path.display()), count);
    }

    // 2. Read index directories (--index-dir)
    for index_dir in &args.index_dir {
        if args.verbose {
            println!("Scanning index directory: {}", index_dir.display());
        }
        let mut dir_count = 0usize;
        if index_dir.exists() {
            for entry in std::fs::read_dir(index_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Sample large files instead of skipping (when --sample is set)
                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    if size > SHMEM_MAX_SIZE as u64 {
                        if args.sample {
                            if args.verbose {
                                println!("  Sampling large file: {} ({:.1} MB)", name, size as f64 / 1_048_576.0);
                            }
                            // Create a sample and store it in shmem
                            match sample_file(&path, SHMEM_MAX_SIZE) {
                                Ok(sample) => {
                                    if args.verbose {
                                        println!("    Sampled: {}/{} lines, {} bytes, entropy={:.2}, hecke={:.2}, cids={}",
                                            sample.sample_lines, sample.original_lines,
                                            sample.sample_size, sample.entropy, sample.hecke_score, sample.detected_cids);
                                    }
                                    // Store the sample in shmem
                                    if args.shmem {
                                        let sample_json = serde_json::to_vec(&sample)?;
                                        let shmem_path = format!("vendormod/samples/{}", name);
                                        let _ = shmem_put("", &shmem_path, &format!("Sample of {} ({} bytes)", name, sample.original_size), &sample_json);
                                    }
                                    // Record the sample as a scanned file entry
                                    all_files.push(ScannedFile {
                                        path: path.to_string_lossy().to_string(),
                                        source: format!("index_dir:{}", index_dir.display()),
                                        exists: true,
                                        size: sample.original_size,
                                        ext: path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string(),
                                    });
                                }
                                Err(e) => {
                                    if args.verbose {
                                        eprintln!("  Warning: Failed to sample {}: {}", name, e);
                                    }
                                }
                            }
                            dir_count += 1;
                            continue;
                        } else {
                            if args.verbose {
                                println!("  Skipping large file: {} ({:.1} MB, use --sample to sample)", name, size as f64 / 1_048_576.0);
                            }
                            continue;
                        }
                    }
                    // Skip binary/non-text files
                    if name.ends_with(".cbor") || name.ends_with(".car") || name.ends_with(".parquet") {
                        continue;
                    }
                    // Skip backup/temp files
                    if name.ends_with('~') || name.ends_with(".bak") || name.starts_with('#') || name.starts_with(".#") {
                        continue;
                    }
                    let count = match read_file_list(&path, &args.base_dir, &args.ext, &mut all_files, &args.max_lines) {
                        Ok(c) => c,
                        Err(e) => {
                            if args.verbose {
                                eprintln!("  Warning: Failed to read {}: {}", name, e);
                            }
                            0
                        }
                    };
                    dir_count += count;
                }
            }
        }
        source_stats.insert(format!("index_dir:{}", index_dir.display()), dir_count);
    }

    // 3. Read .gitmodules files (--gitmodules)
    for gm_path in &args.gitmodules {
        if args.verbose {
            println!("Parsing .gitmodules: {}", gm_path.display());
        }
        let count = read_gitmodules_file(gm_path, &mut all_gitmodules)?;
        source_stats.insert(format!("gitmodules:{}", gm_path.display()), count);
    }

    // 4. Find .gitmodules via plocate (--plocate-pattern)
    if let Some(pattern) = &args.plocate_pattern {
        if args.verbose {
            println!("Searching plocate for: {}", pattern);
        }
        let count = plocate_gitmodules(pattern, &mut all_gitmodules)?;
        source_stats.insert("plocate".to_string(), count);
    }

    // 5. Read parquet files (--parquet) - shell out to python3
    for pq_path in &args.parquet {
        if args.verbose {
            println!("Reading parquet: {}", pq_path.display());
        }
        let count = read_parquet_index(pq_path, &args.base_dir, &args.ext, &mut all_files)?;
        source_stats.insert(format!("parquet:{}", pq_path.display()), count);
    }

    // 6. Deduplicate files by path
    let mut seen_paths: HashSet<String> = HashSet::new();
    all_files.retain(|f| seen_paths.insert(f.path.clone()));

    // 7. Report
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Scan Results");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Unique files found:     {}", all_files.len());
    println!("Gitmodules entries:     {}", all_gitmodules.len());
    println!();

    // Source breakdown
    println!("Sources:");
    for (source, count) in &source_stats {
        println!("  {}: {} entries", source, count);
    }

    // Extension breakdown
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    for f in &all_files {
        *ext_counts.entry(f.ext.clone()).or_default() += 1;
    }
    let mut ext_sorted: Vec<_> = ext_counts.iter().collect();
    ext_sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!();
    println!("Top extensions:");
    for (ext, count) in ext_sorted.iter().take(20) {
        println!("  .{}: {} files", ext, count);
    }

    // Existence check
    let existing = all_files.iter().filter(|f| f.exists).count();
    let missing = all_files.len() - existing;
    println!();
    println!("Existing files: {}", existing);
    println!("Missing files:  {}", missing);

    // 8. Write output
    if !args.dry_run {
        std::fs::create_dir_all(&args.output_dir)?;

        match args.format.as_str() {
            "json" => {
                let output = serde_json::json!({
                    "total_files": all_files.len(),
                    "total_gitmodules": all_gitmodules.len(),
                    "sources": source_stats,
                    "extension_counts": ext_counts,
                    "files": all_files,
                    "gitmodules": all_gitmodules,
                });
                let json_path = args.output_dir.join("scan-results.json");
                std::fs::write(&json_path, serde_json::to_string_pretty(&output)?)?;
                println!("Written JSON to {}", json_path.display());
            }
            "text" => {
                let txt_path = args.output_dir.join("scan-results.txt");
                let mut out = String::new();
                for f in &all_files {
                    out.push_str(&format!("{} {} {} {}\n", f.path, f.source, f.exists, f.size));
                }
                std::fs::write(&txt_path, &out)?;
                println!("Written text to {}", txt_path.display());
            }
            _ => {
                // summary only — already printed above
                let summary_path = args.output_dir.join("scan-summary.json");
                let summary = serde_json::json!({
                    "total_files": all_files.len(),
                    "total_gitmodules": all_gitmodules.len(),
                    "sources": source_stats,
                    "extension_counts": ext_counts,
                    "existing": existing,
                    "missing": missing,
                });
                std::fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;
                println!("Written summary to {}", summary_path.display());
            }
        }
    }

    // 9. Store in shmem if requested (1MB cap per block)
    if args.shmem {
        // Store summary
        let summary = serde_json::json!({
            "total_files": all_files.len(),
            "total_gitmodules": all_gitmodules.len(),
            "sources": source_stats,
            "existing": existing,
            "missing": missing,
        });
        let summary_bytes = serde_json::to_vec(&summary)?;
        let _ = shmem_put("", "vendormod/scan-summary", "Scan index summary", &summary_bytes);

        // Store gitmodules entries (batch as one if under 1MB)
        let gm_bytes = serde_json::to_vec(&all_gitmodules)?;
        let _ = shmem_put("", "vendormod/scan-gitmodules", &format!("{} gitmodules entries", all_gitmodules.len()), &gm_bytes);

        // Store file entries in chunks of 1000 (to stay under 1MB)
        for (chunk_idx, chunk) in all_files.chunks(1000).enumerate() {
            let chunk_bytes = serde_json::to_vec(chunk)?;
            let _ = shmem_put("", &format!("vendormod/scan-files-{}", chunk_idx), &format!("Files chunk {} ({} entries)", chunk_idx, chunk.len()), &chunk_bytes);
        }

        println!("Stored results in IPLD shmem (1MB cap per block)");
    }

    Ok(())
}

/// Read a text file list (one path per line)
fn read_file_list(
    file_path: &Path,
    base_dir: &Path,
    ext_filter: &[String],
    results: &mut Vec<ScannedFile>,
    max_lines: &usize,
) -> Result<usize> {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => {
            // Try reading as bytes and lossy-converting for non-UTF8 files
            let bytes = std::fs::read(file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            String::from_utf8_lossy(&bytes).to_string()
        }
    };
    let source_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let mut count = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Apply max_lines limit
        if *max_lines > 0 && count >= *max_lines {
            break;
        }

        // Skip URLs (https://, http://, git@, ssh://)
        if line.starts_with("https://") || line.starts_with("http://")
            || line.starts_with("git@") || line.starts_with("ssh://")
        {
            // Record as URL reference, not a file path
            results.push(ScannedFile {
                path: line.to_string(),
                source: source_name.clone(),
                exists: false,
                size: 0,
                ext: "url".to_string(),
            });
            count += 1;
            continue;
        }

        // Strip trailing colons (directory indicators from org-mode/git)
        let cleaned = line.trim_end_matches(':');

        // Expand ~ to home directory
        let expanded = if cleaned.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/mdupont".to_string());
            cleaned.replacen("~", &home, 1)
        } else {
            cleaned.to_string()
        };

        // Resolve path (absolute or relative to base_dir)
        let resolved = if expanded.starts_with('/') {
            PathBuf::from(&expanded)
        } else {
            base_dir.join(&expanded)
        };

        let ext = resolved.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Apply extension filter
        if !ext_filter.is_empty() && !ext_filter.iter().any(|e| e.trim_start_matches('.') == ext) {
            continue;
        }

        let exists = resolved.exists();
        let size = if exists {
            resolved.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        results.push(ScannedFile {
            path: resolved.to_string_lossy().to_string(),
            source: source_name.clone(),
            exists,
            size,
            ext,
        });

        count += 1;
    }

    Ok(count)
}

/// Read a .gitmodules file and extract submodule entries
fn read_gitmodules_file(
    gitmodules_path: &Path,
    results: &mut Vec<ScannedGitmodule>,
) -> Result<usize> {
    let content = std::fs::read_to_string(gitmodules_path)
        .with_context(|| format!("Failed to read {}", gitmodules_path.display()))?;

    let git_config: git_config::File = content.as_str().try_into()
        .context("Failed to parse .gitmodules")?;

    let gm_path_str = gitmodules_path.to_string_lossy().to_string();
    let mut count = 0;

    for section in git_config.sections() {
        let header = section.header();
        if header.name() != "submodule" {
            continue;
        }
        let sub_name = header.subsection_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let path_val = git_config.string("submodule", Some(sub_name.as_str()), "path")
            .unwrap_or_default()
            .to_string();
        let url_val = git_config.string("submodule", Some(sub_name.as_str()), "url")
            .unwrap_or_default()
            .to_string();
        let branch_val = git_config.string("submodule", Some(sub_name.as_str()), "branch")
            .map(|s| s.to_string());

        if !path_val.is_empty() {
            results.push(ScannedGitmodule {
                gitmodules_path: gm_path_str.clone(),
                submodule_name: sub_name,
                submodule_path: path_val,
                submodule_url: url_val,
                submodule_branch: branch_val,
            });
            count += 1;
        }
    }

    Ok(count)
}

/// Find .gitmodules via plocate and parse them
fn plocate_gitmodules(
    pattern: &str,
    results: &mut Vec<ScannedGitmodule>,
) -> Result<usize> {
    let output = std::process::Command::new("plocate")
        .args(["-l", "500", pattern])
        .output()
        .context("Failed to run plocate (is it installed?)")?;

    if !output.status.success() {
        eprintln!("plocate failed: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut total = 0;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.ends_with(".gitmodules") {
            continue;
        }
        let gm_path = PathBuf::from(line);
        if !gm_path.exists() {
            continue;
        }
        match read_gitmodules_file(&gm_path, results) {
            Ok(count) => total += count,
            Err(e) => {
                eprintln!("  Warning: Failed to parse {}: {}", gm_path.display(), e);
            }
        }
    }

    Ok(total)
}

/// Read a parquet file as a file index (shell out to python3 + pyarrow)
fn read_parquet_index(
    parquet_path: &Path,
    base_dir: &Path,
    ext_filter: &[String],
    results: &mut Vec<ScannedFile>,
) -> Result<usize> {
    let pq_str = parquet_path.to_string_lossy().to_string();
    let base_str = base_dir.to_string_lossy().to_string();
    let ext_filter_json = serde_json::to_string(ext_filter)?;

    // Python script to read parquet and output paths as JSON
    let python_script = r#"
import pyarrow.parquet as pq
import json
import sys
import os

pq_path = sys.argv[1]
base_dir = sys.argv[2]
ext_filter = json.loads(sys.argv[3])

t = pq.read_table(pq_path)
df = t.to_pandas()

# Find path columns
path_col = None
for col in df.columns:
    if col.lower() in ('path', 'file_path', 'filepath', 'filename'):
        path_col = col
        break

if path_col is None:
    # Try first string column
    for col in df.columns:
        if df[col].dtype == object:
            path_col = col
            break

if path_col is None:
    print(json.dumps([]))
    sys.exit(0)

paths = df[path_col].dropna().tolist()
if ext_filter:
    exts = set(e.lstrip('.') for e in ext_filter)
    paths = [p for p in paths if isinstance(p, str) and os.path.splitext(p)[1].lstrip('.') in exts]

output = []
for p in paths:
    if not isinstance(p, str):
        continue
    resolved = p if p.startswith('/') else os.path.join(base_dir, p)
    exists = os.path.exists(resolved)
    size = os.path.getsize(resolved) if exists else 0
    ext = os.path.splitext(resolved)[1].lstrip('.')
    output.append({
        "path": resolved,
        "source": os.path.basename(pq_path),
        "exists": exists,
        "size": size,
        "ext": ext
    })

print(json.dumps(output))
"#;

    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(python_script)
        .arg(&pq_str)
        .arg(&base_str)
        .arg(&ext_filter_json)
        .output()
        .context("Failed to run python3 for parquet reading")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: python3 parquet reader failed: {}", stderr);
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<ScannedFile> = serde_json::from_str(&stdout).unwrap_or_default();
    let count = files.len();
    results.extend(files);

    Ok(count)
}

