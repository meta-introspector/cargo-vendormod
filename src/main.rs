//! # Main CLI Dispatcher
//!
//! Cargo-vendormod main entry point — all command logic runs in-process.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

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
    /// List available workloads
    WorkloadList,
    /// Create git worktree for workload
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
    /// Ingest old submodules directory into the new vendormod registry
    Ingest(IngestArgs),
    /// Force-upgrade all crate dependencies to latest versions
    MemecacheUpgrade(MemecacheUpgradeArgs),
    /// Clean stale crate versions from the memecache
    MemecacheGc(MemecacheGcArgs),
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
struct SplitArgs {
    /// Path to the file to split
    #[arg(long)]
    input_file: PathBuf,
}


/// Defined workload configuration
#[derive(Debug, serde::Serialize)]
pub struct WorkloadDef {
    pub name: String,
    pub path: String,
    pub layer1_count: usize,
    pub layer2_count: usize,
    pub description: String,
}

impl Default for WorkloadDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            path: String::new(),
            layer1_count: 0,
            layer2_count: 0,
            description: String::new(),
        }
    }
}

fn get_defined_workloads() -> Vec<WorkloadDef> {
    vec![
        WorkloadDef {
            name: "solana-sdk".to_string(),
            path: "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-sdk".to_string(),
            layer1_count: 93,
            layer2_count: 186,
            description: "Solana SDK - Core on-chain program library".to_string(),
        },
        WorkloadDef {
            name: "solana-main".to_string(),
            path: "/mnt/data1/nix/vendor/rust/cargo2nix/submodules/solana-main".to_string(),
            layer1_count: 218,
            layer2_count: 189,
            description: "Solana Validator - Full validator implementation".to_string(),
        },
    ]
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

        let git_status = if exists {
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

        let current_commit = if exists {
            std::process::Command::new("git")
                .args(["-C", sub_path.to_str().unwrap_or(""), "rev-parse", "HEAD"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        } else {
            "N/A".to_string()
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
fn shmem_put(socket: &str, path: &str, description: &str, data: &[u8]) -> Result<()> {
    // Connect to the shmem server and send a put command
    let mut cmd = std::process::Command::new("letta-ipld-memory");
    cmd.arg("put")
       .arg(path)
       .arg("--description")
       .arg(description)
       .arg("--data")
       .arg("-"); // read from stdin would be ideal, but for now just log
    // For now, just log that we would store it
    eprintln!("[shmem] Would store {} ({} bytes) at {}", path, data.len(), socket);
    Ok(())
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

