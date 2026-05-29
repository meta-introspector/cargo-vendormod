//! # Processing Binary
//!
//! Crate processing - crate discovery, flake generation, combined NUR workflow.

use anyhow::{Context, Result};
use cargo_vendormod::config::Config;
use cargo_vendormod::crate_flake::{FlakeSource, FlakeStyle};
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "process")]
#[command(about = "Crate processing operations", long_about = None)]
struct ProcessingArgs {
    /// Path to vendormod config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Path to workspace root
    #[arg(long, global = true)]
    workspace_path: Option<PathBuf>,

    /// Output directory
    #[arg(long, global = true, default_value = "./processed")]
    output_dir: PathBuf,

    /// Input file (for process-all-crates)
    #[arg(long, global = true)]
    input_file: Option<PathBuf>,

    /// Path to repos.json (for combined/discover)
    #[arg(long, global = true)]
    repos_json: Option<PathBuf>,

    /// Path to repos.json.lock (for combined/discover)
    #[arg(long, global = true)]
    lock_json: Option<PathBuf>,

    /// Path to vendored deps directory (e.g. ./vendor)
    #[arg(long, global = true)]
    vendor_dir: Option<PathBuf>,

    /// Generate Nix flakes
    #[arg(long, global = true)]
    generate_flakes: bool,

    /// Compile crates standalone
    #[arg(long, global = true)]
    compile_standalone: bool,

    /// Use layered processing
    #[arg(long, global = true)]
    layered_processing: bool,

    /// Process specific layer only (1=external, 2=workspace)
    #[arg(long, global = true)]
    layer: Option<u32>,

    /// Maximum parallel jobs
    #[arg(long, global = true, default_value = "4")]
    max_parallel: usize,

    /// Flake generation style: "simple" (default), "crate2nix"
    #[arg(long, global = true, default_value = "simple")]
    style: String,

    /// Path to nix-common directory (required for --style crate2nix)
    #[arg(long, global = true)]
    nix_common: Option<PathBuf>,

    /// Git mirror path for source (e.g. /mnt/data1/git/github.com/ipld/rust-ipld-core.git)
    /// When set, the generated flake references this mirror instead of local files.
    #[arg(long, global = true)]
    git_mirror: Option<PathBuf>,

    /// Display name for the git mirror (used in flake comments)
    #[arg(long, global = true)]
    mirror_name: Option<String>,

    /// Verbose output
    #[arg(long, short, global = true)]
    verbose: bool,

    /// DASL data root path (for generate-lockfiles / dasl-pipeline)
    #[arg(long, global = true)]
    dasl_data: Option<PathBuf>,

    /// Crate list file (for aggregate-flake)
    #[arg(long, global = true)]
    crate_list: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<ProcessingCommands>,
}

#[derive(Subcommand, Debug)]
enum ProcessingCommands {
    /// Process crates from workspace
    Crates,
    /// Process all crates from file list
    All,
    /// Run complete workflow (graph build + processing)
    Workflow,
    /// Generate processing report
    Report,
    /// Discover Cargo.toml files under a directory tree
    Discover,
    /// End-to-end NUR workflow: discover + generate flakes + combined flake
    Combined,
    /// Generate Cargo.lock files for workspaces that lack them
    GenerateLockfiles,
    /// Generate aggregate flake.nix from crate list
    AggregateFlake,
    /// Full DASL pipeline: generate lockfiles → discover crates → generate flakes → aggregate flake
    DaslPipeline,
}

