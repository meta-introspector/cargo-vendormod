//! # Graph Binary
//!
//! Dependency graph analysis - build, analyze, visualize, and partition dependency graphs.

use anyhow::Result;
use cargo_vendormod::config::Config;
use cargo_vendormod::global_dep_graph::{DependencyEdge, DependencyEdgeType, DependencyNode, GlobalDependencyGraph};
use clap::{Parser, Subcommand};
use petgraph::algo::toposort as petgraph_toposort;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "graph")]
#[command(about = "Dependency graph operations", long_about = None)]
struct GraphArgs {
    /// Path to vendormod config file
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    /// Path to workspace root
    #[arg(long, global = true, short = 'w')]
    workspace_path: Option<PathBuf>,

    /// Input file for analysis commands
    #[arg(long, global = true, short = 'i')]
    input_path: Option<PathBuf>,

    /// Output directory
    #[arg(long, global = true, short = 'o', default_value = "./analysis")]
    output_dir: PathBuf,

    /// Output file
    #[arg(long, global = true, short = 'O')]
    output_path: Option<PathBuf>,

    /// Include dev dependencies
    #[arg(long, global = true)]
    include_dev: bool,

    /// Include build dependencies  
    #[arg(long, global = true)]
    include_build: bool,

    /// Expand features
    #[arg(long, global = true)]
    expand_features: bool,

    /// Partition count for graph partitioning
    #[arg(long, global = true, default_value = "8")]
    partition_count: usize,

    /// Partitioning algorithm
    #[arg(long, global = true, default_value = "kaminpar")]
    algorithm: String,

    /// Verbose output
    #[arg(long, global = true, short = 'v')]
    verbose: bool,

    #[command(subcommand)]
    command: Option<GraphCommands>,
}

#[derive(Subcommand, Debug)]
enum GraphCommands {
    /// Build dependency graph from workspace
    Build,
    /// Analyze graph structure and patterns
    Analyze,
    /// Generate DOT visualization
    Visualize,
    /// Analyze TOML structure patterns
    TomlStructure,
    /// Partition graph for parallel processing
    Partition,
    /// Merge multiple project graphs into one global graph
    Merge,
}

fn main() -> Result<()> {
    let args = GraphArgs::parse();
    let _config = Config::load(args.config.as_deref())?;

    let workspace_path = args.workspace_path.clone()
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current dir"));

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    match args.command {
        Some(GraphCommands::Build) => cargo_vendormod::graph::build_graph(&workspace_path, &args.output_dir, args.include_dev, args.include_build, args.expand_features)?,
        Some(GraphCommands::Analyze) => cmd_analyze(&args)?,
        Some(GraphCommands::Visualize) => cmd_visualize(&args)?,
        Some(GraphCommands::TomlStructure) => cmd_toml_structure(&args)?,
        Some(GraphCommands::Merge) => cargo_vendormod::graph::merge_graphs(args.input_path.as_ref().unwrap(), &args.output_dir)?,
        Some(GraphCommands::Partition) => cmd_partition(&args)?,
        None => {
            // Default to build
            cargo_vendormod::graph::build_graph(&workspace_path, &args.output_dir, args.include_dev, args.include_build, args.expand_features)?;
        }
    }

    Ok(())
}

fn cmd_analyze(args: &GraphArgs) -> Result<()> {
    let input_path = args.input_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Input path required for analyze command"))?;

    println!("Analyzing graph from {}", input_path.display());

    let content = std::fs::read_to_string(&input_path)?;
    let graph: cargo_vendormod::global_dep_graph::GlobalDependencyGraph = serde_json::from_str(&content)?;

    // Print metrics
    println!("\n=== Graph Metrics ===");
    println!("Nodes: {}", graph.nodes.len());
    println!("Edges: {}", graph.edges.len());

    // Analyze common patterns
    println!("\n=== Pattern Analysis ===");

    // Count dependency types
    let mut direct = 0;
    let mut transitive = 0;
    let mut dev = 0;

    for edge in &graph.edges {
        use cargo_vendormod::global_dep_graph::DependencyEdgeType;
        match edge.edge_type {
            DependencyEdgeType::Direct => direct += 1,
            DependencyEdgeType::Transitive => transitive += 1,
            DependencyEdgeType::Dev => dev += 1,
            _ => {}
        }
    }

    println!("Direct dependencies: {}", direct);
    println!("Transitive dependencies: {}", transitive);
    println!("Dev dependencies: {}", dev);

    // Save analysis report
    let output_path = args.output_dir.join("analysis.json");
    let report = serde_json::json!({
        "total_nodes": graph.nodes.len(),
        "total_edges": graph.edges.len(),
        "scc_count": graph.strongly_connected_components.len(),
        "direct_edges": direct,
        "transitive_edges": transitive,
        "dev_edges": dev,
    });
    std::fs::write(&output_path, serde_json::to_string_pretty(&report)?)?;

    println!("\nAnalysis saved to: {}", output_path.display());
    Ok(())
}

