use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::dot::{Dot, Config};
use petgraph::algo::{tarjan_scc, toposort};
use toml_edit::DocumentMut;
use petgraph::visit::EdgeRef;
use petgraph::Direction;

use crate::cargo_tool_discovery::CargoToolDiscovery;

// Add cargo-metadata import for direct dependency resolution
use cargo_metadata::{MetadataCommand, Package, DependencyKind};

/// Global dependency graph representing the entire Solana universe
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDependencyGraph {
    pub nodes: Vec<DependencyNode>,
    pub edges: Vec<DependencyEdge>,
    pub features: Vec<FeatureSet>,
    pub toml_structures: Vec<TomlStructure>,
    pub strongly_connected_components: Vec<Vec<String>>,
    pub partitions: Vec<GraphPartition>,
    pub metrics: GraphMetrics,
    pub workspace_path: PathBuf,
    
    // Topological order information
    pub publish_order: Vec<String>,
    pub topological_order: Vec<String>,
    pub external_dependency_order: Vec<String>,
}

/// Node in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyNode {
    pub id: String,
    pub crate_name: String,
    pub version: String,
    pub source: String,
    pub is_workspace_member: bool,
    pub is_direct_dependency: bool,
    pub features: HashSet<String>,
    pub categories: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Edge in the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub from: String,
    pub to: String,
    pub edge_type: DependencyEdgeType,
    pub required_features: HashSet<String>,
    pub optional_features: HashSet<String>,
    pub is_dev_dependency: bool,
    pub is_build_dependency: bool,
    pub properties: HashMap<String, String>,
}

/// Type of dependency edge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DependencyEdgeType {
    Direct,
    Transitive,
    FeatureGated,
    Dev,
    Build,
    Workspace,
    Virtual,
}

/// TOML structure representing the schema and patterns found in Cargo.toml files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlStructure {
    pub crate_name: String,
    pub crate_version: String,
    pub file_path: String,
    pub schema_elements: Vec<SchemaElement>,
    pub workload_patterns: Vec<WorkloadPattern>,
    pub is_workspace: bool,
    pub has_binaries: bool,
    pub has_tests: bool,
    pub has_benchmarks: bool,
}

/// Element in the TOML schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaElement {
    pub element_type: SchemaElementType,
    pub path: String,
    pub value_type: String,
    pub is_array: bool,
    pub is_table: bool,
    pub is_inline_table: bool,
    pub children: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Type of schema element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SchemaElementType {
    Package,
    Dependency,
    DevDependency,
    BuildDependency,
    Feature,
    Binary,
    Benchmark,
    Test,
    Profile,
    Metadata,
    Workspace,
    Patch,
    Replace,
    Lib,
    Target,
    Custom,
}

/// Workload-specific pattern identified in TOML structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkloadPattern {
    pub pattern_type: WorkloadPatternType,
    pub name: String,
    pub location: String,
    pub related_elements: Vec<String>,
    pub properties: HashMap<String, String>,
}

/// Type of workload pattern
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WorkloadPatternType {
    BinaryTarget,
    TestSuite,
    BenchmarkSuite,
    FeatureFlag,
    ConditionalCompilation,
    WorkspaceMember,
    DependencyOverride,
    ProfileOptimization,
    MetadataExtension,
    CustomBuildScript,
}

/// Graph partition created by partitioning algorithm
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPartition {
    pub partition_id: usize,
    pub node_ids: Vec<String>,
    pub node_count: usize,
    pub edge_count: usize,
    pub is_core: bool,
    pub properties: HashMap<String, String>,
    pub metrics: PartitionMetrics,
}

/// Metrics for a single partition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionMetrics {
    pub internal_edges: usize,
    pub external_edges: usize,
    pub density: f64,
    pub modularity: f64,
    pub cohesion: f64,
    pub coupling: f64,
}

/// Partitioning configuration
#[derive(Debug, Clone)]
pub struct PartitioningConfig {
    pub partition_count: usize,
    pub balance_factor: f64,
    pub algorithm: PartitioningAlgorithm,
    pub output_format: PartitionOutputFormat,
}

/// Partitioning algorithm to use
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitioningAlgorithm {
    KaMinPar,
    Metis,
    Louvain,
    KernighanLin,
    Spectral,
    Greedy,
}

impl std::str::FromStr for PartitioningAlgorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "kaminpar" => Ok(PartitioningAlgorithm::KaMinPar),
            "metis" => Ok(PartitioningAlgorithm::Metis),
            "louvain" => Ok(PartitioningAlgorithm::Louvain),
            "kernighanlin" => Ok(PartitioningAlgorithm::KernighanLin),
            "spectral" => Ok(PartitioningAlgorithm::Spectral),
            "greedy" => Ok(PartitioningAlgorithm::Greedy),
            _ => Err(format!("Unknown partitioning algorithm: {}", s)),
        }
    }
}

/// Output format for partitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PartitionOutputFormat {
    Json,
    Dot,
    Svg,
    DotAndSvg,
    All,
}

/// Feature set for a crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSet {
    pub crate_name: String,
    pub features: HashSet<String>,
    pub default_features: HashSet<String>,
    pub feature_dependencies: HashMap<String, HashSet<String>>, // feature -> enabled crates
}

/// Graph metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    pub node_count: usize,
    pub edge_count: usize,
    pub workspace_members: usize,
    pub direct_dependencies: usize,
    pub transitive_dependencies: usize,
    pub feature_gated_dependencies: usize,
    pub dev_dependencies: usize,
    pub build_dependencies: usize,
    pub strongly_connected_components: usize,
    pub diameter: usize,
    pub average_degree: f64,
}

/// Global dependency graph builder
pub struct GlobalDependencyGraphBuilder {
    graph: Graph<DependencyNode, DependencyEdge>,
    node_map: HashMap<String, NodeIndex>,
    workspace_path: PathBuf,
    cargo_tools: Option<CargoToolDiscovery>,
    include_dev_dependencies: bool,
    include_build_dependencies: bool,
    expand_features: bool,
    workspace_members: HashSet<String>,
}