fn main() -> Result<()> {
    let args = ProcessingArgs::parse();
    let config = Config::load(args.config.as_deref())?;

    let workspace_path = args.workspace_path.clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current dir"));

    // Create output directory
    fs::create_dir_all(&args.output_dir)?;

    match args.command {
        Some(ProcessingCommands::Crates) => cmd_crates(&args, &config, &workspace_path)?,
        Some(ProcessingCommands::All) => cmd_all(&args)?,
        Some(ProcessingCommands::Workflow) => cmd_workflow(&args, &workspace_path)?,
        Some(ProcessingCommands::Report) => cmd_report(&args)?,
        Some(ProcessingCommands::Discover) => cmd_discover(&args, &workspace_path)?,
        Some(ProcessingCommands::GenerateLockfiles) => cmd_generate_lockfiles(&args)?,
        Some(ProcessingCommands::AggregateFlake) => cmd_aggregate_flake(&args)?,
        Some(ProcessingCommands::DaslPipeline) => cmd_dasl_pipeline(&args)?,
        Some(ProcessingCommands::Combined) => cmd_combined(&args)?,
        None => {
            cmd_crates(&args, &config, &workspace_path)?;
        }
    }

    Ok(())
}

// ── Helper: build FlakeStyle / FlakeSource from CLI args ─────────────────

fn resolve_style(args: &ProcessingArgs) -> (FlakeStyle, Option<PathBuf>) {
    let style = FlakeStyle::from_str(Some(&args.style));
    let nix_common = if style == FlakeStyle::Crate2Nix {
        Some(args.nix_common.clone().unwrap_or_else(|| {
            eprintln!("Warning: --style crate2nix requires --nix-common; falling back to simple");
            PathBuf::new()
        }))
    } else {
        args.nix_common.clone()
    };
    (style, nix_common)
}

fn resolve_source(args: &ProcessingArgs) -> FlakeSource {
    match (&args.git_mirror, &args.mirror_name) {
        (Some(mirror_path), Some(display)) => FlakeSource::GitMirror {
            mirror_path: mirror_path.clone(),
            display: display.clone(),
        },
        (Some(mirror_path), None) => {
            // Derive display name from mirror path (strip .git suffix)
            let display = mirror_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("mirror")
                .trim_end_matches(".git")
                .to_string();
            FlakeSource::GitMirror {
                mirror_path: mirror_path.clone(),
                display,
            }
        }
        _ => FlakeSource::LocalDir,
    }
}

// ── Command implementations ────────────────────────────────────────────

fn cmd_crates(args: &ProcessingArgs, _config: &Config, workspace_path: &PathBuf) -> Result<()> {
    println!("Processing crates from {}", workspace_path.display());
    println!("  Output: {}", args.output_dir.display());
    println!("  Flakes: {}", args.generate_flakes);
    println!("  Standalone: {}", args.compile_standalone);
    println!("  Layered: {}", args.layered_processing);
    println!("  Parallel: {}", args.max_parallel);

    let (style, nix_common) = resolve_style(args);
    let source = resolve_source(args);

    use cargo_vendormod::global_dep_graph::GlobalDependencyGraphBuilder;

    // Build dependency graph
    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.clone());
    builder.set_options(true, true, true);
    let graph = builder.build_global_graph()?;

    println!("  Graph: {} nodes, {} edges", graph.nodes.len(), graph.edges.len());

    // Process based on layer
    let layer = args.layer.unwrap_or(0);

    if args.layered_processing || layer == 0 {
        // Process external dependencies first
        let external_nodes: Vec<_> = graph.nodes.iter()
            .filter(|n| !n.is_workspace_member)
            .collect();

        println!("\nProcessing {} external dependencies...", external_nodes.len());
        
        for node in external_nodes {
            if args.verbose {
                println!("  Processing: {}", node.crate_name);
            }
            let crate_dir = workspace_path.join(&node.id);
            if crate_dir.exists() {
                if args.generate_flakes {
                    cargo_vendormod::crate_flake::generate_flake(
                        &crate_dir,
                        &args.output_dir,
                        args.vendor_dir.as_deref(),
                        style,
                        nix_common.as_deref(),
                        source.clone(),
                    )?;
                }
            }
        }
    }

    if args.layered_processing || layer == 0 || layer == 2 {
        // Process workspace members
        let workspace_nodes: Vec<_> = graph.nodes.iter()
            .filter(|n| n.is_workspace_member)
            .collect();

        println!("\nProcessing {} workspace members...", workspace_nodes.len());

        for node in workspace_nodes {
            if args.verbose {
                println!("  Processing: {}", node.crate_name);
            }

            // Workspace members: the crate IS the workspace root, so use it directly.
            // The graph builder assigns node.id = crate_name, which when joined with
            // workspace_path produces an incorrect path when workspace_path already
            // points to the crate directory.
            let crate_dir = if workspace_path.ends_with(&node.id) {
                workspace_path.clone()
            } else {
                workspace_path.join(&node.id)
            };

            if crate_dir.exists() {
                if args.generate_flakes {
                    cargo_vendormod::crate_flake::generate_flake(
                        &crate_dir,
                        &args.output_dir,
                        args.vendor_dir.as_deref(),
                        style,
                        nix_common.as_deref(),
                        source.clone(),
                    )?;
                }
            }
        }
    }

    println!("\nProcessing complete.");
    Ok(())
}

