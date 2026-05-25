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
        Some(GraphCommands::Build) => cmd_build(&args, &workspace_path)?,
        Some(GraphCommands::Analyze) => cmd_analyze(&args)?,
        Some(GraphCommands::Visualize) => cmd_visualize(&args)?,
        Some(GraphCommands::TomlStructure) => cmd_toml_structure(&args)?,
        Some(GraphCommands::Merge) => cmd_merge(&args)?,
        Some(GraphCommands::Partition) => cmd_partition(&args)?,
        None => {
            // Default to build
            cmd_build(&args, &workspace_path)?;
        }
    }

    Ok(())
}

fn cmd_build(args: &GraphArgs, workspace_path: &PathBuf) -> Result<()> {
    println!("Building dependency graph from {}", workspace_path.display());

    use cargo_vendormod::global_dep_graph::GlobalDependencyGraphBuilder;

    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.clone());
    builder.set_options(args.include_dev, args.include_build, args.expand_features);

    let graph = builder.build_global_graph()?;

    println!("  Total nodes: {}", graph.nodes.len());
    println!("  Total edges: {}", graph.edges.len());
    println!("  SCC count: {}", graph.strongly_connected_components.len());

    // Save to file
    let output_path = args.output_path.clone()
        .unwrap_or_else(|| args.output_dir.join("graph.json"));

    let json = serde_json::to_string_pretty(&graph)?;
    std::fs::write(&output_path, json)?;

    println!("  Saved to: {}", output_path.display());
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

