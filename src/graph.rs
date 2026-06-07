use anyhow::Result;
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs;
use crate::global_dep_graph::{DependencyEdge, DependencyEdgeType, DependencyNode, GlobalDependencyGraph};
use petgraph::algo::toposort as petgraph_toposort;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::EdgeRef;

pub fn build_graph(workspace_path: &Path, output_dir: &Path, include_dev: bool, include_build: bool, expand_features: bool) -> Result<()> {
    println!("Building dependency graph from {}", workspace_path.display());

    use crate::global_dep_graph::GlobalDependencyGraphBuilder;

    let mut builder = GlobalDependencyGraphBuilder::new(workspace_path.to_path_buf());
    builder.set_options(include_dev, include_build, expand_features);

    let graph = builder.build_global_graph()?;

    println!("  Total nodes: {}", graph.nodes.len());
    println!("  Total edges: {}", graph.edges.len());
    println!("  SCC count: {}", graph.strongly_connected_components.len());

    // Save to file
    let output_path = output_dir.join("graph.json");

    let json = serde_json::to_string_pretty(&graph)?;
    std::fs::write(&output_path, json)?;

    println!("  Saved to: {}", output_path.display());
    Ok(())
}

pub fn merge_graphs(input_path: &Path, output_dir: &Path) -> Result<()> {
    println!("=== Global Graph Merge ===");
    println!("Input: {}", input_path.display());
    println!("Output: {}", output_dir.display());

    // Phase 1: Discover all graph.json files recursively
    let mut graph_files: Vec<PathBuf> = Vec::new();
    for entry in walkdir::WalkDir::new(input_path) {
        let entry = entry?;
        if entry.file_type().is_file() && entry.file_name() == "graph.json" {
            graph_files.push(entry.path().to_path_buf());
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
        let project_name = gf.parent().unwrap()
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
    let out_dir = &output_dir;
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
        metrics: crate::global_dep_graph::GraphMetrics {
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
    Ok(())
}

pub fn analyze_cbor_usage(graph: &GlobalDependencyGraph) -> Result<HashSet<String>> {
    let mut cbor_dependent_crates = HashSet::new();
    let target_crates = vec!["serde_ipld_dagcbor", "serde_ipld_dagcbor-escaped"];

    // Find the nodes for the target CBOR crates
    let cbor_node_ids: Vec<String> = graph.nodes.iter()
        .filter(|node| target_crates.contains(&node.crate_name.as_str()))
        .map(|node| node.id.clone())
        .collect();

    if cbor_node_ids.is_empty() {
        return Ok(cbor_dependent_crates);
    }

    // Iterate through all nodes and check their dependencies
    for node in &graph.nodes {
        // If the node itself is a target CBOR crate, add it
        if target_crates.contains(&node.crate_name.as_str()) {
            cbor_dependent_crates.insert(node.crate_name.clone());
        }

        // Check if any of its direct dependencies are CBOR crates
        for edge in &graph.edges {
            if edge.from == node.id && cbor_node_ids.contains(&edge.to) {
                cbor_dependent_crates.insert(node.crate_name.clone());
            }
        }
    }

    Ok(cbor_dependent_crates)
}

pub fn scan_cbor_usage_in_source(
    repo: &git2::Repository,
    global_graph: &GlobalDependencyGraph,
    cbor_dependent_crates: &HashSet<String>,
) -> Result<HashMap<String, Vec<String>>> {
    let mut usage_details: HashMap<String, Vec<String>> = HashMap::new();

    // Build a map from submodule path (as string) to crate_name (from graph nodes)
    let mut path_to_crate_name: HashMap<PathBuf, String> = HashMap::new();
    for node in &global_graph.nodes {
        if node.is_workspace_member {
            // Extract the path from the node ID
            // Example node.id: "crate:pkg:path+file:///mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core#libipld-core@0.4.3"
            if let Some(path_str) = node.id.strip_prefix("crate:pkg:path+file://") {
                if let Some(repo_root_str) = repo.workdir().and_then(|p| p.to_str()) {
                    if let Some(relative_path) = path_str.strip_prefix(repo_root_str).and_then(|s| s.strip_prefix('/')) {
                        if let Some(sub_path_str) = relative_path.split('#').next() {
                            path_to_crate_name.insert(PathBuf::from(sub_path_str), node.crate_name.clone());
                        }
                    }
                }
            }
        }
    }

    for submodule in repo.submodules()? {
        let submodule_path_rel = submodule.path();
        if let Some(crate_name) = path_to_crate_name.get(submodule_path_rel) {
            if cbor_dependent_crates.contains(crate_name) {
                let submodule_abs_path = repo.workdir().unwrap().join(submodule_path_rel);
                let mut found_in_files = Vec::new();

                for entry in walkdir::WalkDir::new(&submodule_abs_path) {
                    let entry = entry?;
                    if entry.file_type().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs") {
                        let content = std::fs::read_to_string(entry.path())?;
                        if content.contains("serde_ipld_dagcbor::encode") || content.contains("serde_ipld_dagcbor::decode") {
                            found_in_files.push(entry.path().strip_prefix(&submodule_abs_path)?.display().to_string());
                        }
                    }
                }
                if !found_in_files.is_empty() {
                    usage_details.insert(crate_name.clone(), found_in_files);
                }
            }
        }
    }

    Ok(usage_details)
}