fn cmd_all(args: &ProcessingArgs) -> Result<()> {
    let input_file = args.input_file.clone()
        .context("Input file required for process-all command")?;

    println!("Processing crates from file list: {}", input_file.display());

    let content = fs::read_to_string(&input_file)?;
    let crate_paths: Vec<PathBuf> = content.lines()
        .map(PathBuf::from)
        .collect();

    println!("  Found {} crates", crate_paths.len());
    println!("  Parallel: {}", args.max_parallel);

    let (style, nix_common) = resolve_style(args);
    let source = resolve_source(args);

    let output_dir = args.output_dir.clone();
    let verbose = args.verbose;
    let generate_flakes = args.generate_flakes;
    let vendor_dir = args.vendor_dir.clone();

    if generate_flakes {
        use rayon::prelude::*;
        crate_paths.par_iter().for_each(|crate_path| {
            if verbose {
                println!("  Processing: {}", crate_path.display());
            }

            if let Err(e) = cargo_vendormod::crate_flake::generate_flake(
                crate_path,
                &output_dir,
                vendor_dir.as_deref(),
                style,
                nix_common.as_deref(),
                source.clone(),
            ) {
                eprintln!("  Failed to generate flake for {}: {:#}", crate_path.display(), e);
            }
        });
    }

    println!("\nAll crates processed.");
    Ok(())
}

fn cmd_discover(args: &ProcessingArgs, workspace_path: &PathBuf) -> Result<()> {
    println!("Discovering Cargo.toml files under {}", workspace_path.display());

    let crates = cargo_vendormod::crate_flake::find_crate_dirs(workspace_path)
        .context("Failed to discover crate directories")?;

    println!("\nFound {} crate directories:", crates.len());
    for c in &crates {
        println!("  {}", c.display());
    }

    // Save to file if output_dir is specified
    if !args.output_dir.as_os_str().is_empty() {
        fs::create_dir_all(&args.output_dir)?;
        let list_path = args.output_dir.join("discovered_crates.txt");
        let lines: Vec<String> = crates.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        fs::write(&list_path, lines.join("\n"))?;
        println!("\nSaved crate list to: {}", list_path.display());
    }

    // Optionally generate flakes
    if args.generate_flakes && !crates.is_empty() {
        let (style, nix_common) = resolve_style(args);
        let source = resolve_source(args);

        println!("\nGenerating flakes for {} crates...", crates.len());
        let count = cargo_vendormod::crate_flake::generate_flakes_for_crates(
            &crates,
            &args.output_dir,
            args.vendor_dir.as_deref(),
            style,
            nix_common.as_deref(),
            &source,
            args.verbose,
        )?;
        println!("  Generated {} flake files", count);
    }

    Ok(())
}

