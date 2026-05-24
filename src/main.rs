//! # Main CLI Dispatcher
//!
//! Cargo-vendormod main entry point that delegates to specialized binaries.

use anyhow::Result;
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
        // Delegate to sub-binaries
        Some(Commands::Vendoring(vcmd)) => {
            delegate_to_binary("vendoring", &["--config", &args.config.as_ref().map(|p| p.to_string_lossy().to_string()).unwrap_or_default()])
        }
        Some(Commands::Graph(gcmd)) => {
            delegate_to_binary("graph", &[])
        }
        Some(Commands::Process(pcmd)) => {
            delegate_to_binary("processing", &[])
        }

        // Inline commands
        Some(Commands::Init(_)) => {
            println!("Use: cargo-vendormod vendoring init");
            delegate_to_binary("vendoring", &["init"])
        }
        Some(Commands::FetchUpstream) => {
            delegate_to_binary("vendoring", &["fetch-upstream"])
        }
        Some(Commands::Rebase) => {
            delegate_to_binary("vendoring", &["rebase"])
        }
        Some(Commands::Releases) => {
            delegate_to_binary("vendoring", &["releases"])
        }
        Some(Commands::Status) => {
            delegate_to_binary("vendoring", &["status"])
        }
        Some(Commands::Sync) => {
            delegate_to_binary("vendoring", &["sync"])
        }
        Some(Commands::Patch) => {
            delegate_to_binary("vendoring", &["patch"])
        }

        Some(Commands::BuildGraph(_)) => {
            delegate_to_binary("graph", &["build"])
        }
        Some(Commands::AnalyzeGraph(_)) => {
            delegate_to_binary("graph", &["analyze"])
        }
        Some(Commands::VisualizeGraph(_)) => {
            delegate_to_binary("graph", &["visualize"])
        }
        Some(Commands::PartitionGraph(_)) => {
            delegate_to_binary("graph", &["partition"])
        }

        Some(Commands::ProcessCrates(_)) => {
            delegate_to_binary("processing", &["crates"])
        }
        Some(Commands::ProcessAll(_)) => {
            delegate_to_binary("processing", &["all"])
        }
        Some(Commands::RunWorkflow(_)) => {
            delegate_to_binary("processing", &["workflow"])
        }

        Some(Commands::Report(_)) => {
            delegate_to_binary("processing", &["report"])
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

fn delegate_to_binary(name: &str, _args: &[&str]) -> Result<()> {
    // For now, show a message about the binary name
    // In a full implementation, we'd use std::process::Command to spawn the binary
    println!("Delegating to {} binary (not yet implemented)", name);
    println!("Run with cargo run --bin {} -- [args]", name);
    Ok(())
}