fn cmd_visualize(args: &GraphArgs) -> Result<()> {
    let input_path = args.input_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Input path required for visualize command"))?;

    let output_path = args.output_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Output path required for visualize command"))?;

    println!("Generating visualization from {}", input_path.display());

    let content = std::fs::read_to_string(&input_path)?;
    let graph: cargo_vendormod::global_dep_graph::GlobalDependencyGraph = serde_json::from_str(&content)?;

    // Generate DOT format
    let dot = generate_dot(&graph);
    std::fs::write(&output_path, dot)?;

    println!("DOT file saved to: {}", output_path.display());
    println!("Convert to SVG with: dot -Tsvg {} -o graph.svg", output_path.display());
    Ok(())
}

fn generate_dot(graph: &cargo_vendormod::global_dep_graph::GlobalDependencyGraph) -> String {
    let mut dot = String::new();
    dot.push_str("digraph dependencies {\n");
    dot.push_str("  rankdir=LR;\n");
    dot.push_str("  node [shape=box];\n\n");

    // Add nodes
    for node in &graph.nodes {
        dot.push_str(&format!("  \"{}\" [label=\"{}\\n{}\"];\n",
            node.id, node.crate_name, node.version));
    }

    // Add edges
    for edge in &graph.edges {
        dot.push_str(&format!("  \"{}\" -> \"{}\";\n", edge.from, edge.to));
    }

    dot.push_str("}\n");
    dot
}

fn cmd_toml_structure(args: &GraphArgs) -> Result<()> {
    let input_path = args.input_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Input path required for toml-structure command"))?;

    println!("Analyzing TOML structures from {}", input_path.display());

    let content = std::fs::read_to_string(&input_path)?;
    let graph: cargo_vendormod::global_dep_graph::GlobalDependencyGraph = serde_json::from_str(&content)?;

    // Analyze TOML patterns
    let mut workspace_crates = 0;
    let mut binary_crates = 0;
    let mut lib_crates = 0;
    let mut has_tests = 0;
    let mut has_benches = 0;

    for structure in &graph.toml_structures {
        if structure.is_workspace { workspace_crates += 1; }
        if structure.has_binaries { binary_crates += 1; }
        if !structure.has_binaries { lib_crates += 1; }
        if structure.has_tests { has_tests += 1; }
        if structure.has_benchmarks { has_benches += 1; }
    }

    println!("\n=== TOML Structure Analysis ===");
    println!("Workspace crates: {}", workspace_crates);
    println!("Binary crates: {}", binary_crates);
    println!("Library crates: {}", lib_crates);
    println!("Crates with tests: {}", has_tests);
    println!("Crates with benchmarks: {}", has_benches);

    // Save report
    let output_path = args.output_dir.join("toml_structure.json");
    let report = serde_json::json!({
        "workspace_crates": workspace_crates,
        "binary_crates": binary_crates,
        "library_crates": lib_crates,
        "crates_with_tests": has_tests,
        "crates_with_benches": has_benches,
    });
    std::fs::write(&output_path, serde_json::to_string_pretty(&report)?)?;

    println!("\nTOML analysis saved to: {}", output_path.display());
    Ok(())
}

fn cmd_partition(args: &GraphArgs) -> Result<()> {
    let input_path = args.input_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Input path required for partition command"))?;

    println!("Partitioning graph from {}", input_path.display());
    println!("  Partitions: {}", args.partition_count);
    println!("  Algorithm: {}", args.algorithm);

    let content = std::fs::read_to_string(&input_path)?;
    let graph: cargo_vendormod::global_dep_graph::GlobalDependencyGraph = serde_json::from_str(&content)?;

    // Simple partitioning by node index
    let partition_size = graph.nodes.len() / args.partition_count.max(1);
    let mut partitions: Vec<Vec<String>> = (0..args.partition_count).map(|_| Vec::new()).collect();

    for (i, node) in graph.nodes.iter().enumerate() {
        let partition_idx = (i / partition_size).min(args.partition_count - 1);
        partitions[partition_idx].push(node.id.clone());
    }

    // Save partition info
    let output_path = args.output_dir.join("partitions.json");
    let partition_data: Vec<serde_json::Value> = partitions.iter().enumerate()
        .map(|(i, nodes)| serde_json::json!({
            "partition_id": i,
            "node_count": nodes.len(),
            "nodes": nodes
        }))
        .collect();

    std::fs::write(&output_path, serde_json::to_string_pretty(&partition_data)?)?;

    println!("\nPartitioning complete:");
    for (i, p) in partitions.iter().enumerate() {
        println!("  Partition {}: {} nodes", i, p.len());
    }
    println!("\nPartitions saved to: {}", output_path.display());
    Ok(())
}