fn cmd_combined(args: &ProcessingArgs) -> Result<()> {
    println!("═══ NUR Combined Workflow ═══");
    println!();

    // ── Phase 1: validate inputs ──
    let repos_json = args.repos_json.clone()
        .or_else(|| args.workspace_path.as_ref().map(|p| p.join("repos.json")))
        .context("Need --repos-json or --workspace-path pointing to NUR workspace")?;

    let lock_json = args.lock_json.clone()
        .or_else(|| {
            let base = args.repos_json.as_ref()
                .and_then(|p| p.parent())
                .or_else(|| args.workspace_path.as_ref().map(|p| p.as_path()));
            base.map(|p| p.join("repos.json.lock"))
        })
        .context("Need --lock-json or --workspace-path pointing to NUR workspace")?;

    let repos_dir = repos_json.parent()
        .unwrap_or_else(|| std::path::Path::new("."))
        .join("repos");

    println!("  repos.json:  {}", repos_json.display());
    println!("  repos.json.lock: {}", lock_json.display());
    println!("  repos dir:   {}", repos_dir.display());
    println!("  output dir:  {}", args.output_dir.display());
    println!();

    let (style, nix_common) = resolve_style(args);
    let source = resolve_source(args);

    // ── Phase 2: Discover Cargo.toml files in repos/ ──
    println!("Phase 1/3: Discovering crate directories...");
    let crates = cargo_vendormod::crate_flake::find_crate_dirs(&repos_dir)
        .context("Failed to discover crate directories")?;
    println!("  Found {} crate directories with Cargo.toml", crates.len());

    // Save crate list
    let crates_list_path = args.output_dir.join("discovered_crates.txt");
    let lines: Vec<String> = crates.iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    fs::write(&crates_list_path, lines.join("\n"))?;
    println!("  Saved crate list to: {}", crates_list_path.display());
    println!();

    // ── Phase 3: Generate per-crate flakes ──
    println!("Phase 2/3: Generating per-crate flakes...");
    let flake_count = cargo_vendormod::crate_flake::generate_flakes_for_crates(
        &crates,
        &args.output_dir,
        args.vendor_dir.as_deref(),
        style,
        nix_common.as_deref(),
        &source,
        args.verbose,
    )?;
    println!("  Generated {} flake files", flake_count);
    println!();

    // ── Phase 4: Generate combined NUR flake ──
    println!("Phase 3/3: Generating combined NUR flake.nix...");
    let combined_output = args.output_dir.join("flake.nix");
    let generator = cargo_vendormod::nur_flake::NurFlakeGenerator::new(
        repos_json.clone(),
        lock_json.clone(),
        combined_output.clone(),
    );
    let repo_count = generator
        .generate()
        .context("Failed to generate combined NUR flake")?;
    println!("  Generated combined flake.nix with {} NUR repos", repo_count);
    println!();

    // ── Summary ──
    println!("═══ Workflow Complete ═══");
    println!("  Crates discovered:  {}", crates.len());
    println!("  Flake files:        {}", flake_count);
    println!("  NUR repos in flake: {}", repo_count);
    println!();

    Ok(())
}

fn cmd_workflow(args: &ProcessingArgs, workspace_path: &PathBuf) -> Result<()> {
    println!("Running complete workflow for {}", workspace_path.display());

    // Simplified workflow - just build the graph and process
    use cargo_vendormod::global_dep_graph::GlobalDependencyGraphBuilder;

    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.clone());
    builder.set_options(true, true, true);
    let graph = builder.build_global_graph()?;

    println!("Built graph with {} nodes", graph.nodes.len());

    // Create output structure
    fs::create_dir_all(args.output_dir.join("crates"))?;
    
    // Save graph for later use
    let graph_path = args.output_dir.join("graph.json");
    let json = serde_json::to_string_pretty(&graph)?;
    fs::write(&graph_path, json)?;

    println!("\nWorkflow complete. Graph saved to {}", graph_path.display());
    Ok(())
}