impl GlobalDependencyGraphBuilder {
    /// Create a new GlobalDependencyGraphBuilder
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
            workspace_path,
            cargo_tools: None, // Disable cargo tools by default to avoid reqwest dependency
            include_dev_dependencies: true,
            include_build_dependencies: true,
            expand_features: true,
            workspace_members: HashSet::new(),
        }
    }
    
    /// Set dependency inclusion options
    pub fn set_options(&mut self, include_dev: bool, include_build: bool, expand_features: bool) -> &mut Self {
        self.include_dev_dependencies = include_dev;
        self.include_build_dependencies = include_build;
        self.expand_features = expand_features;
        self
    }
    
    /// Build the global dependency graph
    pub fn build_global_graph(&mut self) -> Result<GlobalDependencyGraph> {
        println!("Building global dependency graph for Solana universe...");
        
        // Step 1: Discover workspace structure
        self.discover_workspace_members()?;
        
        // Step 2: Analyze direct dependencies
        self.analyze_direct_dependencies()?;
        
        // Step 3: Resolve transitive dependencies (removed - using cargo-metadata instead)
        // self.resolve_transitive_dependencies()?;
        
        // Step 4: Expand feature dependencies
        if self.expand_features {
            self.expand_feature_dependencies()?;
        }
        
        // Step 5: Calculate metrics
        let metrics = self.calculate_metrics()?;
        
        // Step 6: Find strongly connected components
        let scc = self.find_strongly_connected_components()?;
        
        // Step 7: Extract graph data
        let nodes: Vec<DependencyNode> = self.graph.node_weights().cloned().collect();
        let edges: Vec<DependencyEdge> = self.graph.edge_weights().cloned().collect();
        
        // Step 8: Collect feature information
        let features = self.collect_feature_information()?;
        
        // Step 9: Analyze TOML structures
        let toml_structures = self.analyze_toml_structures()?;
        
        // Step 10: Calculate topological orders
        let publish_order = self.get_publish_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate publish order: {}", e);
            Vec::new()
        });
        
        let topological_order = self.get_topological_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate topological order: {}", e);
            Vec::new()
        });
        
        let external_dependency_order = self.get_external_dependency_order().unwrap_or_else(|e| {
            eprintln!("Warning: Could not calculate external dependency order: {}", e);
            Vec::new()
        });
        
        Ok(GlobalDependencyGraph {
            nodes,
            edges,
            features,
            toml_structures,
            strongly_connected_components: scc,
            partitions: Vec::new(),
            metrics,
            workspace_path: self.workspace_path.clone(),
            
            // Add topological order information
            publish_order,
            topological_order,
            external_dependency_order,
        })
    }
    
    /// Discover workspace members using cargo metadata (pure Rust, no shell commands).
    fn discover_workspace_members(&mut self) -> Result<()> {
        println!("Discovering workspace members via cargo metadata...");
        
        // Use cargo metadata to find workspace packages (source == None means local/workspace)
        match self.get_workspace_metadata() {
            Ok(metadata) => {
                for package in &metadata.packages {
                    if package.source.is_none() {
                        // Local/workspace package
                        let cargo_toml_path = package.manifest_path.as_std_path();
                        self.add_workspace_member(
                            &package.name,
                            &package.version.to_string(),
                            false,
                            cargo_toml_path,
                        )?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: cargo metadata failed ({}), falling back to filesystem discovery", e);
                self.discover_workspace_members_fallback()?;
            }
        }
        
        println!("Added {} workspace members via cargo metadata", self.workspace_members.len());
        Ok(())
    }
    
    /// Fallback workspace discovery by reading Cargo.toml directly (no shell commands).
    fn discover_workspace_members_fallback(&mut self) -> Result<()> {
        let cargo_toml = self.workspace_path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Ok(());
        }
        let content = fs::read_to_string(&cargo_toml)
            .context("Failed to read Cargo.toml")?;
        let doc = content.parse::<toml_edit::Document>()
            .context("Failed to parse Cargo.toml")?;
        
        // Add root package
        if let Some(package) = doc.get("package") {
            let name = package["name"].as_str().unwrap_or("unknown").to_string();
            let version = package["version"].as_str().unwrap_or("0.0.0").to_string();
            self.add_workspace_member(&name, &version, true, &cargo_toml)?;
        }
        
        // Add workspace members
        if let Some(workspace_table) = doc.get("workspace") {
            if let Some(members) = workspace_table.get("members").and_then(|m| m.as_array()) {
                for member in members {
                    if let Some(member_str) = member.as_str() {
                        self.discover_member_crate(&cargo_toml, member_str)?;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Discover a workspace member crate by its glob/member pattern.
    fn discover_member_crate(&mut self, manifest_path: &Path, member_str: &str) -> Result<()> {
        let workspace_dir = manifest_path.parent().unwrap();
        let member_path = workspace_dir.join(member_str);
        
        // Try the path directly
        let candidate = if member_path.is_dir() {
            member_path.join("Cargo.toml")
        } else if member_path.ends_with("Cargo.toml") {
            member_path
        } else {
            return Ok(()); // glob pattern, skip for fallback
        };
        
        if candidate.exists() {
            let content = fs::read_to_string(&candidate)?;
            let doc = content.parse::<toml_edit::Document>()?;
            if let Some(package) = doc.get("package") {
                let name = package["name"].as_str().unwrap_or("unknown").to_string();
                let version = package["version"].as_str().unwrap_or("0.0.0").to_string();
                self.add_workspace_member(&name, &version, false, &candidate)?;
            }
        }
        
        Ok(())
    }
    
    /// Process a Rust repository with Cargo.toml
    fn process_rust_repository(&mut self, repo_path: &Path, cargo_toml: &Path) -> Result<()> {
        let content = fs::read_to_string(cargo_toml)
            .context("Failed to read Cargo.toml")?;
        
        let doc = content.parse::<toml_edit::Document>()
            .context("Failed to parse Cargo.toml")?;
        
        // Extract package information
        if let Some(package) = doc.get("package") {
            let name = package["name"]
                .as_str()
                .unwrap_or("unknown")
                .to_string();
            let version = package["version"]
                .as_str()
                .unwrap_or("0.0.0")
                .to_string();
            
            // Add as workspace member
            self.add_workspace_member(&name, &version, false, cargo_toml)?;
            
            println!("Added Rust repository: {} (v{})", name, version);
        }
        
        // Also check for workspace definition in this file
        if let Some(workspace_table) = doc.get("workspace") {
            if let Some(members) = workspace_table.get("members") {
                if let Some(member_array) = members.as_array() {
                    for member in member_array {
                        if let Some(member_str) = member.as_str() {
                            let member_path = repo_path.join(member_str);
                            let member_cargo_toml = if member_path.is_dir() {
                                member_path.join("Cargo.toml")
                            } else if member_path.ends_with("Cargo.toml") {
                                member_path
                            } else {
                                member_path.with_extension("toml")
                            };
                            
                            if member_cargo_toml.exists() {
                                if let Err(e) = self.process_rust_repository(repo_path, &member_cargo_toml) {
                                    eprintln!("Warning: Failed to process member {}: {}", member_str, e);
                                }
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Process a non-Rust repository
    fn process_non_rust_repository(&mut self, repo_path: &Path) -> Result<()> {
        // Extract repository name from path
        let repo_name = repo_path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        
        // Create a synthetic package name for non-Rust repositories
        let synthetic_name = format!("git-{}", repo_name);
        
        // Add as workspace member with special marker
        self.add_workspace_member(&synthetic_name, "0.0.0-non-rust", false, repo_path)?;
        
        println!("Added non-Rust repository: {}", synthetic_name);
        
        // TODO: Add language detection and processing for non-Rust repositories
        // This could include:
        // - Go repositories (go.mod)
        // - JavaScript/TypeScript (package.json)
        // - Python (requirements.txt/pyproject.toml)
        // - Other package management files
        
        Ok(())
    }
    
    /// Add a workspace member to the graph
    fn add_workspace_member(&mut self, name: &str, version: &str, is_root: bool, cargo_toml: &Path) -> Result<()> {
        let node_id = format!("workspace:{}", name);
        
        if self.node_map.contains_key(&node_id) {
            return Ok(()); // Already added
        }
        
        let node = DependencyNode {
            id: node_id.clone(),
            crate_name: name.to_string(),
            version: version.to_string(),
            source: "workspace".to_string(),
            is_workspace_member: true,
            is_direct_dependency: is_root,
            features: HashSet::new(),
            categories: vec!["workspace".to_string()],
            properties: [
                ("path".to_string(), cargo_toml.parent().unwrap().display().to_string()),
                ("is_root".to_string(), is_root.to_string()),
            ].iter().cloned().collect(),
        };
        
        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        self.workspace_members.insert(name.to_string());

        println!("Added workspace member: {} v{}", name, version);
        Ok(())
    }
    
    /// Analyze direct dependencies using cargo-metadata (more reliable than cargo-tree)
    fn analyze_direct_dependencies(&mut self) -> Result<()> {
        println!("Analyzing dependencies using cargo-metadata resolve.nodes...");
        
        match self.get_workspace_metadata() {
            Ok(metadata) => {
                // Build a lookup from PackageId to Package
                // cargo_metadata::PackageId implements Hash + Eq + Display
                let resolve = match &metadata.resolve {
                    Some(r) => r,
                    None => {
                        eprintln!("Warning: cargo metadata has no resolve graph (use --no-deps?)");
                        return Ok(());
                    }
                };
                
                let pkg_by_id: HashMap<&cargo_metadata::PackageId, &Package> = metadata.packages
                    .iter()
                    .map(|p| (&p.id, p))
                    .collect();
                
                // Phase 1: Add ALL resolved packages as nodes, using PackageId as unique key
                for node in &resolve.nodes {
                    let package = match pkg_by_id.get(&node.id) {
                        Some(p) => p,
                        None => {
                            eprintln!("Warning: resolve node {} not found in packages", node.id);
                            continue;
                        }
                    };
                    
                    let is_ws = package.source.is_none();
                    let node_id = format!("pkg:{}", node.id);
                    
                    // Normalize source string
                    let source_str = if let Some(src) = &package.source {
                        let s = src.to_string();
                        if s.starts_with("git+") { s[4..].to_string() }
                        else if s.starts_with("registry+") { "crates.io".to_string() }
                        else { s }
                    } else {
                        "workspace".to_string()
                    };
                    
                    // Only add if not already present (from discover_workspace_members)
                    if !self.node_map.contains_key(&node_id) {
                        let idx = self.graph.add_node(DependencyNode {
                            id: node_id.clone(),
                            crate_name: package.name.clone(),
                            version: package.version.to_string(),
                            source: source_str,
                            is_workspace_member: is_ws,
                            is_direct_dependency: false,
                            features: node.features.iter().cloned().collect(),
                            categories: vec!["dependency".to_string()],
                            properties: HashMap::new(),
                        });
                        self.node_map.insert(node_id.clone(), idx);
                    }
                }
                
                // Phase 2: Add edges from resolve.nodes[*].dependencies
                for node in &resolve.nodes {
                    let parent_id = format!("pkg:{}", node.id);
                    let parent_idx = match self.node_map.get(&parent_id) {
                        Some(idx) => *idx,
                        None => continue,
                    };
                    
                    for dep_pkg_id in &node.dependencies {
                        let dep_id = format!("pkg:{}", dep_pkg_id);
                        
                        // Ensure dep node exists
                        if !self.node_map.contains_key(&dep_id) {
                            // Try to find the package info
                            if let Some(package) = pkg_by_id.get(dep_pkg_id) {
                                let is_ws = package.source.is_none();
                                let source_str = if let Some(src) = &package.source {
                                    let s = src.to_string();
                                    if s.starts_with("git+") { s[4..].to_string() }
                                    else if s.starts_with("registry+") { "crates.io".to_string() }
                                    else { s }
                                } else {
                                    "workspace".to_string()
                                };
                                
                                let idx = self.graph.add_node(DependencyNode {
                                    id: dep_id.clone(),
                                    crate_name: package.name.clone(),
                                    version: package.version.to_string(),
                                    source: source_str,
                                    is_workspace_member: is_ws,
                                    is_direct_dependency: false,
                                    features: package.features.keys().cloned().collect(),
                                    categories: vec!["dependency".to_string()],
                                    properties: HashMap::new(),
                                });
                                self.node_map.insert(dep_id.clone(), idx);
                            } else {
                                continue; // Skip unknown deps
                            }
                        }
                        
                        let dep_idx = self.node_map[&dep_id];
                        
                        // Avoid duplicating edges
                        let has_edge = self.graph.edges(parent_idx)
                            .any(|e| e.target() == dep_idx);
                        
                        if !has_edge {
                            self.graph.add_edge(parent_idx, dep_idx, DependencyEdge {
                                from: parent_id.clone(),
                                to: dep_id,
                                edge_type: DependencyEdgeType::Direct,
                                required_features: HashSet::new(),
                                optional_features: HashSet::new(),
                                is_dev_dependency: false,
                                is_build_dependency: false,
                                properties: HashMap::new(),
                            });
                        }
                    }
                }
                
                println!("  Total nodes: {}", self.graph.node_count());
                println!("  Total edges: {}", self.graph.edge_count());
            }
            Err(e) => {
                eprintln!("Warning: Failed to get workspace metadata: {}, falling back to Cargo.lock parser", e);
                self.analyze_from_lockfile()?;
            }
        }
        
        Ok(())
    }
    
    /// Pure-Rust fallback: parse Cargo.lock directly instead of shelling out to `cargo metadata`.
    ///
    /// Reads `Cargo.lock` via `toml_edit::DocumentMut`, extracts `[[package]]` entries
    /// as nodes and their `dependencies` as edges.  Works offline, no `cargo` needed.
    fn analyze_from_lockfile(&mut self) -> Result<()> {
        println!("Analyzing dependencies from Cargo.lock (pure Rust)...");

        let lock_path = self.workspace_path.join("Cargo.lock");
        let lock_content = match fs::read_to_string(&lock_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Warning: Cannot read Cargo.lock at {:?}: {}", lock_path, e);
                eprintln!("Falling back to cargo-tree method...");
                return self.analyze_direct_dependencies_fallback();
            }
        };

        let lock_doc: DocumentMut = match lock_content.parse() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Warning: Failed to parse Cargo.lock: {}", e);
                eprintln!("Falling back to cargo-tree method...");
                return self.analyze_direct_dependencies_fallback();
            }
        };

        let package_array = match lock_doc.get("package") {
            Some(toml_edit::Item::ArrayOfTables(arr)) => arr,
            _ => {
                eprintln!("Warning: No packages found in Cargo.lock");
                return Ok(());
            }
        };

        // Phase 1: Index every [[package]] by name.
        // If multiple versions exist for the same name, keep the first one
        // (Cargo.lock entries are sorted; the workspace/local entry comes first).
        struct LockPkg {
            name: String,
            version: String,
            source: String,
            deps: Vec<String>,
        }

        let mut all_pkgs: Vec<LockPkg> = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        for table in package_array {
            let name = match table.get("name").and_then(|v| v.as_str()) {
                Some(n) => n.to_string(),
                None => continue,
            };

            let version = table
                .get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();

            let source = table
                .get("source")
                .and_then(|v| v.as_str())
                .unwrap_or("workspace")
                .to_string();

            // Parse dependencies — they may be plain strings or inline tables
            let mut deps: Vec<String> = Vec::new();
            if let Some(dep_item) = table.get("dependencies") {
                if let Some(arr) = dep_item.as_array() {
                    for val in arr {
                        let dep_name = if let Some(s) = val.as_str() {
                            // Plain string: "serde" or "serde (>=1.0, <2.0)"
                            // Take only the part before any '(' for version constraints
                            s.split('(').next().unwrap_or(s).trim().to_string()
                        } else if let Some(inline) = val.as_inline_table() {
                            // Inline table: { name = "serde", package = "serde-rename" }
                            // Use 'package' key if present (for renames), otherwise use 'name'
                            inline
                                .get("package")
                                .and_then(|v| v.as_str())
                                .or_else(|| inline.get("name").and_then(|v| v.as_str()))
                                .map(|s| s.to_string())
                                .unwrap_or_default()
                        } else {
                            continue;
                        };
                        if !dep_name.is_empty() {
                            deps.push(dep_name);
                        }
                    }
                }
            }

            // Track the first occurrence of each name (skip name collisions)
            // Keep workspace packages first
            if seen_names.contains(&name) {
                // If the existing entry is a workspace pkg, skip; otherwise replace
                continue;
            }
            seen_names.insert(name.clone());
            all_pkgs.push(LockPkg { name, version, source, deps });
        }

        println!("  Found {} unique packages in Cargo.lock", all_pkgs.len());

        // Phase 2: Add nodes for all packages
        for pkg in &all_pkgs {
            let is_ws = pkg.source == "workspace" || pkg.source.is_empty();
            let source_str = if is_ws {
                "workspace".to_string()
            } else if pkg.source.starts_with("git+") {
                pkg.source[4..].to_string()
            } else if pkg.source.starts_with("registry+") {
                "crates.io".to_string()
            } else {
                pkg.source.clone()
            };

            // Node ID uses lock: prefix to avoid collision with workspace: from discovery
            let node_id = format!("lock:{}", pkg.name);

            if !self.node_map.contains_key(&node_id) {
                let idx = self.graph.add_node(DependencyNode {
                    id: node_id.clone(),
                    crate_name: pkg.name.clone(),
                    version: pkg.version.clone(),
                    source: source_str,
                    is_workspace_member: is_ws,
                    is_direct_dependency: false,
                    features: HashSet::new(),
                    categories: vec!["dependency".to_string()],
                    properties: HashMap::new(),
                });
                self.node_map.insert(node_id, idx);
            }
        }

        // Phase 3: Add edges from each package's dependency list
        for pkg in &all_pkgs {
            // Map workspace: names and external names to their lock: equivalents
            let parent_id = format!("lock:{}", pkg.name);
            let parent_idx = match self.node_map.get(&parent_id) {
                Some(idx) => *idx,
                None => continue,
            };

            for dep_name in &pkg.deps {
                let dep_id = format!("lock:{}", dep_name);

                // Ensure dep node exists (it might be a workspace member not in lockfile)
                if !self.node_map.contains_key(&dep_id) {
                    let idx = self.graph.add_node(DependencyNode {
                        id: dep_id.clone(),
                        crate_name: dep_name.clone(),
                        version: "0.0.0".to_string(),
                        source: "unknown".to_string(),
                        is_workspace_member: false,
                        is_direct_dependency: false,
                        features: HashSet::new(),
                        categories: vec!["dependency".to_string()],
                        properties: HashMap::new(),
                    });
                    self.node_map.insert(dep_id.clone(), idx);
                }

                let dep_idx = self.node_map[&dep_id];

                // Avoid duplicating edges
                let has_edge = self.graph.edges(parent_idx).any(|e| e.target() == dep_idx);
                if !has_edge {
                    self.graph.add_edge(parent_idx, dep_idx, DependencyEdge {
                        from: parent_id.clone(),
                        to: dep_id,
                        edge_type: DependencyEdgeType::Direct,
                        required_features: HashSet::new(),
                        optional_features: HashSet::new(),
                        is_dev_dependency: false,
                        is_build_dependency: false,
                        properties: HashMap::new(),
                    });
                }
            }
        }

        println!("  Total nodes: {}", self.graph.node_count());
        println!("  Total edges: {}", self.graph.edge_count());
        Ok(())
    }

    /// Fallback method using cargo-tree (original implementation)
    fn analyze_direct_dependencies_fallback(&mut self) -> Result<()> {
        println!("Analyzing direct dependencies using cargo-tree fallback...");
        
        // Collect workspace member node IDs first to avoid borrow checker issues
        let workspace_nodes: Vec<String> = self.node_map.iter()
            .filter(|(node_id, _)| node_id.starts_with("workspace:"))
            .map(|(node_id, _)| node_id.clone())
            .collect();
        
        // Run cargo tree for each workspace member
        for node_id in workspace_nodes {
            let crate_name = node_id.trim_start_matches("workspace:");
            
            // Try different path patterns for workspace members
            let mut cargo_toml_path = None;
            
            // Pattern 1: Direct member (e.g., solana-cli/Cargo.toml)
            let direct_path = self.workspace_path.join(crate_name).join("Cargo.toml");
            if direct_path.exists() {
                cargo_toml_path = Some(direct_path);
            }
            
            // Pattern 2: In src/ directory (e.g., src/solana-cli/Cargo.toml)
            if cargo_toml_path.is_none() {
                let src_path = self.workspace_path.join("src").join(crate_name).join("Cargo.toml");
                if src_path.exists() {
                    cargo_toml_path = Some(src_path);
                }
            }
            
            // Pattern 3: Check if crate_name itself is a Cargo.toml
            if cargo_toml_path.is_none() && crate_name.ends_with("Cargo.toml") {
                let toml_path = self.workspace_path.join(crate_name);
                if toml_path.exists() {
                    cargo_toml_path = Some(toml_path);
                }
            }
            
            if let Some(cargo_toml) = cargo_toml_path {
                // Use cargo API to get dependencies directly
                match self.get_dependencies_using_cargo_api(&cargo_toml, crate_name) {
                    Ok(_) => {},
                    Err(e) => {
                        eprintln!("Warning: Failed to get dependencies for {} using cargo API: {}", crate_name, e);
                    }
                }
                continue;
                

            } else {
                eprintln!("Warning: Could not find Cargo.toml for workspace member: {}", crate_name);
            }
        }
        
        Ok(())
    }
    
    /// Get dependencies using cargo API directly
    fn get_dependencies_using_cargo_api(&mut self, manifest_path: &Path, crate_name: &str) -> Result<()> {
        // Use cargo-metadata as the primary method
        let metadata = match cargo_metadata::MetadataCommand::new()
            .manifest_path(manifest_path)
            .features(cargo_metadata::CargoOpt::AllFeatures)
            // .no_deps() would exclude dependencies, so we don't call it to include all dependencies
            .exec() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Warning: Failed to get metadata for {}: {}", crate_name, e);
                return Ok(());
            }
        };

        // Find our package in the metadata
        if let Some(our_package) = metadata.packages.iter().find(|p| p.name == crate_name) {
            // Add dependencies from this package
            for dep in &our_package.dependencies {
                let edge_type = match dep.kind {
                    cargo_metadata::DependencyKind::Normal => DependencyEdgeType::Direct,
                    cargo_metadata::DependencyKind::Development => DependencyEdgeType::Dev,
                    cargo_metadata::DependencyKind::Build => DependencyEdgeType::Build,
                    _ => DependencyEdgeType::Direct, // Default to Direct for unknown types
                };

                let from_node = format!("workspace:{}", crate_name);
                let to_node = format!("crate:{}", dep.name);

                let edge = DependencyEdge {
                    from: from_node.clone(),
                    to: to_node.clone(),
                    edge_type,
                    required_features: dep.features.iter().cloned().collect(),
                    optional_features: HashSet::new(),
                    is_dev_dependency: dep.kind == cargo_metadata::DependencyKind::Development,
                    is_build_dependency: dep.kind == cargo_metadata::DependencyKind::Build,
                    properties: HashMap::new(),
                };
                let from_idx = self.node_map[&from_node];
                let to_idx = self.node_map[&to_node];
                self.graph.add_edge(from_idx, to_idx, edge);

                eprintln!("Added dependency: {} -> {}", from_node, to_node);
            }
        }

        Ok(())
    }

    /// Get workspace metadata using cargo-metadata
    fn get_workspace_metadata(&self) -> Result<cargo_metadata::Metadata> {
        MetadataCommand::new()
            .manifest_path(&self.workspace_path.join("Cargo.toml"))
            .features(cargo_metadata::CargoOpt::AllFeatures)
            // .no_deps() would exclude dependencies, so we don't call it to include all dependencies
            .exec()
            .context("Failed to execute cargo metadata command")
    }
    
    /// Add dependencies from a package to the graph
    fn add_package_dependencies(&mut self, package: &Package, parent_node_id: &str) -> Result<()> {
        // Add direct dependencies
        for dep in &package.dependencies {
            // Skip build/dev dependencies if not included
            if !self.include_dev_dependencies && matches!(dep.kind, DependencyKind::Development) {
                continue;
            }
            if !self.include_build_dependencies && matches!(dep.kind, DependencyKind::Build) {
                continue;
            }
            
            let dep_node_id = format!("crate:{}", dep.name);
            
            // Get or create the dependency node
            let dep_idx = *self.node_map.entry(dep_node_id.clone()).or_insert_with(|| {
                // Check if this dependency is actually a workspace member
                let (source, is_workspace_member) = if self.workspace_members.contains(&dep.name) {
                    ("workspace".to_string(), true)
                } else {
                    // Use the actual source from the dependency if available
                    let source = if let Some(source) = &dep.source {
                        // Clean up git+ prefix and other formatting
                        let mut source_str = source.to_string();
                        if source_str.starts_with("git+") {
                            source_str = source_str[4..].to_string();
                        }
                        if source_str.starts_with("registry+") {
                            source_str = "crates.io".to_string();
                        }
                        source_str
                    } else {
                        "crates.io".to_string()
                    };
                    (source, false)
                };
                
                let idx = self.graph.add_node(DependencyNode {
                    id: dep_node_id.clone(),
                    crate_name: dep.name.clone(),
                    version: dep.req.to_string(),
                    source: source,
                    is_workspace_member: is_workspace_member,
                    is_direct_dependency: false,
                    features: HashSet::new(),
                    categories: vec!["dependency".to_string()],
                    properties: HashMap::new(),
                });
                idx
            });
            
            // Get the parent node index
            let parent_idx = self.node_map[parent_node_id];
            
            // Determine edge type
            let edge_type = match dep.kind {
                DependencyKind::Normal => DependencyEdgeType::Direct,
                DependencyKind::Development => DependencyEdgeType::Dev,
                DependencyKind::Build => DependencyEdgeType::Build,
                _ => DependencyEdgeType::Direct,
            };
            
            // Add edge from parent to dependency
            self.graph.add_edge(parent_idx, dep_idx, DependencyEdge {
                from: parent_node_id.to_string(),
                to: dep_node_id,
                edge_type,
                required_features: HashSet::new(),
                optional_features: HashSet::new(),
                is_dev_dependency: matches!(dep.kind, DependencyKind::Development),
                is_build_dependency: matches!(dep.kind, DependencyKind::Build),
                properties: HashMap::new(),
            });
        }
        
        Ok(())
    }
    

    
    /// Resolve transitive dependencies

    

    
    /// Expand feature dependencies
    fn expand_feature_dependencies(&mut self) -> Result<()> {
        println!("Expanding feature dependencies...");
        
        // Collect nodes that need feature expansion first to avoid borrowing issues
        let nodes_to_expand: Vec<String> = self.node_map
            .iter()
            .filter_map(|(node_id, _)| {
                if !node_id.starts_with("workspace:") {
                    if let Some(idx) = self.node_map.get(node_id) {
                        if let Some(node) = self.graph.node_weight(*idx) {
                            if !node.features.is_empty() {
                                return Some(node_id.clone());
                            }
                        }
                    }
                }
                None
            })
            .collect();
        
        // Now expand features for each node
        for node_id in nodes_to_expand {
            if let Some(idx) = self.node_map.get(&node_id) {
                if let Some(node_features) = self.graph.node_weight(*idx).map(|n| n.features.clone()) {
                    self.expand_features_for_crate(&node_id, &node_features)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Expand features for a specific crate
    fn expand_features_for_crate(&mut self, crate_id: &str, features: &HashSet<String>) -> Result<()> {
        let crate_name = crate_id.trim_start_matches("crate:");
        
        // For each feature, check what it enables
        for feature in features {
            // In real implementation, this would use cargo tree with specific features
            // For now, we'll simulate feature expansion
            
            // Simulate feature-enabled dependencies
            let feature_deps = vec![
                (format!("{}-feature", crate_name), "1.0.0", DependencyEdgeType::FeatureGated),
                (format!("{}-{}", crate_name, feature), "1.0.0", DependencyEdgeType::FeatureGated),
            ];
            
            for (dep_name, dep_version, edge_type) in feature_deps {
                let dep_node_id = format!("crate:{}", dep_name);
                
                // Add dependency node if not exists
                if !self.node_map.contains_key(&dep_node_id) {
                    let node = DependencyNode {
                        id: dep_node_id.clone(),
                        crate_name: dep_name,
                        version: dep_version.to_string(),
                        source: "crates.io".to_string(),
                        is_workspace_member: false,
                        is_direct_dependency: false,
                        features: HashSet::new(),
                        categories: vec!["feature-dependency".to_string()],
                        properties: [
                            ("enabled_by".to_string(), format!("{}:{}", crate_name, feature)),
                        ].iter().cloned().collect(),
                    };
                    let idx = self.graph.add_node(node);
                    self.node_map.insert(dep_node_id.clone(), idx);
                }
                
                // Add feature-gated edge
                let from_idx = self.node_map[crate_id];
                let to_idx = self.node_map[&dep_node_id];
                
                let edge = DependencyEdge {
                    from: crate_id.to_string(),
                    to: dep_node_id,
                    edge_type,
                    required_features: {
                        let mut set = HashSet::new();
                        set.insert(feature.clone());
                        set
                    },
                    optional_features: HashSet::new(),
                    is_dev_dependency: false,
                    is_build_dependency: false,
                    properties: [
                        ("feature".to_string(), feature.clone()),
                    ].iter().cloned().collect(),
                };
                
                self.graph.add_edge(from_idx, to_idx, edge);
            }
        }
        
        Ok(())
    }
    
    /// Calculate graph metrics
    fn calculate_metrics(&self) -> Result<GraphMetrics> {
        let node_count = self.graph.node_count();
        let edge_count = self.graph.edge_count();
        
        let workspace_members = self.graph.node_weights()
            .filter(|n| n.is_workspace_member)
            .count();
        
        let direct_dependencies = self.graph.edge_weights()
            .filter(|e| matches!(e.edge_type, DependencyEdgeType::Direct))
            .count();
        
        let transitive_dependencies = self.graph.edge_weights()
            .filter(|e| matches!(e.edge_type, DependencyEdgeType::Transitive))
            .count();
        
        let feature_gated_dependencies = self.graph.edge_weights()
            .filter(|e| matches!(e.edge_type, DependencyEdgeType::FeatureGated))
            .count();
        
        let dev_dependencies = self.graph.edge_weights()
            .filter(|e| e.is_dev_dependency)
            .count();
        
        let build_dependencies = self.graph.edge_weights()
            .filter(|e| e.is_build_dependency)
            .count();
        
        // Calculate diameter (simplified)
        let diameter = (node_count as f64).log2() as usize;
        
        // Calculate average degree
        let average_degree = (2 * edge_count) as f64 / node_count as f64;
        
        Ok(GraphMetrics {
            node_count,
            edge_count,
            workspace_members,
            direct_dependencies,
            transitive_dependencies,
            feature_gated_dependencies,
            dev_dependencies,
            build_dependencies,
            strongly_connected_components: 0, // Will be calculated separately
            diameter,
            average_degree,
        })
    }
    
    /// Find strongly connected components
    fn find_strongly_connected_components(&self) -> Result<Vec<Vec<String>>> {
        let scc = tarjan_scc(&self.graph);
        
        let components = scc.iter()
            .map(|component| {
                component.iter()
                    .filter_map(|&idx| {
                        self.graph.node_weight(idx)
                            .map(|node| node.id.clone())
                    })
                    .collect()
            })
            .filter(|component: &Vec<_>| component.len() > 1) // Only components with >1 node
            .collect();
        
        Ok(components)
    }
    
    /// Collect feature information
    fn collect_feature_information(&self) -> Result<Vec<FeatureSet>> {
        let mut feature_sets = Vec::new();
        
        for (node_id, _) in &self.node_map {
            if !node_id.starts_with("workspace:") {
                if let Some(idx) = self.node_map.get(node_id) {
                    if let Some(node) = self.graph.node_weight(*idx) {
                        if !node.features.is_empty() {
                            // Find feature dependencies
                            let mut feature_deps = HashMap::new();
                            
                            for edge in self.graph.edges(*idx) {
                                if let Some(edge_weight) = self.graph.edge_weight(edge.id()) {
                                    if !edge_weight.required_features.is_empty() {
                                        for feature in &edge_weight.required_features {
                                            let target_node = self.graph.node_weight(edge.target());
                                            if let Some(target) = target_node {
                                                feature_deps.entry(feature.clone())
                                                    .or_insert(HashSet::new())
                                                    .insert(target.crate_name.clone());
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Get default features (simplified)
                            let default_features = node.features.iter()
                                .filter(|f| f.starts_with("default-") || f == &"default")
                                .cloned()
                                .collect();
                            
                            feature_sets.push(FeatureSet {
                                crate_name: node.crate_name.clone(),
                                features: node.features.clone(),
                                default_features,
                                feature_dependencies: feature_deps,
                            });
                        }
                    }
                }
            }
        }
        
        Ok(feature_sets)
    }
    
    /// Generate visualization of the dependency graph
    pub fn generate_visualization(&self, output_path: &Path) -> Result<()> {
        println!("Generating dependency graph visualization...");
        
        let dot = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]);
        let dot_content = format!("{:?}", dot);
        
        fs::write(output_path, dot_content)
            .context("Failed to write visualization")?;
        
        println!("Visualization saved to {}", output_path.display());
        Ok(())
    }
    
    /// Export graph to JSON
    pub fn export_to_json(&mut self, output_path: &Path) -> Result<()> {
        let graph = self.build_global_graph()?;
        let json = serde_json::to_string_pretty(&graph)
            .context("Failed to serialize graph")?;
        
        fs::write(output_path, json)
            .context("Failed to write JSON")?;
        
        Ok(())
    }
    
    /// Analyze graph for Solana-specific patterns
    pub fn analyze_solana_patterns(&mut self) -> Result<SolanaAnalysis> {
        println!("Analyzing Solana-specific patterns...");
        
        let graph = self.build_global_graph()?;
        
        // Find critical path (simplified)
        let critical_path = self.find_critical_path()?;
        
        // Identify core components
        let core_components = self.identify_core_components()?;
        
        // Find circular dependencies
        let circular_deps = graph.strongly_connected_components;
        
        // Calculate centrality metrics
        let centrality = self.calculate_centrality()?;
        
        Ok(SolanaAnalysis {
            critical_path,
            core_components,
            circular_dependencies: circular_deps,
            centrality,
            metrics: graph.metrics,
        })
    }
    
    /// Find critical path (simplified)
    fn find_critical_path(&self) -> Result<Vec<String>> {
        // In real implementation, this would use proper path finding algorithms
        // For now, return workspace members as critical path
        let mut critical_path = Vec::new();
        
        for (node_id, _) in &self.node_map {
            if node_id.starts_with("workspace:") {
                critical_path.push(node_id.clone());
            }
        }
        
        Ok(critical_path)
    }
    
    /// Identify core components
    fn identify_core_components(&self) -> Result<Vec<CoreComponent>> {
        let mut core_components = Vec::new();
        
        // Find nodes with high degree (simplified)
        for (node_id, _) in &self.node_map {
            if let Some(idx) = self.node_map.get(node_id) {
                let degree = self.graph.edges(*idx).count();
                if degree > 5 { // Arbitrary threshold
                    if let Some(node) = self.graph.node_weight(*idx) {
                        core_components.push(CoreComponent {
                            crate_name: node.crate_name.clone(),
                            version: node.version.clone(),
                            degree,
                            is_workspace_member: node.is_workspace_member,
                        });
                    }
                }
            }
        }
        
        Ok(core_components)
    }
    
    /// Calculate centrality metrics (simplified)
    fn calculate_centrality(&self) -> Result<CentralityMetrics> {
        let node_count = self.graph.node_count();
        
        // Find node with highest degree
        let mut max_degree = 0;
        let mut max_betweenness = 0.0;
        let mut max_closeness = 0.0;
        
        for (node_id, _) in &self.node_map {
            if let Some(idx) = self.node_map.get(node_id) {
                let degree = self.graph.edges(*idx).count();
                if degree > max_degree {
                    max_degree = degree;
                }
                
                // Simplified betweenness and closeness
                let betweenness = degree as f64 / node_count as f64;
                if betweenness > max_betweenness {
                    max_betweenness = betweenness;
                }
                
                let closeness = 1.0 / (node_count as f64 - degree as f64 + 1.0);
                if closeness > max_closeness {
                    max_closeness = closeness;
                }
            }
        }
        
        Ok(CentralityMetrics {
            max_degree,
            max_betweenness,
            max_closeness,
            average_degree: self.calculate_metrics()?.average_degree,
        })
    }
    
    /// Analyze TOML structures in all Cargo.toml files
    /// Analyze TOML structures in the workspace
    fn analyze_toml_structures(&self) -> Result<Vec<TomlStructure>> {
        let mut toml_structures = Vec::new();
        
        // Use cargo metadata to discover all workspace Cargo.toml files
        // This is fast (no filesystem walk) and pure Rust (no find/shell)
        match self.get_workspace_metadata() {
            Ok(metadata) => {
                for package in &metadata.packages {
                    if package.source.is_none() {
                        let cargo_file = package.manifest_path.as_std_path();
                        if cargo_file.exists() {
                            if let Some(toml_structure) = self.analyze_cargo_toml(cargo_file) {
                                toml_structures.push(toml_structure);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: cargo metadata failed ({}), falling back to filesystem discovery", e);
                // Fallback: find Cargo.toml files but skip target/ and vendor/
                let cargo_files = self.find_all_cargo_files_fast(&self.workspace_path)?;
                for cargo_file in cargo_files {
                    if let Some(toml_structure) = self.analyze_cargo_toml(&cargo_file) {
                        toml_structures.push(toml_structure);
                    }
                }
            }
        }
        
        Ok(toml_structures)
    }
    /// Find all Cargo.toml files recursively (with target/ and vendor/ skipping)
    fn find_all_cargo_files_fast(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        let mut cargo_files = Vec::new();
        
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                // Skip target/, vendor/, .git/ and other large generated directories
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if matches!(dir_name, "target" | "vendor" | ".git" | "node_modules" | "build") {
                        continue;
                    }
                    let mut sub_files = self.find_all_cargo_files_fast(&path)?;
                    cargo_files.append(&mut sub_files);
                } else if path.file_name() == Some("Cargo.toml".as_ref()) {
                    cargo_files.push(path.clone());
                }
            }
        }
        
        Ok(cargo_files)
    }
    
    /// Analyze a single Cargo.toml file
    fn analyze_cargo_toml(&self, cargo_file: &Path) -> Option<TomlStructure> {
        let content = match fs::read_to_string(cargo_file) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("Warning: Failed to read {}: {}", cargo_file.display(), e);
                return None;
            }
        };
        
        let doc = match content.parse::<DocumentMut>() {
            Ok(doc) => doc,
            Err(e) => {
                eprintln!("Warning: Failed to parse {}: {}", cargo_file.display(), e);
                return None;
            }
        };
        
        // Extract basic information with proper error handling
        let crate_name = doc.get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .unwrap_or("unknown")
            .to_string();
        
        let crate_version = doc.get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();
        
        // Analyze schema elements
        let schema_elements = match self.analyze_schema_elements(&doc) {
            Ok(elements) => elements,
            Err(e) => {
                eprintln!("Warning: Failed to analyze schema elements: {}", e);
                return None;
            }
        };
        
        // Identify workload patterns
        let workload_patterns = match self.identify_workload_patterns(&doc, &schema_elements) {
            Ok(patterns) => patterns,
            Err(e) => {
                eprintln!("Warning: Failed to identify workload patterns: {}", e);
                return None;
            }
        };
        
        // Check for workspace
        let is_workspace = doc.get("workspace").is_some();
        
        // Check for binaries, tests, benchmarks
        let has_binaries = doc.get("bin").is_some();
        let has_tests = self.has_test_patterns(&doc);
        let has_benchmarks = doc.get("bench").is_some();
        
        Some(TomlStructure {
            crate_name,
            crate_version,
            file_path: cargo_file.to_string_lossy().to_string(),
            schema_elements,
            workload_patterns,
            is_workspace,
            has_binaries,
            has_tests,
            has_benchmarks,
        })
    }
    
    /// Analyze schema elements in TOML document
    fn analyze_schema_elements(&self, doc: &DocumentMut) -> Result<Vec<SchemaElement>> {
        let mut elements = Vec::new();
        
        // Analyze package section
        if let Some(package) = doc.get("package") {
            let package_element = self.analyze_table_element("package", package, SchemaElementType::Package)?;
            elements.push(package_element);
        }
        
        // Analyze dependencies
        for dep_type in ["dependencies", "dev-dependencies", "build-dependencies"] {
            if let Some(deps) = doc.get(dep_type) {
                if let toml_edit::Item::Table(deps_table) = deps {
                    for (dep_name, dep_value) in deps_table.iter() {
                        let element_type = match dep_type {
                            "dependencies" => SchemaElementType::Dependency,
                            "dev-dependencies" => SchemaElementType::DevDependency,
                            "build-dependencies" => SchemaElementType::BuildDependency,
                            _ => SchemaElementType::Dependency,
                        };
                        
                        let path = format!("{}.{}", dep_type, dep_name);
                        let element = SchemaElement {
                            element_type,
                            path: path.clone(),
                            value_type: "dependency".to_string(),
                            is_array: false,
                            is_table: false,
                            is_inline_table: dep_value.is_table_like(),
                            children: Vec::new(),
                            properties: HashMap::new(),
                        };
                        elements.push(element);
                    }
                }
            }
        }
        
        // Analyze features
        if let Some(features) = doc.get("features") {
            if let toml_edit::Item::Table(features_table) = features {
                for (feature_name, feature_value) in features_table.iter() {
                    let element = SchemaElement {
                        element_type: SchemaElementType::Feature,
                        path: format!("features.{}", feature_name),
                        value_type: "feature".to_string(),
                        is_array: feature_value.is_array(),
                        is_table: feature_value.is_table(),
                        is_inline_table: feature_value.is_inline_table(),
                        children: Vec::new(),
                        properties: HashMap::new(),
                    };
                    elements.push(element);
                }
            }
        }
        
        // Analyze binaries
        if let Some(bins) = doc.get("bin") {
            if let toml_edit::Item::Value(toml_edit::Value::Array(bins_array)) = bins {
                for (i, bin_item) in bins_array.iter().enumerate() {
                    if let toml_edit::Value::InlineTable(bin_table) = bin_item {
                        if let Some(name) = bin_table.get("name").and_then(|v| v.as_str()) {
                            let element = SchemaElement {
                                element_type: SchemaElementType::Binary,
                                path: format!("bin[{}]", i),
                                value_type: "binary".to_string(),
                                is_array: false,
                                is_table: false,
                                is_inline_table: true,
                                children: vec![format!("bin[{}].name", i), format!("bin[{}].path", i)],
                                properties: HashMap::from([
                                    ("name".to_string(), name.to_string()),
                                    ("path".to_string(), bin_table.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                                ]),
                            };
                            elements.push(element);
                        }
                    }
                }
            }
        }
        
        Ok(elements)
    }
    
    /// Analyze a table element
    fn analyze_table_element(&self, name: &str, item: &toml_edit::Item, element_type: SchemaElementType) -> Result<SchemaElement> {
        let mut children = Vec::new();
        let mut properties = HashMap::<String, String>::new();
        
        if let toml_edit::Item::Table(table) = item {
            for (key, value) in table.iter() {
                children.push(format!("{}.{}", name, key));
                if let Some(str_val) = value.as_str() {
                    properties.insert(key.to_string(), str_val.to_string());
                }
            }
        }
        
        Ok(SchemaElement {
            element_type,
            path: name.to_string(),
            value_type: "table".to_string(),
            is_array: false,
            is_table: true,
            is_inline_table: false,
            children,
            properties,
        })
    }
    
    /// Identify workload patterns in TOML document
    fn identify_workload_patterns(&self, doc: &DocumentMut, elements: &[SchemaElement]) -> Result<Vec<WorkloadPattern>> {
        let mut patterns = Vec::new();
        
        // Identify binary targets
        if let Some(bins) = doc.get("bin") {
            if let toml_edit::Item::Value(toml_edit::Value::Array(bins_array)) = bins {
                for (i, bin_item) in bins_array.iter().enumerate() {
                    if let toml_edit::Value::InlineTable(bin_table) = bin_item {
                        if let Some(name) = bin_table.get("name").and_then(|v| v.as_str()) {
                            patterns.push(WorkloadPattern {
                                pattern_type: WorkloadPatternType::BinaryTarget,
                                name: name.to_string(),
                                location: format!("bin[{}]", i),
                                related_elements: vec![format!("bin[{}]", i)],
                                properties: HashMap::from([
                                    ("path".to_string(), bin_table.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string()),
                                ]),
                            });
                        }
                    }
                }
            }
        }
        
        // Identify test suites
        if self.has_test_patterns(doc) {
            patterns.push(WorkloadPattern {
                pattern_type: WorkloadPatternType::TestSuite,
                name: "tests".to_string(),
                location: "tests/".to_string(),
                related_elements: elements.iter()
                    .filter(|e| e.element_type == SchemaElementType::Feature)
                    .map(|e| e.path.clone())
                    .collect(),
                properties: HashMap::new(),
            });
        }
        
        // Identify benchmarks
        if let Some(_bench) = doc.get("bench") {
            patterns.push(WorkloadPattern {
                pattern_type: WorkloadPatternType::BenchmarkSuite,
                name: "benchmarks".to_string(),
                location: "benchmarks/".to_string(),
                related_elements: vec!["bench".to_string()],
                properties: HashMap::new(),
            });
        }
        
        // Identify feature flags
        if let Some(features) = doc.get("features") {
            if let toml_edit::Item::Table(features_table) = features {
                for (feature_name, _) in features_table.iter() {
                    patterns.push(WorkloadPattern {
                        pattern_type: WorkloadPatternType::FeatureFlag,
                        name: feature_name.to_string(),
                        location: format!("features.{}", feature_name),
                        related_elements: vec![format!("features.{}", feature_name)],
                        properties: HashMap::new(),
                    });
                }
            }
        }
        
        // Identify workspace members
        if let Some(_workspace) = doc.get("workspace") {
            patterns.push(WorkloadPattern {
                pattern_type: WorkloadPatternType::WorkspaceMember,
                name: "workspace".to_string(),
                location: "workspace".to_string(),
                related_elements: vec!["workspace".to_string()],
                properties: HashMap::new(),
            });
        }
        
        Ok(patterns)
    }
    
    /// Check if TOML has test patterns
    fn has_test_patterns(&self, doc: &DocumentMut) -> bool {
        // Check for test features
        if let Some(features) = doc.get("features") {
            if let toml_edit::Item::Table(features_table) = features {
                for (feature_name, _) in features_table.iter() {
                    if feature_name.starts_with("test-") || feature_name.ends_with("-test") {
                        return true;
                    }
                }
            }
        }
        
        // Check for [[test]] sections (would be in array format)
        doc.get("test").is_some()
    }
    
    /// Get workspace members in dependency order (dependencies first, dependents last).
    ///
    /// Returns crates in the order they should be processed: a crate's dependencies
    /// are always processed before the crate itself.
    ///
    /// Uses topological sort on the dependency graph to ensure correct ordering.
    ///
    /// # Errors
    /// Returns error if circular dependencies are detected among workspace members.
    pub fn get_publish_order(&self) -> Result<Vec<String>> {
        // Build a subgraph with only workspace members
        // This is critical: external dependencies can have cycles,
        // but workspace members should never have cycles (Cargo enforces this)
        let mut subgraph = Graph::<DependencyNode, DependencyEdgeType>::new();
        let mut node_map: HashMap<String, NodeIndex> = HashMap::new();
        
        // Add only workspace member nodes
        for (node_id, &idx) in &self.node_map {
            if let Some(node) = self.graph.node_weight(idx) {
                if node.is_workspace_member {
                    let new_idx = subgraph.add_node(node.clone());
                    node_map.insert(node_id.clone(), new_idx);
                }
            }
        }
        
        // Add edges between workspace members only
        // Skip dev-dependencies as they don't affect processing order
        for (from_id, &from_idx) in &self.node_map {
            if let Some(from_node) = self.graph.node_weight(from_idx) {
                if from_node.is_workspace_member {
                    if let Some(&from_sub_idx) = node_map.get(from_id) {
                        // Check all outgoing edges from this workspace member
                        for edge in self.graph.edges_directed(from_idx, Direction::Outgoing) {
                            let to_idx = edge.target();
                            if let Some(to_node) = self.graph.node_weight(to_idx) {
                                if to_node.is_workspace_member {
                                    if let Some(&to_sub_idx) = node_map.get(&self.graph[to_idx].id) {
                                        // Skip dev-dependencies - they don't affect processing order
                                        if !edge.weight().is_dev_dependency {
                                            subgraph.add_edge(from_sub_idx, to_sub_idx, edge.weight().edge_type.clone());
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Run toposort on the workspace-only subgraph
        let sorted = toposort(&subgraph, None).map_err(|_cycle| {
            anyhow::anyhow!(
                "Circular dependency detected involving workspace crate. This should not happen in a valid Cargo workspace."
            )
        })?;
        
        // Collect node IDs in dependency order
        let result: Vec<String> = sorted.into_iter().filter_map(|idx| {
            subgraph.node_weight(idx).map(|n| n.id.clone())
        }).collect();
        
        Ok(result)
    }
    
    /// Get topological order for all nodes (including external dependencies).
    ///
    /// This includes both workspace members and external crates in dependency order.
    /// Useful for layer 1 processing (external dependencies).
    ///
    /// # Note
    /// External dependencies may have cycles, so this method may fail.
    /// For workspace-only processing, use get_publish_order() instead.
    pub fn get_topological_order(&self) -> Result<Vec<String>> {
        // Run toposort on the full graph
        let sorted = toposort(&self.graph, None).map_err(|cycle| {
            // Try to identify the cycle by finding nodes involved in the cycle
            let mut cycle_nodes = Vec::new();
            
            // The cycle contains the node that's part of the cycle
            if let Some(node) = self.graph.node_weight(cycle.node_id()) {
                cycle_nodes.push(node.id.clone());
                
                // Look for incoming/outgoing edges that might form the cycle
                for edge in self.graph.edges_directed(cycle.node_id(), petgraph::Direction::Incoming) {
                    if let Some(source_node) = self.graph.node_weight(edge.source()) {
                        cycle_nodes.push(source_node.id.clone());
                    }
                }
                for edge in self.graph.edges_directed(cycle.node_id(), petgraph::Direction::Outgoing) {
                    if let Some(target_node) = self.graph.node_weight(edge.target()) {
                        cycle_nodes.push(target_node.id.clone());
                    }
                }
            }
            
            if !cycle_nodes.is_empty() {
                anyhow::anyhow!("Circular dependency detected in dependency graph involving: {:?}", cycle_nodes)
            } else {
                anyhow::anyhow!("Circular dependency detected in dependency graph")
            }
        })?;
        
        // Collect node IDs in topological order
        let result: Vec<String> = sorted.into_iter().filter_map(|idx| {
            self.graph.node_weight(idx).map(|n| n.id.clone())
        }).collect();
        
        Ok(result)
    }
    
    /// Get topological order for external dependencies only
    pub fn get_external_dependency_order(&self) -> Result<Vec<String>> {
        // Build subgraph with only external dependencies
        let mut ext_subgraph = Graph::<DependencyNode, DependencyEdgeType>::new();
        let mut ext_node_map: HashMap<String, NodeIndex> = HashMap::new();
        
        // Add external dependency nodes
        for (node_id, &idx) in &self.node_map {
            if let Some(node) = self.graph.node_weight(idx) {
                if !node.is_workspace_member {
                    let new_idx = ext_subgraph.add_node(node.clone());
                    ext_node_map.insert(node_id.clone(), new_idx);
                }
            }
        }
        
        // Add edges between external dependencies
        for (from_id, &from_idx) in &self.node_map {
            if let Some(from_node) = self.graph.node_weight(from_idx) {
                if !from_node.is_workspace_member {
                    if let Some(&from_ext_idx) = ext_node_map.get(from_id) {
                        for edge in self.graph.edges_directed(from_idx, Direction::Outgoing) {
                            let to_idx = edge.target();
                            if let Some(to_node) = self.graph.node_weight(to_idx) {
                                if !to_node.is_workspace_member {
                                    if let Some(&to_ext_idx) = ext_node_map.get(&self.graph[to_idx].id) {
                                        ext_subgraph.add_edge(from_ext_idx, to_ext_idx, edge.weight().edge_type.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
        // Run toposort on external dependency subgraph
        let sorted = toposort(&ext_subgraph, None).map_err(|cycle| {
            // Try to identify the cycle by finding nodes involved in the cycle
            let mut cycle_nodes = Vec::new();
            
            // The cycle contains the node that's part of the cycle
            if let Some(node) = ext_subgraph.node_weight(cycle.node_id()) {
                cycle_nodes.push(node.id.clone());
                
                // Look for incoming/outgoing edges that might form the cycle
                for edge in ext_subgraph.edges_directed(cycle.node_id(), petgraph::Direction::Incoming) {
                    if let Some(source_node) = ext_subgraph.node_weight(edge.source()) {
                        cycle_nodes.push(source_node.id.clone());
                    }
                }
                for edge in ext_subgraph.edges_directed(cycle.node_id(), petgraph::Direction::Outgoing) {
                    if let Some(target_node) = ext_subgraph.node_weight(edge.target()) {
                        cycle_nodes.push(target_node.id.clone());
                    }
                }
            }
            
            if !cycle_nodes.is_empty() {
                anyhow::anyhow!("Circular dependency detected among external dependencies involving: {:?}", cycle_nodes)
            } else {
                anyhow::anyhow!("Circular dependency detected among external dependencies")
            }
        })?;
        
        Ok(sorted.into_iter().filter_map(|idx| {
            ext_subgraph.node_weight(idx).map(|n| n.id.clone())
        }).collect())
    }
    
    /// Get workspace members only (for layer 2 processing)
    pub fn get_workspace_members(&self) -> Vec<String> {
        self.graph.node_weights()
            .filter(|n| n.is_workspace_member)
            .map(|n| n.id.clone())
            .collect()
    }
    
    /// Get external dependencies only (for layer 1 processing)
    pub fn get_external_dependencies(&self) -> Vec<String> {
        self.graph.node_weights()
            .filter(|n| !n.is_workspace_member)
            .map(|n| n.id.clone())
            .collect()
    }
}



/// Solana-specific analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolanaAnalysis {
    pub critical_path: Vec<String>,
    pub core_components: Vec<CoreComponent>,
    pub circular_dependencies: Vec<Vec<String>>,
    pub centrality: CentralityMetrics,
    pub metrics: GraphMetrics,
}

/// Core component in Solana
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreComponent {
    pub crate_name: String,
    pub version: String,
    pub degree: usize,
    pub is_workspace_member: bool,
}

/// Centrality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CentralityMetrics {
    pub max_degree: usize,
    pub max_betweenness: f64,
    pub max_closeness: f64,
    pub average_degree: f64,
}

/// Global dependency graph analyzer
pub struct GlobalDependencyGraphAnalyzer {
    builder: GlobalDependencyGraphBuilder,
}

impl GlobalDependencyGraphAnalyzer {
    /// Create a new analyzer
    pub fn new(workspace_path: PathBuf) -> Self {
        Self {
            builder: GlobalDependencyGraphBuilder::new(workspace_path),
        }
    }
    
    /// Build and analyze the global graph
    pub fn build_and_analyze(&mut self) -> Result<(GlobalDependencyGraph, SolanaAnalysis)> {
        let graph = self.builder.build_global_graph()?;
        let analysis = self.builder.analyze_solana_patterns()?;
        
        Ok((graph, analysis))
    }
    
    /// Generate comprehensive report
    pub fn generate_report(&self, graph: &GlobalDependencyGraph, analysis: &SolanaAnalysis, output_path: &Path) -> Result<()> {
        let mut report = String::new();
        
        report.push_str("# Global Dependency Graph Analysis Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Graph statistics
        report.push_str("## Graph Statistics\n\n");
        report.push_str(&format!("- **Total Nodes**: {}\n", graph.metrics.node_count));
        report.push_str(&format!("- **Total Edges**: {}\n", graph.metrics.edge_count));
        report.push_str(&format!("- **Workspace Members**: {}\n", graph.metrics.workspace_members));
        report.push_str(&format!("- **Direct Dependencies**: {}\n", graph.metrics.direct_dependencies));
        report.push_str(&format!("- **Transitive Dependencies**: {}\n", graph.metrics.transitive_dependencies));
        report.push_str(&format!("- **Feature-Gated Dependencies**: {}\n", graph.metrics.feature_gated_dependencies));
        report.push_str(&format!("- **Dev Dependencies**: {}\n", graph.metrics.dev_dependencies));
        report.push_str(&format!("- **Build Dependencies**: {}\n", graph.metrics.build_dependencies));
        report.push_str(&format!("- **Strongly Connected Components**: {}\n", graph.metrics.strongly_connected_components));
        report.push_str(&format!("- **Graph Diameter**: {}\n", graph.metrics.diameter));
        report.push_str(&format!("- **Average Degree**: {:.2}\n\n", graph.metrics.average_degree));
        
        // Critical path
        report.push_str("## Critical Path\n\n");
        report.push_str("The critical path represents the most important dependencies in the Solana universe:\n\n");
        for (i, node_id) in analysis.critical_path.iter().enumerate() {
            report.push_str(&format!("{}. {}\n", i + 1, node_id));
        }
        
        // Core components
        report.push_str("\n## Core Components\n\n");
        report.push_str("Components with high connectivity (degree > 5):\n\n");
        report.push_str("| Crate | Version | Degree | Type |\n");
        report.push_str("|-------|---------|--------|------|\n");
        for component in &analysis.core_components {
            let crate_type = if component.is_workspace_member { "Workspace" } else { "External" };
            report.push_str(&format!("| {} | {} | {} | {} |\n",
                component.crate_name, component.version, component.degree, crate_type));
        }
        
        // Circular dependencies
        if !analysis.circular_dependencies.is_empty() {
            report.push_str("\n## Circular Dependencies\n\n");
            report.push_str(&format!("Found {} strongly connected components:\n\n", analysis.circular_dependencies.len()));
            
            for (i, component) in analysis.circular_dependencies.iter().enumerate() {
                report.push_str(&format!("### Component {}\n\n", i + 1));
                report.push_str("Nodes:\n");
                for node in component {
                    report.push_str(&format!("- {}\n", node));
                }
                report.push_str("\n");
            }
        } else {
            report.push_str("\n## Circular Dependencies\n\nNo circular dependencies found.\n");
        }
        
        // Centrality metrics
        report.push_str("\n## Centrality Metrics\n\n");
        report.push_str(&format!("- **Max Degree**: {}\n", analysis.centrality.max_degree));
        report.push_str(&format!("- **Max Betweenness**: {:.4}\n", analysis.centrality.max_betweenness));
        report.push_str(&format!("- **Max Closeness**: {:.4}\n", analysis.centrality.max_closeness));
        report.push_str(&format!("- **Average Degree**: {:.2}\n\n", analysis.centrality.average_degree));
        
        // Feature analysis
        report.push_str("## Feature Analysis\n\n");
        report.push_str(&format!("- **Total Feature Sets**: {}\n", graph.features.len()));
        
        for feature_set in &graph.features {
            report.push_str(&format!("\n### {} v{}\n\n", feature_set.crate_name, "1.0.0"));
            report.push_str(&format!("- **Features**: {}\n", feature_set.features.len()));
            report.push_str(&format!("- **Default Features**: {}\n", feature_set.default_features.len()));
            report.push_str("- **Feature Dependencies**:\n");
            
            for (feature, deps) in &feature_set.feature_dependencies {
                report.push_str(&format!("  - **{}**: {}\n", feature, deps.iter().cloned().collect::<Vec<_>>().join(", ")));
            }
        }
        
        // Strongly connected components
        report.push_str("\n## Strongly Connected Components\n\n");
        report.push_str(&format!("Found {} SCCs (excluding single nodes):\n\n", graph.strongly_connected_components.len()));
        
        for (i, component) in graph.strongly_connected_components.iter().enumerate() {
            report.push_str(&format!("### SCC {}\n\n", i + 1));
            report.push_str("Nodes:\n");
            for node in component {
                report.push_str(&format!("- {}\n", node));
            }
        }
        
        // Recommendations
        report.push_str("\n## Recommendations\n\n");
        
        if !analysis.circular_dependencies.is_empty() {
            report.push_str("⚠️ **Circular Dependencies Detected**: Consider refactoring to break dependency cycles.\n\n");
        }
        
        if analysis.centrality.max_degree > 10 {
            report.push_str("📊 **High Connectivity**: Some components have very high degree. Consider if this is intentional architecture.\n\n");
        }
        
        if graph.metrics.feature_gated_dependencies > graph.metrics.direct_dependencies / 2 {
            report.push_str("🎛️ **Feature-Heavy**: Many dependencies are feature-gated. Ensure proper feature management.\n\n");
        }
        
        report.push_str("✅ **Overall**: The dependency graph appears well-structured with clear critical path and core components.\n");
        
        fs::write(output_path, report)
            .context("Failed to write analysis report")?;
        
        Ok(())
    }
    
    /// Generate TOML structure visualization
    pub fn generate_toml_structure_visualization(&self, graph: &GlobalDependencyGraph, output_path: &Path) -> Result<()> {
        let mut visualization = String::new();
        
        visualization.push_str("# TOML Structure Analysis\n\n");
        visualization.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Summary statistics
        visualization.push_str("## Summary Statistics\n\n");
        visualization.push_str(&format!("- **Total Cargo.toml Files Analyzed**: {}\n", graph.toml_structures.len()));
        
        let workspace_count = graph.toml_structures.iter().filter(|s| s.is_workspace).count();
        let binary_count = graph.toml_structures.iter().filter(|s| s.has_binaries).count();
        let test_count = graph.toml_structures.iter().filter(|s| s.has_tests).count();
        let benchmark_count = graph.toml_structures.iter().filter(|s| s.has_benchmarks).count();
        
        visualization.push_str(&format!("- **Workspace Files**: {}\n", workspace_count));
        visualization.push_str(&format!("- **Files with Binaries**: {}\n", binary_count));
        visualization.push_str(&format!("- **Files with Tests**: {}\n", test_count));
        visualization.push_str(&format!("- **Files with Benchmarks**: {}\n\n", benchmark_count));
        
        // Detailed analysis per file
        visualization.push_str("## Detailed TOML Structure Analysis\n\n");
        
        for (i, structure) in graph.toml_structures.iter().enumerate() {
            visualization.push_str(&format!("### {} - {} v{}\n\n", i + 1, structure.crate_name, structure.crate_version));
            visualization.push_str(&format!("- **File Path**: {}\n", structure.file_path));
            visualization.push_str(&format!("- **Is Workspace**: {}\n", structure.is_workspace));
            visualization.push_str(&format!("- **Has Binaries**: {}\n", structure.has_binaries));
            visualization.push_str(&format!("- **Has Tests**: {}\n", structure.has_tests));
            visualization.push_str(&format!("- **Has Benchmarks**: {}\n\n", structure.has_benchmarks));
            
            // Schema elements
            visualization.push_str("**Schema Elements**:\n\n");
            
            for element in &structure.schema_elements {
                visualization.push_str(&format!("- **{}** ({}): {}\n", 
                    element.path, 
                    element.value_type, 
                    match element.element_type {
                        SchemaElementType::Package => "Package",
                        SchemaElementType::Dependency => "Dependency",
                        SchemaElementType::DevDependency => "Dev Dependency",
                        SchemaElementType::BuildDependency => "Build Dependency",
                        SchemaElementType::Feature => "Feature",
                        SchemaElementType::Binary => "Binary",
                        SchemaElementType::Benchmark => "Benchmark",
                        SchemaElementType::Test => "Test",
                        SchemaElementType::Profile => "Profile",
                        SchemaElementType::Metadata => "Metadata",
                        SchemaElementType::Workspace => "Workspace",
                        SchemaElementType::Patch => "Patch",
                        SchemaElementType::Replace => "Replace",
                        SchemaElementType::Lib => "Lib",
                        SchemaElementType::Target => "Target",
                        SchemaElementType::Custom => "Custom",
                    }
                ));
                
                if !element.children.is_empty() {
                    visualization.push_str(&format!("  - Children: {}\n", element.children.join(", ")));
                }
                
                if !element.properties.is_empty() {
                    visualization.push_str("  - Properties:\n");
                    for (key, value) in &element.properties {
                        visualization.push_str(&format!("    - {}: {}\n", key, value));
                    }
                }
            }
            
            // Workload patterns
            if !structure.workload_patterns.is_empty() {
                visualization.push_str("\n**Workload Patterns**:\n\n");
                
                for pattern in &structure.workload_patterns {
                    visualization.push_str(&format!("- **{}** ({}): {}\n", 
                        pattern.name, 
                        match pattern.pattern_type {
                            WorkloadPatternType::BinaryTarget => "Binary Target",
                            WorkloadPatternType::TestSuite => "Test Suite",
                            WorkloadPatternType::BenchmarkSuite => "Benchmark Suite",
                            WorkloadPatternType::FeatureFlag => "Feature Flag",
                            WorkloadPatternType::ConditionalCompilation => "Conditional Compilation",
                            WorkloadPatternType::WorkspaceMember => "Workspace Member",
                            WorkloadPatternType::DependencyOverride => "Dependency Override",
                            WorkloadPatternType::ProfileOptimization => "Profile Optimization",
                            WorkloadPatternType::MetadataExtension => "Metadata Extension",
                            WorkloadPatternType::CustomBuildScript => "Custom Build Script",
                        },
                        pattern.location
                    ));
                    
                    if !pattern.related_elements.is_empty() {
                        visualization.push_str(&format!("  - Related Elements: {}\n", pattern.related_elements.join(", ")));
                    }
                    
                    if !pattern.properties.is_empty() {
                        visualization.push_str("  - Properties:\n");
                        for (key, value) in &pattern.properties {
                            visualization.push_str(&format!("    - {}: {}\n", key, value));
                        }
                    }
                }
            }
            
            visualization.push_str("\n---\n\n");
        }
        
        // Pattern summary
        visualization.push_str("## Workload Pattern Summary\n\n");
        
        let mut pattern_counts: HashMap<WorkloadPatternType, usize> = HashMap::new();
        for structure in &graph.toml_structures {
            for pattern in &structure.workload_patterns {
                *pattern_counts.entry(pattern.pattern_type.clone()).or_insert(0) += 1;
            }
        }
        
        for (pattern_type, count) in pattern_counts {
            visualization.push_str(&format!("- **{}**: {}\n", 
                match pattern_type {
                    WorkloadPatternType::BinaryTarget => "Binary Targets",
                    WorkloadPatternType::TestSuite => "Test Suites",
                    WorkloadPatternType::BenchmarkSuite => "Benchmark Suites",
                    WorkloadPatternType::FeatureFlag => "Feature Flags",
                    WorkloadPatternType::ConditionalCompilation => "Conditional Compilations",
                    WorkloadPatternType::WorkspaceMember => "Workspace Members",
                    WorkloadPatternType::DependencyOverride => "Dependency Overrides",
                    WorkloadPatternType::ProfileOptimization => "Profile Optimizations",
                    WorkloadPatternType::MetadataExtension => "Metadata Extensions",
                    WorkloadPatternType::CustomBuildScript => "Custom Build Scripts",
                },
                count
            ));
        }
        
        fs::write(output_path, visualization)
            .context("Failed to write TOML structure visualization")?;
        
        Ok(())
    }
    
    /// Generate TOML structure graph visualization (DOT format)
    pub fn generate_toml_structure_graph(&self, graph: &GlobalDependencyGraph, output_path: &Path) -> Result<()> {
        let mut dot = String::new();
        dot.push_str("digraph TOML_Structure {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=filled, fillcolor=lightblue];\n");
        dot.push_str("    edge [color=gray];\n\n");
        
        // Add nodes for each TOML file
        for (i, structure) in graph.toml_structures.iter().enumerate() {
            let file_id = format!("file_{}", i);
            let label = format!("{}\n{}", structure.crate_name, structure.crate_version);
            let shape = if structure.is_workspace { "folder" } else { "box" };
            let color = if structure.is_workspace { "lightgreen" } else { "lightblue" };
            
            dot.push_str(&format!("    {} [label=\"{}\", shape={}, fillcolor={}];\n", 
                file_id, label, shape, color));
        }
        
        // Add schema elements as subgraphs
        for (i, structure) in graph.toml_structures.iter().enumerate() {
            let file_id = format!("file_{}", i);
            dot.push_str(&format!("    subgraph cluster_{} {{\n", i));
            dot.push_str(&format!("        label=\"{}\";\n", structure.crate_name));
            dot.push_str("        style=dashed;\n");
            dot.push_str("        color=gray;\n");
            
            for (j, element) in structure.schema_elements.iter().enumerate() {
                let element_id = format!("file_{}_elem_{}", i, j);
                let element_label = format!("{}\n{}", 
                    element.path, 
                    match element.element_type {
                        SchemaElementType::Package => "Package",
                        SchemaElementType::Dependency => "Dep",
                        SchemaElementType::DevDependency => "DevDep",
                        SchemaElementType::BuildDependency => "BuildDep",
                        SchemaElementType::Feature => "Feature",
                        SchemaElementType::Binary => "Binary",
                        SchemaElementType::Benchmark => "Benchmark",
                        SchemaElementType::Test => "Test",
                        SchemaElementType::Profile => "Profile",
                        SchemaElementType::Metadata => "Metadata",
                        SchemaElementType::Workspace => "Workspace",
                        SchemaElementType::Patch => "Patch",
                        SchemaElementType::Replace => "Replace",
                        SchemaElementType::Lib => "Lib",
                        SchemaElementType::Target => "Target",
                        SchemaElementType::Custom => "Custom",
                    }
                );
                
                let element_color = match element.element_type {
                    SchemaElementType::Package => "lightyellow",
                    SchemaElementType::Dependency | SchemaElementType::DevDependency | SchemaElementType::BuildDependency => "lightpink",
                    SchemaElementType::Feature => "lightgreen",
                    SchemaElementType::Binary | SchemaElementType::Benchmark | SchemaElementType::Test => "lightblue",
                    SchemaElementType::Metadata => "lightgray",
                    SchemaElementType::Workspace => "palegreen",
                    _ => "white",
                };
                
                dot.push_str(&format!("        {} [label=\"{}\", shape=box, style=filled, fillcolor={}];\n", 
                    element_id, element_label, element_color));
                
                // Connect element to file
                dot.push_str(&format!("        {} -> {} [style=dashed];\n", file_id, element_id));
            }
            
            dot.push_str("    }\n\n");
        }
        
        // Add workload patterns
        for (i, structure) in graph.toml_structures.iter().enumerate() {
            for (j, pattern) in structure.workload_patterns.iter().enumerate() {
                let pattern_id = format!("pattern_{}_{}", i, j);
                let pattern_label = format!("{}\n{}", 
                    pattern.name, 
                    match pattern.pattern_type {
                        WorkloadPatternType::BinaryTarget => "Binary",
                        WorkloadPatternType::TestSuite => "Test",
                        WorkloadPatternType::BenchmarkSuite => "Benchmark",
                        WorkloadPatternType::FeatureFlag => "Feature",
                        WorkloadPatternType::ConditionalCompilation => "Conditional",
                        WorkloadPatternType::WorkspaceMember => "Workspace",
                        WorkloadPatternType::DependencyOverride => "Override",
                        WorkloadPatternType::ProfileOptimization => "Profile",
                        WorkloadPatternType::MetadataExtension => "Metadata",
                        WorkloadPatternType::CustomBuildScript => "Build",
                    }
                );
                
                let pattern_color = match pattern.pattern_type {
                    WorkloadPatternType::BinaryTarget => "lightblue",
                    WorkloadPatternType::TestSuite => "lightgreen",
                    WorkloadPatternType::BenchmarkSuite => "lightyellow",
                    WorkloadPatternType::FeatureFlag => "lightpink",
                    WorkloadPatternType::ConditionalCompilation => "lightgray",
                    WorkloadPatternType::WorkspaceMember => "palegreen",
                    WorkloadPatternType::DependencyOverride => "orange",
                    WorkloadPatternType::ProfileOptimization => "purple",
                    WorkloadPatternType::MetadataExtension => "gray",
                    WorkloadPatternType::CustomBuildScript => "brown",
                };
                
                dot.push_str(&format!("    {} [label=\"{}\", shape=ellipse, style=filled, fillcolor={}];\n", 
                    pattern_id, pattern_label, pattern_color));
                
                let file_id = format!("file_{}", i);
                dot.push_str(&format!("    {} -> {} [color=red, style=bold];\n", file_id, pattern_id));
            }
        }
        
        dot.push_str("}\n");
        
        fs::write(output_path, dot)
            .context("Failed to write TOML structure graph")?;
        
        Ok(())
    }
    
    /// Perform graph partitioning using external tool (KaMinPar)
    pub fn perform_graph_partitioning(
        &self,
        graph: &GlobalDependencyGraph,
        config: PartitioningConfig,
        output_dir: &Path
    ) -> Result<GlobalDependencyGraph> {
        println!("Performing graph partitioning with {:?}...", config.algorithm);
        
        // Create output directory
        fs::create_dir_all(output_dir)
            .context("Failed to create output directory")?;
        
        // Step 1: Export graph in format suitable for partitioning tool
        let input_file = output_dir.join("graph_for_partitioning.txt");
        self.export_graph_for_partitioning(graph, &input_file)?;
        
        // Step 2: Call external partitioning tool
        let partition_result = self.call_partitioning_tool(&input_file, &config)?;
        
        // Step 3: Import partition results back into our graph
        let mut partitioned_graph = graph.clone();
        partitioned_graph.partitions = self.import_partition_results(&partition_result)?;
        
        // Step 4: Calculate partition metrics
        self.calculate_partition_metrics(&mut partitioned_graph)?;
        
        // Step 5: Generate visualizations for each partition
        self.generate_partition_visualizations(&partitioned_graph, output_dir)?;
        
        println!("Graph partitioning complete!");
        println!("- Partitions created: {}", partitioned_graph.partitions.len());
        println!("- Average partition size: {}", 
            partitioned_graph.partitions.iter().map(|p| p.node_count).sum::<usize>() as f64 / partitioned_graph.partitions.len() as f64);
        
        Ok(partitioned_graph)
    }
    
    /// Export graph in format suitable for KaMinPar
    fn export_graph_for_partitioning(&self, graph: &GlobalDependencyGraph, output_path: &Path) -> Result<()> {
        let mut content = String::new();
        
        // KaMinPar format: first line is node count and edge count
        // Then each line is "from to weight"
        
        // Create node mapping
        let mut node_mapping: HashMap<String, usize> = HashMap::new();
        for (i, node) in graph.nodes.iter().enumerate() {
            node_mapping.insert(node.id.clone(), i);
        }
        
        let node_count = graph.nodes.len();
        let edge_count = graph.edges.len();
        
        content.push_str(&format!("{} {}\n", node_count, edge_count));
        
        // Add edges
        for edge in &graph.edges {
            if let (Some(from_idx), Some(to_idx)) = (node_mapping.get(&edge.from), node_mapping.get(&edge.to)) {
                // Use weight 1 for all edges (unweighted graph)
                content.push_str(&format!("{} {} 1\n", from_idx, to_idx));
            }
        }
        
        fs::write(output_path, content)
            .context("Failed to write partitioning input file")?;
        
        Ok(())
    }
    
    /// Call external partitioning tool (KaMinPar)
    fn call_partitioning_tool(&self, input_file: &Path, config: &PartitioningConfig) -> Result<PathBuf> {
        let output_file = input_file.parent().unwrap().join("partition_result.txt");
        
        // Check if KaMinPar is available
        let kaminpar_path = if cfg!(target_os = "linux") {
            PathBuf::from("/home/mdupont/04/20/KaMinPar")
        } else {
            // Default path - user should configure this
            PathBuf::from("KaMinPar")
        };
        
        if !kaminpar_path.exists() {
            println!("Warning: KaMinPar not found at {}. Using fallback partitioning.", kaminpar_path.display());
            // Create a simple fallback partition
            self.create_fallback_partition(input_file, &output_file, config.partition_count)?;
            return Ok(output_file);
        }
        
        // Build KaMinPar command
        let mut cmd = Command::new(kaminpar_path);
        cmd.arg(input_file)
            .arg(&output_file)
            .arg("--k").arg(config.partition_count.to_string())
            .arg("--imbalance").arg(config.balance_factor.to_string())
            .arg("--seed").arg("42"); // Fixed seed for reproducibility
        
        println!("Running KaMinPar: {:?}", cmd);
        
        let output = cmd.output()
            .context("Failed to execute KaMinPar")?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            println!("KaMinPar failed: {}", error_msg);
            println!("Falling back to simple partitioning...");
            self.create_fallback_partition(input_file, &output_file, config.partition_count)?;
        }
        
        Ok(output_file)
    }
    
    /// Create fallback partition when KaMinPar is not available
    fn create_fallback_partition(&self, input_file: &Path, output_file: &Path, partition_count: usize) -> Result<()> {
        // Read the input file to get node count
        let content = fs::read_to_string(input_file)
            .context("Failed to read partitioning input file")?;
        
        let mut lines = content.lines();
        let first_line = lines.next().ok_or_else(|| anyhow::anyhow!("Invalid input file"))?;
        let parts: Vec<&str> = first_line.split_whitespace().collect();
        if parts.len() < 2 {
            return Err(anyhow::anyhow!("Invalid input file format"));
        }
        
        let node_count = parts[0].parse::<usize>()?;
        
        // Simple round-robin partitioning
        let mut partition_assignments = Vec::with_capacity(node_count);
        for i in 0..node_count {
            let partition_id = i % partition_count;
            partition_assignments.push(partition_id);
        }
        
        // Write output in KaMinPar format
        let mut output_content = String::new();
        for assignment in partition_assignments {
            output_content.push_str(&format!("{}\n", assignment));
        }
        
        fs::write(output_file, output_content)
            .context("Failed to write fallback partition file")?;
        
        Ok(())
    }
    
    /// Import partition results from KaMinPar output
    fn import_partition_results(&self, partition_file: &Path) -> Result<Vec<GraphPartition>> {
        let content = fs::read_to_string(partition_file)
            .context("Failed to read partition result file")?;
        
        let mut partitions: HashMap<usize, GraphPartition> = HashMap::new();
        
        for (node_idx, line) in content.lines().enumerate() {
            let partition_id = line.trim().parse::<usize>()
                .context("Failed to parse partition ID")?;
            
            let partition = partitions.entry(partition_id).or_insert(GraphPartition {
                partition_id,
                node_ids: Vec::new(),
                node_count: 0,
                edge_count: 0,
                is_core: false,
                properties: HashMap::new(),
                metrics: PartitionMetrics {
                    internal_edges: 0,
                    external_edges: 0,
                    density: 0.0,
                    modularity: 0.0,
                    cohesion: 0.0,
                    coupling: 0.0,
                },
            });
            
            partition.node_ids.push(node_idx.to_string());
            partition.node_count += 1;
        }
        
        let mut result: Vec<GraphPartition> = partitions.into_values().collect();
        result.sort_by_key(|p| p.partition_id);
        
        Ok(result)
    }
    
    /// Calculate partition metrics
    fn calculate_partition_metrics(&self, graph: &mut GlobalDependencyGraph) -> Result<()> {
        // Create node to partition mapping
        let mut node_to_partition: HashMap<String, usize> = HashMap::new();
        for partition in &graph.partitions {
            for node_id in &partition.node_ids {
                node_to_partition.insert(node_id.clone(), partition.partition_id);
            }
        }
        
        // Calculate metrics for each partition
        for partition in &mut graph.partitions {
            let mut internal_edges = 0;
            let mut external_edges = 0;
            
            // Count edges
            for edge in &graph.edges {
                if let (Some(from_part), Some(to_part)) = (
                    node_to_partition.get(&edge.from),
                    node_to_partition.get(&edge.to)
                ) {
                    if from_part == to_part && *from_part == partition.partition_id {
                        internal_edges += 1;
                    } else if *from_part == partition.partition_id || *to_part == partition.partition_id {
                        external_edges += 1;
                    }
                }
            }
            
            partition.edge_count = internal_edges + external_edges;
            partition.metrics.internal_edges = internal_edges;
            partition.metrics.external_edges = external_edges;
            
            // Calculate density (internal edges / possible edges)
            let possible_edges = partition.node_count * (partition.node_count - 1);
            partition.metrics.density = if possible_edges > 0 {
                internal_edges as f64 / possible_edges as f64
            } else {
                0.0
            };
            
            // Calculate cohesion (internal edges / total edges)
            let total_edges = internal_edges + external_edges;
            partition.metrics.cohesion = if total_edges > 0 {
                internal_edges as f64 / total_edges as f64
            } else {
                0.0
            };
            
            // Calculate coupling (external edges / total edges)
            partition.metrics.coupling = if total_edges > 0 {
                external_edges as f64 / total_edges as f64
            } else {
                0.0
            };
            
            // Simple modularity estimate
            partition.metrics.modularity = partition.metrics.cohesion - partition.metrics.coupling;
            
            // Mark core partitions (high cohesion, low coupling)
            partition.is_core = partition.metrics.cohesion > 0.7 && partition.metrics.coupling < 0.3;
        }
        
        Ok(())
    }
    
    /// Generate visualizations for each partition
    fn generate_partition_visualizations(&self, graph: &GlobalDependencyGraph, output_dir: &Path) -> Result<()> {
        println!("Generating partition visualizations...");
        
        // Create partitions directory
        let partitions_dir = output_dir.join("partitions");
        fs::create_dir_all(&partitions_dir)
            .context("Failed to create partitions directory")?;
        
        // Generate overview visualization
        self.generate_partition_overview(graph, &partitions_dir)?;
        
        // Generate individual partition visualizations
        for partition in &graph.partitions {
            self.generate_single_partition_visualization(graph, partition, &partitions_dir)?;
        }
        
        // Generate SVG versions
        self.convert_dot_to_svg(&partitions_dir)?;
        
        Ok(())
    }
    
    /// Generate partition overview visualization
    fn generate_partition_overview(&self, graph: &GlobalDependencyGraph, output_dir: &Path) -> Result<()> {
        let overview_path = output_dir.join("partition_overview.dot");
        let mut dot = String::new();
        
        dot.push_str("digraph Partition_Overview {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=box, style=filled];\n");
        dot.push_str("    edge [penwidth=2];\n\n");
        
        // Add partition nodes
        for partition in &graph.partitions {
            let color = if partition.is_core { "lightgreen" } else { "lightblue" };
            let label = format!("Partition {}\\nNodes: {}\\nEdges: {}\\nDensity: {:.2}",
                partition.partition_id,
                partition.node_count,
                partition.edge_count,
                partition.metrics.density
            );
            
            dot.push_str(&format!("    partition_{} [label=\"{}\", fillcolor={}];\n", 
                partition.partition_id, label, color));
        }
        
        // Add edges between partitions (based on inter-partition dependencies)
        let node_to_partition: HashMap<_, _> = graph.partitions.iter()
            .flat_map(|p| p.node_ids.iter().map(move |n| (n.clone(), p.partition_id)))
            .collect();
        
        let mut inter_partition_edges: HashMap<(usize, usize), usize> = HashMap::new();
        
        for edge in &graph.edges {
            if let (Some(from_part), Some(to_part)) = (
                node_to_partition.get(&edge.from),
                node_to_partition.get(&edge.to)
            ) {
                if from_part != to_part {
                    let key = if from_part < to_part {
                        (*from_part, *to_part)
                    } else {
                        (*to_part, *from_part)
                    };
                    *inter_partition_edges.entry(key).or_insert(0) += 1;
                }
            }
        }
        
        for ((from_part, to_part), count) in inter_partition_edges {
            let width = (count as f64 / 10.0).max(1.0).min(5.0);
            dot.push_str(&format!("    partition_{} -> partition_{} [label=\"{}\", penwidth={}];\n",
                from_part, to_part, count, width));
        }
        
        dot.push_str("}\n");
        
        fs::write(overview_path, dot)
            .context("Failed to write partition overview")?;
        
        Ok(())
    }
    
    /// Generate visualization for a single partition
    fn generate_single_partition_visualization(
        &self,
        graph: &GlobalDependencyGraph,
        partition: &GraphPartition,
        output_dir: &Path
    ) -> Result<()> {
        let partition_dir = output_dir.join(format!("partition_{}", partition.partition_id));
        fs::create_dir_all(&partition_dir)
            .context("Failed to create partition directory")?;
        
        let dot_path = partition_dir.join("partition.dot");
        let mut dot = String::new();
        
        dot.push_str(&format!("digraph Partition_{} {{\n", partition.partition_id));
        dot.push_str("    rankdir=LR;\n");
        dot.push_str(&format!("    label=\"Partition {} \";\n", partition.partition_id));
        dot.push_str(format!("{}\";\n", partition.partition_id).as_str());
        dot.push_str("    labelloc=top;\n");
        dot.push_str("    labeljust=center;\n");
        dot.push_str("    node [shape=box, style=filled, fillcolor=lightblue];\n");
        dot.push_str("    edge [color=gray];\n\n");
        
        // Create node ID to index mapping
        let node_map: HashMap<_, _> = graph.nodes.iter()
            .enumerate()
            .map(|(i, n)| (n.id.clone(), i))
            .collect();
        
        // Add nodes in this partition
        let mut node_indices = Vec::new();
        for node_id in &partition.node_ids {
            if let Some(idx) = node_map.get(node_id) {
                let node = &graph.nodes[*idx];
                let label = format!("{}\\nv{}", node.crate_name, node.version);
                
                dot.push_str(&format!("    node_{} [label=\"{}\"];\n", 
                    idx, label));
                node_indices.push(*idx);
            }
        }
        
        // Add internal edges
        for edge in &graph.edges {
            if let (Some(from_idx), Some(to_idx)) = (
                node_map.get(&edge.from),
                node_map.get(&edge.to)
            ) {
                if node_indices.contains(&from_idx) && node_indices.contains(&to_idx) {
                    let edge_type = match edge.edge_type {
                        DependencyEdgeType::Direct => "Direct",
                        DependencyEdgeType::Transitive => "Transitive",
                        DependencyEdgeType::Dev => "Dev",
                        DependencyEdgeType::Build => "Build",
                        DependencyEdgeType::FeatureGated => "Feature",
                        _ => "Other",
                    };
                    
                    dot.push_str(&format!("    node_{} -> node_{} [label=\"{}\"];\n",
                        from_idx, to_idx, edge_type));
                }
            }
        }
        
        // Add metrics as a comment
        dot.push_str(&format!("\n    // Partition Metrics\n"));
        dot.push_str(&format!("    // Nodes: {}\n", partition.node_count));
        dot.push_str(&format!("    // Edges: {}\n", partition.edge_count));
        dot.push_str(&format!("    // Density: {:.3}\n", partition.metrics.density));
        dot.push_str(&format!("    // Cohesion: {:.3}\n", partition.metrics.cohesion));
        dot.push_str(&format!("    // Coupling: {:.3}\n", partition.metrics.coupling));
        dot.push_str(&format!("    // Modularity: {:.3}\n", partition.metrics.modularity));
        
        dot.push_str("}\n");
        
        fs::write(dot_path, dot)
            .context("Failed to write partition visualization")?;
        
        // Also generate a summary file
        let summary_path = partition_dir.join("summary.txt");
        let mut summary = String::new();
        
        summary.push_str(&format!("Partition {} Summary\n", partition.partition_id));
        summary.push_str(&format!("=====================\n\n"));
        summary.push_str(&format!("Partition ID: {}\n", partition.partition_id));
        summary.push_str(&format!("Node Count: {}\n", partition.node_count));
        summary.push_str(&format!("Edge Count: {}\n", partition.edge_count));
        summary.push_str(&format!("Is Core: {}\n\n", partition.is_core));
        
        summary.push_str("Metrics:\n");
        summary.push_str(&format!("  Density: {:.3}\n", partition.metrics.density));
        summary.push_str(&format!("  Cohesion: {:.3}\n", partition.metrics.cohesion));
        summary.push_str(&format!("  Coupling: {:.3}\n", partition.metrics.coupling));
        summary.push_str(&format!("  Modularity: {:.3}\n\n", partition.metrics.modularity));
        
        summary.push_str("Nodes:\n");
        for node_id in &partition.node_ids {
            if let Some(idx) = node_map.get(node_id) {
                let node = &graph.nodes[*idx];
                summary.push_str(&format!("  - {} v{} ({})\n", 
                    node.crate_name, node.version, node.id));
            }
        }
        
        fs::write(summary_path, summary)
            .context("Failed to write partition summary")?;
        
        Ok(())
    }
    
    /// Convert DOT files to SVG using dot command
    fn convert_dot_to_svg(&self, directory: &Path) -> Result<()> {
        println!("Converting DOT files to SVG...");
        
        // Find all .dot files
        let mut dot_files = Vec::new();
        self.find_dot_files(directory, &mut dot_files)?;
        
        for dot_file in dot_files {
            let svg_file = dot_file.with_extension("svg");
            
            let output = Command::new("dot")
                .arg("-Tsvg")
                .arg(&dot_file)
                .arg("-o")
                .arg(&svg_file)
                .output()
                .context("Failed to execute dot command")?;
            
            if !output.status.success() {
                let error_msg = String::from_utf8_lossy(&output.stderr);
                println!("Warning: Failed to convert {} to SVG: {}", 
                    dot_file.display(), error_msg);
                continue;
            }
            
            println!("Generated SVG: {}", svg_file.display());
        }
        
        Ok(())
    }
    
    /// Find all .dot files recursively
    fn find_dot_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                
                if path.extension() == Some("dot".as_ref()) {
                    files.push(path.clone());
                }
                
                if path.is_dir() {
                    self.find_dot_files(&path, files)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Generate comprehensive partition report
    pub fn generate_partition_report(
        &self,
        graph: &GlobalDependencyGraph,
        output_path: &Path
    ) -> Result<()> {
        let mut report = String::new();
        
        report.push_str("# Graph Partitioning Analysis Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Summary
        report.push_str("## Partition Summary\n\n");
        report.push_str(&format!("- **Total Partitions**: {}\n", graph.partitions.len()));
        report.push_str(&format!("- **Total Nodes**: {}\n", graph.metrics.node_count));
        report.push_str(&format!("- **Total Edges**: {}\n", graph.metrics.edge_count));
        
        let core_partitions = graph.partitions.iter().filter(|p| p.is_core).count();
        report.push_str(&format!("- **Core Partitions**: {}\n", core_partitions));
        
        let avg_partition_size = graph.partitions.iter().map(|p| p.node_count).sum::<usize>() as f64 / graph.partitions.len() as f64;
        report.push_str(&format!("- **Average Partition Size**: {:.1}\n", avg_partition_size));
        
        let avg_density = graph.partitions.iter().map(|p| p.metrics.density).sum::<f64>() / graph.partitions.len() as f64;
        report.push_str(&format!("- **Average Density**: {:.3}\n", avg_density));
        
        let avg_cohesion = graph.partitions.iter().map(|p| p.metrics.cohesion).sum::<f64>() / graph.partitions.len() as f64;
        report.push_str(&format!("- **Average Cohesion**: {:.3}\n", avg_cohesion));
        
        let avg_coupling = graph.partitions.iter().map(|p| p.metrics.coupling).sum::<f64>() / graph.partitions.len() as f64;
        report.push_str(&format!("- **Average Coupling**: {:.3}\n\n", avg_coupling));
        
        // Partition Details
        report.push_str("## Partition Details\n\n");
        
        for partition in &graph.partitions {
            report.push_str(&format!("### Partition {}\n\n", partition.partition_id));
            report.push_str(&format!("- **Nodes**: {}\n", partition.node_count));
            report.push_str(&format!("- **Edges**: {}\n", partition.edge_count));
            report.push_str(&format!("- **Core Partition**: {}\n\n", partition.is_core));
            
            report.push_str("**Metrics**:\n\n");
            report.push_str(&format!("| Metric | Value |\n"));
            report.push_str(&format!("|--------|-------|\n"));
            report.push_str(&format!("| Density | {:.3} |\n", partition.metrics.density));
            report.push_str(&format!("| Cohesion | {:.3} |\n", partition.metrics.cohesion));
            report.push_str(&format!("| Coupling | {:.3} |\n", partition.metrics.coupling));
            report.push_str(&format!("| Modularity | {:.3} |\n", partition.metrics.modularity));
            
            report.push_str("\n**Key Nodes**:\n\n");
            
            // Find top nodes by degree in this partition
            let mut node_degrees: HashMap<String, usize> = HashMap::new();
            for node_id in &partition.node_ids {
                node_degrees.insert(node_id.clone(), 0);
            }
            
            for edge in &graph.edges {
                if partition.node_ids.contains(&edge.from) {
                    *node_degrees.entry(edge.from.clone()).or_insert(0) += 1;
                }
                if partition.node_ids.contains(&edge.to) {
                    *node_degrees.entry(edge.to.clone()).or_insert(0) += 1;
                }
            }
            
            let mut sorted_nodes: Vec<_> = node_degrees.iter().collect();
            sorted_nodes.sort_by(|a, b| b.1.cmp(a.1));
            
            let node_map: HashMap<_, _> = graph.nodes.iter().map(|n| (n.id.clone(), n)).collect();
            
            for (node_id, degree) in sorted_nodes.iter().take(5) {
                if let Some(node) = node_map.get(*node_id) {
                    report.push_str(&format!("- {} v{} (degree: {})\n", 
                        node.crate_name, node.version, degree));
                }
            }
            
            report.push_str("---\n\n");
        }
        
        // Recommendations
        report.push_str("## Recommendations\n\n");
        
        if core_partitions > 0 {
            report.push_str(&format!("✅ **{} Core Partitions Identified**: These are highly cohesive, loosely coupled modules that represent the architectural core of the system.\n\n", core_partitions));
        }
        
        let low_cohesion_partitions: Vec<_> = graph.partitions.iter()
            .filter(|p| p.metrics.cohesion < 0.5)
            .collect();
        
        if !low_cohesion_partitions.is_empty() {
            report.push_str("⚠️ **Low Cohesion Partitions**: Consider refactoring these partitions for better modularity:\n\n");
            for partition in low_cohesion_partitions {
                report.push_str(&format!("- Partition {} (Cohesion: {:.3})\n", 
                    partition.partition_id, partition.metrics.cohesion));
            }
            report.push_str("\n");
        }
        
        let high_coupling_partitions: Vec<_> = graph.partitions.iter()
            .filter(|p| p.metrics.coupling > 0.7)
            .collect();
        
        if !high_coupling_partitions.is_empty() {
            report.push_str("⚠️ **High Coupling Partitions**: These partitions have many external dependencies:\n\n");
            for partition in high_coupling_partitions {
                report.push_str(&format!("- Partition {} (Coupling: {:.3})\n", 
                    partition.partition_id, partition.metrics.coupling));
            }
            report.push_str("\n");
        }
        
        report.push_str("📊 **Overall**: The partitioning reveals the modular structure of the system. Use this analysis to understand component boundaries and improve architectural design.\n");
        
        fs::write(output_path, report)
            .context("Failed to write partition report")?;
        
        Ok(())
    }
}