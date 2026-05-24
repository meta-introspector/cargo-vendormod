//! # Processing Binary
//!
//! Crate processing - topological processing, compilation, flake generation.

use anyhow::{Context, Result};
use cargo_vendormod::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "process")]
#[command(about = "Crate processing operations", long_about = None)]
struct ProcessingArgs {
    /// Path to vendormod config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Path to workspace root
    #[arg(long)]
    workspace_path: Option<PathBuf>,

    /// Output directory
    #[arg(long, default_value = "./processed")]
    output_dir: PathBuf,

    /// Input file (for process-all-crates)
    #[arg(long)]
    input_file: Option<PathBuf>,

    /// Generate Nix flakes
    #[arg(long)]
    generate_flakes: bool,

    /// Compile crates standalone
    #[arg(long)]
    compile_standalone: bool,

    /// Use layered processing
    #[arg(long)]
    layered_processing: bool,

    /// Process specific layer only (1=external, 2=workspace)
    #[arg(long)]
    layer: Option<u32>,

    /// Maximum parallel jobs
    #[arg(long, default_value = "4")]
    max_parallel: usize,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<ProcessingCommands>,
}

#[derive(Subcommand, Debug)]
enum ProcessingCommands {
    /// Process crates from workspace
    Crates,
    /// Process all crates from file list
    All,
    /// Run complete workflow
    Workflow,
    /// Generate processing report
    Report,
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
        None => {
            cmd_crates(&args, &config, &workspace_path)?;
        }
    }

    Ok(())
}

fn cmd_crates(args: &ProcessingArgs, _config: &Config, workspace_path: &PathBuf) -> Result<()> {
    println!("Processing crates from {}", workspace_path.display());
    println!("  Output: {}", args.output_dir.display());
    println!("  Flakes: {}", args.generate_flakes);
    println!("  Standalone: {}", args.compile_standalone);
    println!("  Layered: {}", args.layered_processing);
    println!("  Parallel: {}", args.max_parallel);

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
                    generate_flake(&crate_dir, &node.crate_name, &args.output_dir)?;
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
            let crate_dir = workspace_path.join(&node.id);
            if crate_dir.exists() {
                if args.generate_flakes {
                    generate_flake(&crate_dir, &node.crate_name, &args.output_dir)?;
                }
            }
        }
    }

    println!("\nProcessing complete.");
    Ok(())
}

fn generate_flake(_crate_dir: &PathBuf, crate_name: &str, output_dir: &PathBuf) -> Result<()> {
    let flake_dir = output_dir.join("flakes").join(crate_name);
    fs::create_dir_all(&flake_dir)?;

    let flake_content = format!(r#"
{{
  description = "{}";

  outputs = {{ self, nixpkgs }}: {{
    packages.x86_64-linux.default = nixpkgs.mkShell {{
      buildInputs = with nixpkgs; [
        rustc
        cargo
      ];
    }};
  }};
}}
"#, crate_name);

    fs::write(flake_dir.join("flake.nix"), flake_content)?;
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

    use rayon::prelude::*;

    let output_dir = args.output_dir.clone();
    let verbose = args.verbose;
    let generate_flakes = args.generate_flakes;

    crate_paths.par_iter().for_each(|crate_path| {
        if verbose {
            println!("  Processing: {}", crate_path.display());
        }

        // Process each crate
        if generate_flakes {
            let crate_name = crate_path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            
            if let Err(e) = generate_flake(crate_path, crate_name, &output_dir) {
                eprintln!("  Failed to generate flake for {}: {}", crate_name, e);
            }
        }
    });

    println!("\nAll crates processed.");
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