fn cmd_report(args: &ProcessingArgs) -> Result<()> {
    let input_file = args.input_file.clone()
        .context("Input file required for report command")?;

    println!("Generating processing report");

    let content = fs::read_to_string(&input_file)?;
    let crate_paths: Vec<PathBuf> = content.lines()
        .map(PathBuf::from)
        .collect();

    // Analyze processed crates
    let mut total_size = 0u64;
    let mut with_tests = 0;
    let mut with_benches = 0;
    let mut with_examples = 0;

    for crate_path in &crate_paths {
        // Count lines of code
        for entry in walkdir::WalkDir::new(crate_path).max_depth(3).into_iter().flatten() {
            if entry.path().extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(meta) = fs::metadata(entry.path()) {
                    total_size += meta.len();
                }
            }
        }

        // Check for tests/benches/examples
        if crate_path.join("tests").exists() { with_tests += 1; }
        if crate_path.join("benches").exists() { with_benches += 1; }
        if crate_path.join("examples").exists() { with_examples += 1; }
    }

    println!("\n=== Processing Report ===");
    println!("Total crates: {}", crate_paths.len());
    println!("Total size: {} bytes", total_size);
    println!("Crates with tests: {}", with_tests);
    println!("Crates with benches: {}", with_benches);
    println!("Crates with examples: {}", with_examples);

    // Save report
    let report_path = args.output_dir.join("processing_report.json");
    let report = serde_json::json!({
        "total_crates": crate_paths.len(),
        "total_bytes": total_size,
        "with_tests": with_tests,
        "with_benches": with_benches,
        "with_examples": with_examples,
    });
    fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;

    println!("\nReport saved to: {}", report_path.display());
    Ok(())
}

/// Generate Cargo.lock files for all DASL workspaces that lack them.
///
/// Runs `cargo generate-lockfile` in each workspace under `--dasl-data` that
/// does not have a `Cargo.lock`. Copies results to `--output-dir/lockfiles/`.
fn cmd_generate_lockfiles(args: &ProcessingArgs) -> Result<()> {
    let dasl_data = args.dasl_data.clone()
        .or_else(|| args.workspace_path.clone())
        .context("Need --dasl-data or --workspace-path pointing to DASL data root")?;

    let lockfiles_dir = args.output_dir.join("lockfiles");

    println!("═══ DASL Lockfile Generator ═══");
    println!("  DASL data root:   {}", dasl_data.display());
    println!("  Lockfiles output: {}", lockfiles_dir.display());
    println!("  Verbose:          {}", args.verbose);
    println!();

    let results = cargo_vendormod::dasl_pipeline::generate_workspace_lockfiles(
        &dasl_data, &lockfiles_dir, args.verbose,
    )?;

    println!("{}", cargo_vendormod::dasl_pipeline::format_lockfile_summary(&results));
    println!("═══ Lockfile generation complete ═══");
    Ok(())
}

/// Generate an aggregate flake.nix from the discovered crate list.
///
/// Reads `--crate-list` (one crate directory per line), resolves each crate's
/// name, version, source path, and Cargo.lock (if any), and writes a top-level
/// `flake.nix` at `--output-dir/flake.nix`.
fn cmd_aggregate_flake(args: &ProcessingArgs) -> Result<()> {
    let crate_list = args.crate_list.clone()
        .or_else(|| args.input_file.clone())
        .or_else(|| {
            // Default: look for discovered_crates.txt in output dir
            let f = args.output_dir.join("discovered_crates.txt");
            if f.exists() { Some(f) } else { None }
        })
        .context("Need --crate-list or --input-file, or run discover first")?;

    let output_path = if args.output_dir.join("flake.nix").exists()
        && args.output_dir.file_name().and_then(|n| n.to_str()) == Some(".")
    {
        // If output is current dir, write to a named file
        args.output_dir.join("aggregate-flake.nix")
    } else {
        // Write flake.nix directly in output dir
        args.output_dir.join("flake.nix")
    };

    println!("═══ DASL Aggregate Flake Generator ═══");
    println!("  Crate list:    {}", crate_list.display());
    println!("  Output:        {}", output_path.display());
    println!("  Verbose:       {}", args.verbose);
    println!();

    cargo_vendormod::dasl_pipeline::generate_aggregate_flake(
        &crate_list, &output_path, args.verbose,
    )?;

    println!("═══ Aggregate flake generation complete ═══");
    println!("  Written to: {}", output_path.display());
    Ok(())
}

