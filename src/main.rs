//! # Main CLI Dispatcher
//!
//! Cargo-vendormod main entry point — all command logic runs in-process.

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

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

        Some(Commands::FlakeCheck(args)) => {
            let report = cargo_vendormod::flake_check::check_flake_coverage(&args.flakes_dir)?;
            if args.json {
                cargo_vendormod::flake_check::print_json(&report)?;
            } else {
                cargo_vendormod::flake_check::print_report(&report);
            }
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