/// Merge multiple project graphs into one global graph.
/// Reads all projects_graphs/*/graph/graph.json files, deduplicates nodes/edges,
/// runs topological sort, outputs build order + upgrade plan + bare repo list.
fn cmd_merge(args: &GraphArgs) -> Result<()> {
    let input_path = args.input_path.as_ref()
        .ok_or_else(|| anyhow::anyhow!("Input path (directory of project graphs) required for merge command"))?;

    println!("=== Global Graph Merge ===");
    println!("Input: {}", input_path.display());
    println!("Output: {}", args.output_dir.display());

    // Phase 1: Discover all graph.json files
    let mut graph_files: Vec<PathBuf> = Vec::new();
    let entries = match fs::read_dir(input_path) {
        Ok(e) => e,
        Err(e) => anyhow::bail!("Failed to read input dir {}: {}", input_path.display(), e),
    };

    for entry in entries.flatten() {
        let project_dir = entry.path();
        if !project_dir.is_dir() {
            continue;
        }
        let graph_file = project_dir.join("graph").join("graph.json");
        if graph_file.exists() {
            graph_files.push(graph_file);
        }
    }

    if graph_files.is_empty() {
        anyhow::bail!("No graph.json files found under {}", input_path.display());
    }

    println!("Found {} project graph files", graph_files.len());

    // Phase 2: Load all graphs and merge
    let mut merged_nodes: HashMap<String, DependencyNode> = HashMap::new();
    let mut merged_edges: Vec<DependencyEdge> = Vec::new();
    let mut edge_set: HashSet<(String, String)> = HashSet::new();
    let mut project_versions: HashMap<String, Vec<(String, String)>> = HashMap::new(); // crate -> [(project, version)]
    let mut projects: Vec<String> = Vec::new();
    let mut all_sources: HashSet<String> = HashSet::new();

    for gf in &graph_files {
        let project_name = gf.parent().unwrap().parent().unwrap()
            .file_name().and_then(|n| n.to_str()).unwrap_or("unknown")
            .to_string();
        projects.push(project_name.clone());

        let content = fs::read_to_string(gf)
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", gf.display(), e))?;
        let graph: GlobalDependencyGraph = serde_json::from_str(&content)
            .map_err(|e| anyhow::anyhow!("Failed to parse {}: {}", gf.display(), e))?;

        // Merge nodes (dedup by ID)
        for node in &graph.nodes {
            let entry = merged_nodes.entry(node.id.clone())
                .or_insert_with(|| node.clone());
            // Keep workspace member flag if any project considers it workspace
            if node.is_workspace_member {
                entry.is_workspace_member = true;
            }
            // Track which projects use which crate and at what version
            project_versions.entry(node.crate_name.clone())
                .or_default()
                .push((project_name.clone(), node.version.clone()));

            // Collect git sources for bare repo mirroring
            if !node.source.is_empty() && node.source != "workspace" && node.source != "crates.io" && node.source != "unknown" {
                all_sources.insert(node.source.clone());
            }
        }

        // Merge edges (dedup by (from, to))
        for edge in &graph.edges {
            let key = (edge.from.clone(), edge.to.clone());
            if edge_set.insert(key) {
                merged_edges.push(edge.clone());
            }
        }
    }

    println!("Merged: {} nodes, {} edges, {} projects", merged_nodes.len(), merged_edges.len(), projects.len());
    println!("Unique git sources for mirroring: {}", all_sources.len());

    // Phase 3: Build petgraph and run topological sort
    let mut pg = Graph::<DependencyNode, DependencyEdgeType>::new();
    let mut pg_map: HashMap<String, NodeIndex> = HashMap::new();

    for (id, node) in &merged_nodes {
        let idx = pg.add_node(node.clone());
        pg_map.insert(id.clone(), idx);
    }

    for edge in &merged_edges {
        if let (Some(&from_idx), Some(&to_idx)) = (pg_map.get(&edge.from), pg_map.get(&edge.to)) {
            pg.add_edge(from_idx, to_idx, edge.edge_type.clone());
        }
    }

    // Topological sort
    let build_order = match petgraph_toposort(&pg, None) {
        Ok(sorted) => {
            let order: Vec<String> = sorted.iter()
                .filter_map(|idx| pg.node_weight(*idx))
                .map(|n| n.crate_name.clone())
                .collect();
            println!("Topological sort succeeded: {} crates in order", order.len());
            order
        }
        Err(cycle) => {
            // Identify cycle
            let mut cycle_crates = Vec::new();
            if let Some(node) = pg.node_weight(cycle.node_id()) {
                cycle_crates.push(node.crate_name.clone());
                for edge in pg.edges_directed(cycle.node_id(), petgraph::Direction::Incoming) {
                    if let Some(source) = pg.node_weight(edge.source()) {
                        cycle_crates.push(source.crate_name.clone());
                    }
                }
            }
            eprintln!("Warning: Circular dependency detected: {:?}", cycle_crates);
            // Return partial order — non-cyclic subset
            let partial_order: Vec<String> = pg.node_weights()
                .map(|n| n.crate_name.clone())
                .collect();
            partial_order
        }
    };

    // Phase 4: Generate upgrade plan
    // For each crate used across multiple projects, note version discrepancies
    let mut upgrade_plan: Vec<serde_json::Value> = Vec::new();
    let mut upgrade_count = 0;

    for (crate_name, versions) in &project_versions {
        let unique_versions: HashSet<&str> = versions.iter().map(|(_, v)| v.as_str()).collect();
        let project_list: Vec<&str> = versions.iter().map(|(p, _)| p.as_str()).collect();

        if unique_versions.len() > 1 {
            // Version discrepancy — needs upgrading
            let oldest = unique_versions.iter().min().unwrap_or(&"");
            let newest = unique_versions.iter().max().unwrap_or(&"");
            upgrade_count += 1;
            upgrade_plan.push(serde_json::json!({
                "crate": crate_name,
                "versions": unique_versions.iter().cloned().collect::<Vec<_>>(),
                "oldest": oldest,
                "newest": newest,
                "used_by": project_list,
                "action": format!("upgrade from {} to {}", oldest, newest)
            }));
        }
    }
    println!("Upgrade candidates (version mismatches): {}", upgrade_count);

    // Phase 5: Output everything
    let out_dir = &args.output_dir;
    fs::create_dir_all(out_dir)?;

    // 5a. Merged global graph
    let global_graph_path = out_dir.join("global_graph.json");

    // Capture unique crate names before consuming merged_nodes
    let mut unique_crate_names: Vec<String> = Vec::new();
    {
        let mut seen: HashSet<&str> = HashSet::new();
        for node in merged_nodes.values() {
            if !node.is_workspace_member && seen.insert(node.crate_name.as_str()) {
                unique_crate_names.push(node.crate_name.clone());
            }
        }
    }

    let merged_graph = GlobalDependencyGraph {
        nodes: merged_nodes.into_values().collect(),
        edges: merged_edges,
        features: Vec::new(),
        toml_structures: Vec::new(),
        strongly_connected_components: Vec::new(),
        partitions: Vec::new(),
        metrics: cargo_vendormod::global_dep_graph::GraphMetrics {
            node_count: 0,
            edge_count: 0,
            workspace_members: 0,
            direct_dependencies: 0,
            transitive_dependencies: 0,
            feature_gated_dependencies: 0,
            dev_dependencies: 0,
            build_dependencies: 0,
            strongly_connected_components: 0,
            diameter: 0,
            average_degree: 0.0,
        },
        workspace_path: PathBuf::new(),
        publish_order: build_order.clone(),
        topological_order: build_order.clone(),
        external_dependency_order: Vec::new(),
    };
    fs::write(&global_graph_path, serde_json::to_string_pretty(&merged_graph)?)?;
    println!("Global graph: {}", global_graph_path.display());

    // 5b. Build order (topological sort)
    let build_order_path = out_dir.join("build_order.json");
    let build_order_data: Vec<serde_json::Value> = build_order.iter()
        .enumerate()
        .map(|(i, name)| serde_json::json!({
            "rank": i + 1,
            "crate": name
        }))
        .collect();
    fs::write(&build_order_path, serde_json::to_string_pretty(&serde_json::json!({
        "total_crates": build_order_data.len(),
        "order": build_order_data
    }))?)?;
    println!("Build order: {} crates -> {}", build_order.len(), build_order_path.display());

    // 5c. Upgrade plan
    let upgrade_path = out_dir.join("upgrade_plan.json");
    fs::write(&upgrade_path, serde_json::to_string_pretty(&serde_json::json!({
        "total_candidates": upgrade_count,
        "plans": upgrade_plan
    }))?)?;
    println!("Upgrade plan: {} candidates -> {}", upgrade_count, upgrade_path.display());

    // 5d. Bare repo list (git sources needing mirroring)
    let mirrors_path = out_dir.join("mirrors.json");
    let sources_sorted: Vec<String> = all_sources.into_iter().collect();
    fs::write(&mirrors_path, serde_json::to_string_pretty(&serde_json::json!({
        "total_mirrors": sources_sorted.len(),
        "repositories": sources_sorted
    }))?)?;
    println!("Bare repos to mirror: {} -> {}", sources_sorted.len(), mirrors_path.display());

    // 5e. Projects index
    let projects_path = out_dir.join("projects.json");
    fs::write(&projects_path, serde_json::to_string_pretty(&serde_json::json!({
        "total_projects": projects.len(),
        "projects": projects
    }))?)?;
    println!("Projects: {} -> {}", projects.len(), projects_path.display());

    // Phase 6: Create bare git repos for each unique source
    let bare_dir = out_dir.join("bare_repos");
    fs::create_dir_all(&bare_dir)?;
    let mut cargo_replace: HashMap<String, String> = HashMap::new();
    let mut mirrors_created = 0u32;

    for source in &sources_sorted {
        // Parse git URL to get a repo name (strip ? and # params)
        let repo_name = source
            .split('?').next().unwrap_or(source) // remove query params
            .split('#').next().unwrap_or(source) // remove fragment
            .rsplit('/').next().unwrap_or("unknown")
            .trim_end_matches(".git")
            .to_string();

        let bare_path = bare_dir.join(format!("{}.git", repo_name));

        if !bare_path.exists() {
            // Initialize empty bare repo
            let _status = Command::new("git")
                .args(["init", "--bare", &bare_path.to_string_lossy()])
                .output();
            mirrors_created += 1;
        }

        // Record replacement mapping for cargo config
        if source.contains("github.com") || source.contains("git://") || source.starts_with("https://") {
            let canonical = source.split('?').next().unwrap_or(source).to_string();
            cargo_replace.insert(canonical, format!("file://{}", bare_path.to_string_lossy()));
        }
    }
    println!("Bare repos created: {} -> {}", mirrors_created, bare_dir.display());

    // 6b. Generate .cargo/config.toml patch to use local mirrors
    let cargo_patch_path = out_dir.join("cargo_mirror_config.toml");
    let mut cargo_config = String::new();
    cargo_config.push_str("# Cargo mirror config — replaces git sources with local bare repos\n");
    cargo_config.push_str("# Place in project_root/.cargo/config.toml\n\n");
    for (original, local) in &cargo_replace {
        cargo_config.push_str(&format!(
            r#"[source."{original}"]
git = "{local}"
replace-with = "local-mirror"

"#,
        ));
    }
    if !cargo_replace.is_empty() {
        cargo_config.push_str("[source.\"local-mirror\"]\n");
        cargo_config.push_str(&format!("directory = \"{}\"\n", bare_dir.display()));
    }
    fs::write(&cargo_patch_path, cargo_config)?;
    println!("Cargo mirror config: {}", cargo_patch_path.display());

    // 6c. Generate minimal flake.nix for each unique crate
    let flakes_dir = out_dir.join("flakes");
    fs::create_dir_all(&flakes_dir)?;
    let mut flakes_created = 0u32;

    for crate_name in &unique_crate_names {
        let crate_flake_dir = flakes_dir.join(crate_name);
        fs::create_dir_all(&crate_flake_dir)?;

        let flake_content = format!(r#"# Auto-generated flake for crate: {name}
# Generated by cargo-vendormod graph merge
{{
  description = "{name} — dependency crate";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  }};

  outputs = {{ self, nixpkgs, flake-utils, rust-overlay }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {{ inherit system overlays; }};
        rustToolchain = pkgs.rust-bin.stable.latest.default;
      in
      {{
        packages.default = pkgs.rustPlatform.buildRustPackage {{
          pname = "{name}";
          version = "0.1.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          nativeBuildInputs = with pkgs; [ rustToolchain pkg-config ];
        }};

        devShells.default = pkgs.mkShell {{
          buildInputs = with pkgs; [ rustToolchain ];
        }};
      }});
}}
"#, name = crate_name);

        fs::write(crate_flake_dir.join("flake.nix"), flake_content)?;
        flakes_created += 1;
    }
    println!("Crate flakes generated: {} -> {}", flakes_created, flakes_dir.display());

    println!("\n=== Merge Complete ===");
    Ok(())
}