/// Full DASL pipeline: generate lockfiles → discover crates → generate per-crate flakes → aggregate flake.
///
/// All outputs go under `--output-dir`:
///   <output>/lockfiles/              — Cargo.lock copies
///   <output>/flakes/<crate>/         — per-crate flake.nix + package.nix
///   <output>/discovered_crates.txt   — full crate list
///   <output>/flake.nix               — aggregate flake.nix
fn cmd_dasl_pipeline(args: &ProcessingArgs) -> Result<()> {
    let dasl_data = args.dasl_data.clone()
        .or_else(|| args.workspace_path.clone())
        .context("Need --dasl-data or --workspace-path pointing to DASL data root")?;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       DASL Full Pipeline                                    ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  DASL data root:  {}", dasl_data.display());
    println!("  Output dir:      {}", args.output_dir.display());
    println!("  Verbose:         {}", args.verbose);
    println!();

    // ── Phase 1: Generate lockfiles ──
    println!("Phase 1/3: Generating Cargo.lock files...");
    let lockfiles_dir = args.output_dir.join("lockfiles");
    let lock_results = cargo_vendormod::dasl_pipeline::generate_workspace_lockfiles(
        &dasl_data, &lockfiles_dir, args.verbose,
    )?;
    println!("{}", cargo_vendormod::dasl_pipeline::format_lockfile_summary(&lock_results));

    // ── Phase 2: Discover crates and generate per-crate flakes ──
    println!("Phase 2/3: Discovering crates and generating per-crate flakes...");
    {
        let crates = cargo_vendormod::crate_flake::find_crate_dirs(&dasl_data)
            .context("Failed to discover crate directories")?;
        println!("  Found {} crate directories", crates.len());

        // Save crate list
        let list_path = args.output_dir.join("discovered_crates.txt");
        let lines: Vec<String> = crates.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        fs::write(&list_path, lines.join("\n"))?;
        println!("  Saved crate list to: {}", list_path.display());

        // Generate flakes
        let style = crate_flake_style_from_str(Some(&args.style));
        let source = resolve_source(args);

        println!("  Generating flakes...");
        let flake_count = cargo_vendormod::crate_flake::generate_flakes_for_crates(
            &crates,
            &args.output_dir,
            args.vendor_dir.as_deref(),
            style,
            args.nix_common.as_deref(),
            &source,
            args.verbose,
        )?;
        println!("  Generated {} flake files", flake_count);
    }

    // ── Phase 3: Generate aggregate flake ──
    println!();
    println!("Phase 3/3: Generating aggregate flake.nix...");
    let crate_list_path = args.output_dir.join("discovered_crates.txt");
    let aggregate_path = args.output_dir.join("flake.nix");
    cargo_vendormod::dasl_pipeline::generate_aggregate_flake(
        &crate_list_path, &aggregate_path, args.verbose,
    )?;
    println!("  Aggregate flake: {}", aggregate_path.display());

    // ── Summary ──
    let total_crate_dirs = std::fs::read_dir(args.output_dir.join("flakes"))
        .map(|d| d.count())
        .unwrap_or(0);
    let lock_count = std::fs::read_dir(&lockfiles_dir)
        .map(|d| d.count())
        .unwrap_or(0);

    println!();
    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║       Pipeline Complete                                     ║");
    println!("╚══════════════════════════════════════════════════════════════╝");
    println!("  Crates with flakes: {}", total_crate_dirs);
    println!("  Workspaces with locks: {}", lock_count);
    println!("  Output: {}", args.output_dir.display());
    println!();

    Ok(())
}

/// Helper: resolve FlakeStyle from string (copied pattern from resolve_style).
fn crate_flake_style_from_str(s: Option<&str>) -> cargo_vendormod::crate_flake::FlakeStyle {
    cargo_vendormod::crate_flake::FlakeStyle::from_str(s)
}
