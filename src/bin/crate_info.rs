//! Standalone tool to list all crates with their git repository information

use anyhow::{Context, Result};
use lazy_static::lazy_static;
use petgraph::graph::{Graph, NodeIndex};
use petgraph::visit::Bfs;
use rayon::prelude::*;
use regex::Regex;
use serde::{Deserialize, Serialize};
use cargo_metadata::{MetadataCommand, DependencyKind};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

lazy_static! {
    static ref GITHUB_URL_RE: Regex =
        Regex::new(r"github\.com/([^/]+)/([^/.]+)(?:\.git)?").unwrap();
    static ref PACKAGE_RE: Regex =
        Regex::new(r#"\[\[package\]\]\s+name\s*=\s*"([^"]+)"\s+version\s*=\s*"([^"]+)"\s+source\s*=\s*"([^"]+)""#).unwrap();
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RepoStatus {
    Exists,
    NotFound,
    Deleted,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoCache {
    pub repos: HashMap<String, RepoStatus>,
}

impl RepoCache {
    fn new() -> Self {
        Self { repos: HashMap::new() }
    }

    fn load(path: &Path) -> Self {
        if let Ok(content) = fs::read_to_string(path) {
            if let Ok(cache) = serde_json::from_str(&content) {
                return cache;
            }
        }
        Self::new()
    }

    fn save(&self, path: &Path) -> Result<()> {
        let json = serde_json::to_string_pretty(self).context("Failed to serialize cache")?;
        fs::write(path, json).context("Failed to write cache")?;
        Ok(())
    }

    fn get(&self, url: &str) -> Option<&RepoStatus> {
        self.repos.get(url)
    }

    fn insert(&mut self, url: String, status: RepoStatus) {
        self.repos.insert(url, status);
    }

    fn is_retryable(&self, url: &str) -> bool {
        match self.get(url) {
            Some(RepoStatus::Exists) => true,
            Some(RepoStatus::NotFound | RepoStatus::Deleted) => false,
            Some(RepoStatus::Error(_)) => true,
            None => true,
        }
    }
}

fn check_repo_exists_with_curl(url: &str) -> RepoStatus {
    let clean_url = url.trim_end_matches(".git");

    let output = Command::new("curl")
        .args(["-sI", "-o", "/dev/null", "-w", "%{http_code}", "--max-time", "10", clean_url])
        .output();

    match output {
        Ok(out) => {
            let code = String::from_utf8_lossy(&out.stdout).trim().to_string();
            match code.as_str() {
                "200" | "301" | "302" => RepoStatus::Exists,
                "404" => RepoStatus::NotFound,
                "410" => RepoStatus::Deleted,
                "000" => RepoStatus::Error("Connection timeout".to_string()),
                other => RepoStatus::Error(format!("HTTP {}", other)),
            }
        }
        Err(e) => RepoStatus::Error(format!("Curl error: {}", e)),
    }
}

fn check_and_cache_repo(url: &str, cache: &std::sync::Mutex<RepoCache>, cache_path: &Path) -> bool {
    let status = {
        let mut cache = cache.lock().unwrap();
        if let Some(status) = cache.get(url) {
            match status {
                RepoStatus::Exists => return true,
                RepoStatus::NotFound | RepoStatus::Deleted => return false,
                RepoStatus::Error(_) => {}
            }
        }

        println!("  Checking: {}", url);
        let status = check_repo_exists_with_curl(url);
        println!("    Status: {:?}", status);

        cache.insert(url.to_string(), status.clone());
        cache.save(cache_path).ok();

        status
    };

    matches!(status, RepoStatus::Exists)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrateInfo {
    pub name: Option<String>,
    pub version: Option<String>,
    pub repository: Option<String>,
    pub manifest_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneResult {
    pub url: String,
    pub mirror_path: String,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub repo_path: String,
    pub found_urls: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanReport {
    pub scan_path: String,
    pub crates: Vec<CrateInfo>,
    pub clone_results: Vec<CloneResult>,
    pub search_results: Vec<SearchResult>,
    pub total_unique_repos: usize,
    pub iterations: usize,
    pub cpu_cores: usize,
    pub duration_secs: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPartition {
    pub partition_id: usize,
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartitionReport {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub num_partitions: usize,
    pub partitions: Vec<GraphPartition>,
    pub partitioning_method: String,
    pub partition_sizes: Vec<usize>,
}

fn extract_repo_name(url: &str) -> String {
    url.trim_end_matches(".git")
        .split('/')
        .last()
        .unwrap_or("unknown")
        .to_string()
}

fn get_cargo_metadata_deps(workspace_path: &Path) -> Vec<(String, String)> {
    let mut deps = Vec::new();

    if let Ok(metadata) = MetadataCommand::new()
        .manifest_path(workspace_path.join("Cargo.toml"))
        .exec()
    {
        for package in &metadata.packages {
            let crate_name = package.name.clone();
            for dep in &package.dependencies {
                // Skip dev/build dependencies
                if matches!(dep.kind, DependencyKind::Development | DependencyKind::Build) {
                    continue;
                }
                deps.push((crate_name.clone(), dep.name.clone()));
            }
        }
    }
    deps
}

fn build_dependency_graph(_crates: &[CrateInfo], _search_results: &[SearchResult], workspace_path: &Path) -> (Graph<String, ()>, HashMap<String, NodeIndex>) {
    let mut graph: Graph<String, ()> = Graph::new();
    let mut node_indices: HashMap<String, NodeIndex> = HashMap::new();

    // Get ALL dependencies from cargo metadata
    if let Ok(metadata) = MetadataCommand::new()
        .manifest_path(workspace_path.join("Cargo.toml"))
        .exec()
    {
        // Add ALL packages as nodes
        for package in &metadata.packages {
            let name = package.name.clone();
            if !node_indices.contains_key(&name) {
                let idx = graph.add_node(name.clone());
                node_indices.insert(name, idx);
            }
        }

        // Add edges for ALL dependencies
        for package in &metadata.packages {
            let source_name = package.name.clone();
            let Some(&src_idx) = node_indices.get(&source_name) else {
                continue;
            };

            for dep in &package.dependencies {
                // Skip dev/build dependencies
                if matches!(dep.kind, DependencyKind::Development | DependencyKind::Build) {
                    continue;
                }
                let dep_name = dep.name.clone();

                // Add dep node if not exists
                if !node_indices.contains_key(&dep_name) {
                    let idx = graph.add_node(dep_name.clone());
                    node_indices.insert(dep_name.clone(), idx);
                }

                if let Some(&tgt_idx) = node_indices.get(&dep_name) {
                    if src_idx != tgt_idx {
                        graph.add_edge(src_idx, tgt_idx, ());
                    }
                }
            }
        }
    }

    (graph, node_indices)
}

fn partition_graph_round_robin(graph: &Graph<String, ()>, num_partitions: usize) -> Vec<Vec<NodeIndex>> {
    let mut partitions: Vec<Vec<NodeIndex>> = vec![Vec::new(); num_partitions];

    for node in graph.node_indices() {
        let partition_id = node.index() % num_partitions;
        partitions[partition_id].push(node);
    }

    partitions
}

fn partition_graph_by_bfs(graph: &Graph<String, ()>, num_partitions: usize) -> Vec<Vec<NodeIndex>> {
    let mut partitions: Vec<Vec<NodeIndex>> = vec![Vec::new(); num_partitions];
    let mut visited: std::collections::HashSet<NodeIndex> = std::collections::HashSet::new();

    for start in graph.node_indices() {
        if visited.contains(&start) {
            continue;
        }

        let mut bfs = Bfs::new(graph, start);
        let mut partition_id = 0;

        while let Some(node) = bfs.next(graph) {
            if visited.contains(&node) {
                continue;
            }
            visited.insert(node);

            let target_partition = partition_id % num_partitions;
            partitions[target_partition].push(node);
            partition_id += 1;
        }
    }

    partitions
}

fn partition_graph_random(graph: &Graph<String, ()>, num_partitions: usize) -> Vec<Vec<NodeIndex>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut partitions: Vec<Vec<NodeIndex>> = vec![Vec::new(); num_partitions];

    for node in graph.node_indices() {
        let mut hasher = DefaultHasher::new();
        node.index().hash(&mut hasher);
        let partition_id = (hasher.finish() as usize) % num_partitions;
        partitions[partition_id].push(node);
    }

    partitions
}

fn create_partition_report(
    graph: &Graph<String, ()>,
    partitions: Vec<Vec<NodeIndex>>,
    method: &str,
) -> PartitionReport {
    let partition_sizes: Vec<usize> = partitions.iter().map(|p| p.len()).collect();

    let graph_partitions: Vec<GraphPartition> = partitions
        .iter()
        .enumerate()
        .map(|(id, nodes)| {
            let node_names: Vec<String> = nodes
                .iter()
                .map(|n| graph[*n].clone())
                .collect();

            let mut edges = Vec::new();
            for node in nodes {
                for neighbor in graph.neighbors(*node) {
                    let source = graph[*node].clone();
                    let target = graph[neighbor].clone();
                    edges.push((source, target));
                }
            }

            GraphPartition {
                partition_id: id,
                nodes: node_names,
                edges,
            }
        })
        .collect();

    PartitionReport {
        total_nodes: graph.node_count(),
        total_edges: graph.edge_count(),
        num_partitions: partitions.len(),
        partitions: graph_partitions,
        partitioning_method: method.to_string(),
        partition_sizes,
    }
}

fn parse_github_url(url: &str) -> Option<(String, String, String)> {
    let url = url.trim();

    // Handle /tree/ in URL - strip it and everything after
    let url = if let Some(pos) = url.find("/tree/") {
        &url[..pos]
    } else {
        url
    };

    let (host, path) = if url.starts_with("git@") {
        let after_at = url.trim_start_matches("git@");
        if let Some((h, p)) = after_at.split_once(':') {
            (h.to_string(), p.to_string())
        } else {
            return None;
        }
    } else if url.starts_with("https://") || url.starts_with("http://") {
        let cleaned = url.trim_start_matches("https://").trim_start_matches("http://");
        if let Some((h, p)) = cleaned.split_once('/') {
            (h.to_string(), p.to_string())
        } else {
            return None;
        }
    } else {
        return None;
    };

    let path = path.trim_end_matches(".git");
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 2 {
        Some((host, parts[0].to_string(), parts[1].to_string()))
    } else {
        None
    }
}

fn clone_bare_to_mirror(repo_url: &str, mirror_base: &Path, cache: &std::sync::Mutex<RepoCache>, cache_path: &Path) -> Result<PathBuf> {
    {
        let mut cache = cache.lock().unwrap();
        if !cache.is_retryable(repo_url) {
            println!("  Skipping (cached as unavailable): {}", repo_url);
            return Err(anyhow::anyhow!("Repository not available"));
        }
    }

    if !check_and_cache_repo(repo_url, &cache, cache_path) {
        return Err(anyhow::anyhow!("Repository check failed (404/deleted)"));
    }

    let git_exe = std::env::var("GIT_EXECUTABLE").unwrap_or_else(|_| "git".to_string());
    let git_exe = PathBuf::from(git_exe);

    let parsed = parse_github_url(repo_url).ok_or_else(|| anyhow::anyhow!("Failed to parse URL: {}", repo_url))?;
    let (host, owner, repo) = parsed;

    let mirror_path = mirror_base.join(&host).join(&owner).join(format!("{}.git", &repo));
    if mirror_path.exists() {
        println!("  Already exists: {}", mirror_path.display());
        return Ok(mirror_path);
    }

    if let Some(parent) = mirror_path.parent() {
        fs::create_dir_all(parent).context("Failed to create mirror directory")?;
    }

    let clean_url = if let Some(pos) = repo_url.find("/tree/") {
        repo_url[..pos].to_string()
    } else {
        repo_url.to_string()
    };

    println!("  Cloning bare: {} -> {}", clean_url, mirror_path.display());
    let output = Command::new(&git_exe)
        .args(["clone", "--bare", "--quiet", &clean_url, &mirror_path.to_str().unwrap()])
        .output()
        .context(format!("Failed to clone {}", clean_url))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        let err_str = err.to_string();

        if err_str.contains("Could not read") || err_str.contains("error: could not fetch") || err_str.contains("HTTP 404") || err_str.contains("repository not found") {
            {
                let mut cache = cache.lock().unwrap();
                cache.insert(repo_url.to_string(), RepoStatus::Deleted);
                cache.save(cache_path).ok();
            }
            println!("  Repo appears deleted: {}", repo_url);
            return Err(anyhow::anyhow!("Repository appears deleted: {}", err_str));
        }

        anyhow::bail!("Failed to clone {}: {}", clean_url, err);
    }

    Ok(mirror_path)
}

fn search_all_git_objects_for_urls(bare_repo_path: &Path) -> Vec<String> {
    let git_exe = std::env::var("GIT_EXECUTABLE").unwrap_or_else(|_| "git".to_string());
    let git_exe = PathBuf::from(git_exe);

    let mut found_urls = Vec::new();

    // Use git grep to search all objects for URLs - more efficient
    let hosts = ["github.com", "gitlab.com", "bitbucket.org", "sourcehut.org", "codeberg.org", "code.google.com", "fedoraproject.org"];
    for host in hosts {
        let pattern = format!("https?://{}[^[:space:]\"']+", host);
        let output = Command::new(&git_exe)
            .arg("-C")
            .arg(bare_repo_path)
            .arg("grep")
            .arg("-oE")
            .arg(&pattern)
            .arg("--all")
            .arg("--every-match")
            .output();

        if let Ok(output) = output {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                let url = line.trim().to_string();
                if !url.is_empty() && !found_urls.contains(&url) {
                    found_urls.push(url);
                }
            }
        }
    }

    // Also try SSH URLs
    let ssh_patterns = ["git@github.com:", "git@gitlab.com:", "git@bitbucket.org:"];
    for pattern in ssh_patterns {
        let output = Command::new(&git_exe)
            .arg("-C")
            .arg(bare_repo_path)
            .arg("grep")
            .arg("-oE")
            .arg(&format!("{}[^[:space:]\"']+", pattern))
            .arg("--all")
            .arg("--every-match")
            .output();

        if let Ok(output) = output {
            let content = String::from_utf8_lossy(&output.stdout);
            for line in content.lines() {
                let url = line.trim().to_string();
                if !url.is_empty() && !found_urls.contains(&url) {
                    // Convert git@ to https://
                    let https_url = url.replace("git@", "https://").replace(":", "/");
                    if !found_urls.contains(&https_url) {
                        found_urls.push(https_url);
                    }
                }
            }
        }
    }

    found_urls
}

fn scan_directory_for_cargo_repos(base_path: &Path) -> Result<Vec<CrateInfo>> {
    let mut crates = Vec::new();
    let mut visited_paths = std::collections::HashSet::new();

    fn visit_dir(dir: &Path, crates: &mut Vec<CrateInfo>, visited: &mut std::collections::HashSet<PathBuf>, depth: usize) -> Result<()> {
        if depth > 20 {
            return Ok(());
        }

        let canonical = dir.canonicalize().unwrap_or_else(|_| dir.to_path_buf());
        if visited.contains(&canonical) {
            return Ok(());
        }
        visited.insert(canonical);

        let cargo_toml = dir.join("Cargo.toml");
        if cargo_toml.exists() {
            eprintln!("Found Cargo.toml: {}", cargo_toml.display());
            if let Ok(content) = fs::read_to_string(&cargo_toml) {
                if let Ok(doc) = content.parse::<toml_edit::DocumentMut>() {
                    let name = doc.get("package").and_then(|p| p.get("name")).and_then(|v| v.as_str()).map(String::from);
                    let version = doc.get("package").and_then(|p| p.get("version")).and_then(|v| v.as_str()).map(String::from);
                    let repository = doc.get("package").and_then(|p| p.get("repository")).and_then(|v| v.as_str()).map(String::from);

                    if let Some(repo) = &repository {
                        eprintln!("  Repository: {}", repo);
                        crates.push(CrateInfo {
                            name,
                            version,
                            repository: Some(repo.clone()),
                            manifest_path: cargo_toml.display().to_string(),
                        });
                    }
                }
            }
        }

        // Also check for Cargo.lock in this directory
        let cargo_lock = dir.join("Cargo.lock");
        if cargo_lock.exists() {
            if let Ok(git_deps) = parse_cargo_lock_for_git_deps(&cargo_lock) {
                for dep in git_deps {
                    crates.push(dep);
                }
            }
        }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.is_dir() {
                    visit_dir(&path, crates, visited, depth + 1)?;
                }
            }
        }
        Ok(())
    }

    eprintln!("Starting scan at: {}", base_path.display());
    visit_dir(base_path, &mut crates, &mut visited_paths, 0)?;
    Ok(crates)
}

fn parse_cargo_lock_for_git_deps(lock_path: &Path) -> Result<Vec<CrateInfo>> {
    let content = fs::read_to_string(lock_path)
        .context("Failed to read Cargo.lock")?;

    let mut deps = Vec::new();
    let mut seen = std::collections::HashSet::new();

    // Regex to match [[package]] blocks with git+ source
    for caps in PACKAGE_RE.captures_iter(&content) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let version = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        let source = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        if !source.starts_with("git+") {
            continue;
        }

        // Extract URL from git+https://...#commit
        let url_part = source.strip_prefix("git+").unwrap_or(source);
        let url = url_part.split('#').next().unwrap_or(url_part);

        // Try to extract GitHub owner/repo
        if let Some(caps) = GITHUB_URL_RE.captures(url) {
            let owner = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            let repo = caps.get(2).map(|m| m.as_str()).unwrap_or("");

            // Avoid duplicates
            let key = format!("{}:{}:{}", owner, repo, version);
            if seen.contains(&key) {
                continue;
            }
            seen.insert(key);

            let repo_url = format!("https://github.com/{}/{}", owner, repo);
            eprintln!("  [Cargo.lock] Found git dep: {} v{} -> {}", name, version, repo_url);

            deps.push(CrateInfo {
                name: Some(name.to_string()),
                version: Some(version.to_string()),
                repository: Some(repo_url),
                manifest_path: lock_path.display().to_string(),
            });
        }
    }

    Ok(deps)
}

fn get_cargo_lock_transitive_deps(lock_path: &Path) -> Vec<(String, String, String)> {
    // Returns: (package_name, dep_name, source_url)
    // Simplified: just extract (name, dep) pairs from Cargo.lock
    let mut deps = Vec::new();

    let content = match fs::read_to_string(lock_path) {
        Ok(c) => c,
        Err(_) => return deps,
    };

    // Parse [[package]] blocks - get name and dependencies
    let re = Regex::new(r#"\[\[package\]\]\s+name\s*=\s*"([^"]+)".*?dependencies\s*=\s*\[(.*?)\]"#).unwrap();

    for caps in re.captures_iter(&content) {
        let pkg_name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let dep_block = caps.get(2).map(|m| m.as_str()).unwrap_or("");
        
        // Extract dependency names from the array
        let dep_name_re = Regex::new(r#"name\s*=\s*"([^"]+)""#).unwrap();
        for dep_cap in dep_name_re.captures_iter(dep_block) {
            if let Some(dep_name) = dep_cap.get(1) {
                deps.push((pkg_name.to_string(), dep_name.as_str().to_string(), String::new()));
            }
        }
    }

    deps
}

fn get_git_deps_from_lock(lock_path: &Path) -> HashMap<String, String> {
    // Map: crate_name -> git_url
    let mut git_deps = HashMap::new();

    let content = match fs::read_to_string(lock_path) {
        Ok(c) => c,
        Err(_) => return git_deps,
    };

    let package_re = Regex::new(r#"\[\[package\]\]\s+name\s*=\s*"([^"]+)"\s+version\s*=\s*"([^"]+)"\s+source\s*=\s*"([^"]+)""#).unwrap();

    for caps in package_re.captures_iter(&content) {
        let name = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let source = caps.get(3).map(|m| m.as_str()).unwrap_or("");

        if source.starts_with("git+") {
            let url = source.strip_prefix("git+").unwrap_or(source);
            let clean_url = url.split('#').next().unwrap_or(url).to_string();
            git_deps.insert(name.to_string(), clean_url);
        }
    }

    git_deps
}

fn serialize_report(report: &ScanReport, output_dir: &Path) -> Result<()> {
    fs::create_dir_all(output_dir)?;

    // JSON (pretty)
    let json_path = output_dir.join("scan_report.json");
    let json = serde_json::to_string_pretty(report).context("Failed to serialize to JSON")?;
    fs::write(&json_path, json).context("Failed to write JSON")?;
    println!("  JSON: {}", json_path.display());

    // JSON (compact)
    let json_compact_path = output_dir.join("scan_report.min.json");
    let json_compact = serde_json::to_string(report).context("Failed to serialize to compact JSON")?;
    fs::write(&json_compact_path, json_compact).context("Failed to write compact JSON")?;
    println!("  JSON (compact): {}", json_compact_path.display());

    // TOML
    let toml_path = output_dir.join("scan_report.toml");
    let toml = toml::to_string_pretty(report).context("Failed to serialize to TOML")?;
    fs::write(&toml_path, toml).context("Failed to write TOML")?;
    println!("  TOML: {}", toml_path.display());

    // Print summary
    println!("\n📊 Serialization Summary:");
    println!("  Total repos found: {}", report.total_unique_repos);
    println!("  Crates scanned: {}", report.crates.len());
    println!("  Successful clones: {}", report.clone_results.iter().filter(|r| r.success).count());
    println!("  Failed clones: {}", report.clone_results.iter().filter(|r| !r.success).count());

    Ok(())
}

fn main() -> Result<()> {
    let start_time = std::time::Instant::now();

    let wp = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let output_dir = std::env::args()
        .nth(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("output"));

    println!("🔍 Scanning directory: {}", wp.display());
    println!("📁 Output directory: {}", output_dir.display());

    // Initialize repo cache
    let cache_path = output_dir.join("repo_cache.json");
    let cache = std::sync::Mutex::new(RepoCache::load(&cache_path));
    println!("📦 Loaded cache with {} entries", cache.lock().unwrap().repos.len());

    // Scan directory recursively for Cargo.toml files and extract repository URLs
    println!("\n=== SCANNING FOR CARGO CRATES ===");
    let crates = scan_directory_for_cargo_repos(&wp)?;
    let crate_repos: std::collections::HashSet<String> = crates.iter()
        .filter_map(|c| c.repository.clone())
        .collect();
    println!("Found {} crates with repository URLs", crate_repos.len());

    let mirror_base = PathBuf::from("/home/mdupont/git");

    // Start with crate repos
    let mut unique_repos: std::collections::HashSet<String> = crate_repos;
    println!("Starting with {} unique repos", unique_repos.len());

    // Track all results for serialization
    let mut all_clone_results: Vec<CloneResult> = Vec::new();
    let mut all_search_results: Vec<SearchResult> = Vec::new();

    // 8 iterations of search and clone
    let num_cpus = num_cpus::get();
    println!("Using {} CPU cores for parallel operations", num_cpus);

    for iteration in 0..8 {
        println!("\n=== ITERATION {} ===", iteration + 1);
        println!("Current unique repos: {}", unique_repos.len());

        // Clone repos (sequential due to curl check + cache)
        let to_clone: Vec<String> = unique_repos.iter().cloned().collect();
        let clone_start = std::time::Instant::now();

        let clone_results: Vec<_> = to_clone.par_iter()
            .map(|repo_url| {
                let result = clone_bare_to_mirror(repo_url, &mirror_base, &cache, &cache_path);
                (result, repo_url.clone())
            })
            .collect();

        let mut newly_cloned = Vec::new();
        for (result, url) in clone_results {
            let clone_result = CloneResult {
                url: url.clone(),
                mirror_path: result.as_ref().map(|p| p.display().to_string()).unwrap_or_default(),
                success: result.is_ok(),
                error: result.as_ref().err().map(|e| e.to_string()),
            };
            all_clone_results.push(clone_result);

            match result {
                Ok(path) => {
                    println!("  OK: {}", path.display());
                    newly_cloned.push(path);
                }
                Err(e) => eprintln!("  FAILED: {}", e),
            }
        }
        println!("  Cloned {} repos in {:?}", newly_cloned.len(), clone_start.elapsed());

        // Search newly cloned repos in parallel
        let search_start = std::time::Instant::now();
        let repos_to_search: Vec<_> = newly_cloned.clone();

        let search_results: Vec<(PathBuf, Vec<String>)> = repos_to_search
            .par_iter()
            .map(|repo_path| {
                println!("  Searching: {}", repo_path.display());
                let found = search_all_git_objects_for_urls(repo_path);
                (repo_path.clone(), found)
            })
            .collect();

        let mut new_urls = Vec::new();
        for (repo_path, urls) in search_results {
            let search_result = SearchResult {
                repo_path: repo_path.display().to_string(),
                found_urls: urls.clone(),
            };
            all_search_results.push(search_result);

            for url in urls {
                if !unique_repos.contains(&url) {
                    println!("    Found: {}", url);
                    new_urls.push(url);
                }
            }
        }
        println!("  Searched repos in {:?}", search_start.elapsed());

        // Add new URLs to unique_repos for next iteration
        for url in new_urls {
            unique_repos.insert(url);
        }

        if newly_cloned.is_empty() && iteration > 0 {
            println!("No new repos to clone, stopping early");
            break;
        }
    }

    let duration = start_time.elapsed().as_secs_f64();

    println!("\n=== FINAL UNIQUE REPOS: {} ===", unique_repos.len());
    for r in &unique_repos {
        println!("  {}", r);
    }

    // Graph partitioning
    println!("\n=== PARTITIONING GRAPH ===");
    let num_partitions = (num_cpus as f64 * 2.0) as usize;

    // Get dependencies from cargo metadata
    let metadata_deps = get_cargo_metadata_deps(&wp);
    println!("Found {} dependency relationships from cargo metadata", metadata_deps.len());

    let (graph, _node_indices) = build_dependency_graph(&crates, &all_search_results, &wp);
    println!("Built graph with {} nodes and {} edges", graph.node_count(), graph.edge_count());

    let methods = vec![
        ("round_robin", partition_graph_round_robin(&graph, num_partitions)),
        ("bfs", partition_graph_by_bfs(&graph, num_partitions)),
        ("random", partition_graph_random(&graph, num_partitions)),
    ];

    for (method_name, partitions) in methods {
        println!("\n--- {} partitioning ({}) ---", method_name, num_partitions);
        let partition_report = create_partition_report(&graph, partitions, method_name);

        fs::create_dir_all(&output_dir).ok();

        println!("  Nodes per partition: {:?}", partition_report.partition_sizes);

        let partition_json_path = output_dir.join(format!("partition_{}.json", method_name));
        let json = serde_json::to_string_pretty(&partition_report).context("Failed to serialize partition")?;
        fs::write(&partition_json_path, json).context("Failed to write partition JSON")?;
        println!("  Written: {}", partition_json_path.display());

        for partition in &partition_report.partitions {
            let part_file = output_dir.join(format!("partition_{}_part{}.txt", method_name, partition.partition_id));
            let mut content = format!("# Partition {} ({} nodes, {} edges)\n\n", partition.partition_id, partition.nodes.len(), partition.edges.len());
            content.push_str("## Nodes:\n");
            for node in &partition.nodes {
                content.push_str(&format!("  - {}\n", node));
            }
            content.push_str("\n## Edges:\n");
            for (src, tgt) in &partition.edges {
                content.push_str(&format!("  {} -> {}\n", src, tgt));
            }
            fs::write(&part_file, content).ok();
        }
    }

    // Create report and serialize
    let report = ScanReport {
        scan_path: wp.display().to_string(),
        crates,
        clone_results: all_clone_results,
        search_results: all_search_results,
        total_unique_repos: unique_repos.len(),
        iterations: 8,
        cpu_cores: num_cpus,
        duration_secs: duration,
    };

    println!("\n=== SERIALIZING DATA ===");
    serialize_report(&report, &output_dir)?;

    println!("\n✅ Done! Repos cloned to {}", mirror_base.display());
    Ok(())
}
