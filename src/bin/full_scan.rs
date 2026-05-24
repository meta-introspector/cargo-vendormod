//! Comprehensive recursive scanner that clones all git repos and creates newroot structure

use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "full-scan")]
#[command(about = "Full recursive scan - clone all git deps and create newroot")]
struct Args {
    /// Root directory to scan
    #[arg(long, default_value = ".")]
    scan_root: PathBuf,

    /// Mirrors directory for bare clones
    #[arg(long, default_value = "~/git")]
    mirrors_dir: PathBuf,

    /// Newroot directory to create with submodules
    #[arg(long, default_value = "./newroot")]
    newroot_dir: PathBuf,

    /// Maximum recursion depth
    #[arg(long, default_value = "5")]
    max_depth: usize,

    /// Maximum iterations
    #[arg(long, default_value = "8")]
    max_iterations: usize,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mirrors_dir = args.mirrors_dir.expand_user()?;
    let scan_root = args.scan_root.canonicalize()?;
    let newroot_dir = args.newroot_dir;

    println!("=== Full Recursive Git Scanner ===");
    println!("Scan root: {}", scan_root.display());
    println!("Mirrors: {}", mirrors_dir.display());
    println!("Newroot: {}", newroot_dir.display());
    println!("Max depth: {}, Max iterations: {}", args.max_depth, args.max_iterations);

    // Create directories
    fs::create_dir_all(&mirrors_dir)?;
    fs::create_dir_all(&newroot_dir)?;

    let mut processed_urls: HashSet<String> = HashSet::new();
    let mut scan_dirs: Vec<PathBuf> = vec![scan_root.clone()];
    let mut cloned_repos: Vec<(String, PathBuf)> = Vec::new();
    let mut total_cloned = 0;

    for iteration in 1..=args.max_iterations {
        println!("\n=== Iteration {} of {} ===", iteration, args.max_iterations);
        
        let mut new_dirs: Vec<PathBuf> = Vec::new();
        let mut cloned_this_iter = 0;

        for dir in &scan_dirs {
            // Find all git URLs in this directory
            let urls = find_all_git_urls(dir, args.max_depth)?;
            
            for url in urls {
                // Normalize URL
                let normalized = normalize_url(&url);
                if processed_urls.contains(&normalized) {
                    continue;
                }
                processed_urls.insert(normalized.clone());

                // Clone to mirrors
                match clone_to_mirrors(&normalized, &mirrors_dir) {
                    Ok(bare_path) => {
                        cloned_this_iter += 1;
                        total_cloned += 1;
                        cloned_repos.push((normalized.clone(), bare_path.clone()));
                        println!("  Cloned: {}", normalized);
                        
                        // Add to newroot as submodule
                        if let Err(e) = add_submodule(&normalized, &bare_path, &newroot_dir) {
                            println!("  Warning: could not add submodule: {}", e);
                        }

                        // Check if this cloned repo has more Cargo.toml files
                        // We need to check out a working dir first
                        if let Ok(work_dir) = checkout_for_scan(&bare_path) {
                            if has_cargo_toml(&work_dir) {
                                new_dirs.push(work_dir);
                            }
                        }
                    }
                    Err(e) => {
                        println!("  Failed: {} - {}", normalized, e);
                    }
                }
            }
        }

        println!("Cloned {} repos this iteration (total: {})", cloned_this_iter, total_cloned);

        if cloned_this_iter == 0 {
            println!("No new repos found, stopping.");
            break;
        }

        scan_dirs = new_dirs;
    }

    println!("\n=== Summary ===");
    println!("Total unique URLs found: {}", processed_urls.len());
    println!("Total cloned: {}", total_cloned);
    println!("Mirrors: {} repos", count_repos(&mirrors_dir));

    // Generate summary report
    let report_path = PathBuf::from("scan_report.json");
    let report = serde_json::json!({
        "total_urls": processed_urls.len(),
        "total_cloned": total_cloned,
        "mirrors_dir": mirrors_dir.display().to_string(),
        "newroot_dir": newroot_dir.display().to_string(),
        "repos": cloned_repos.iter().map(|(url, path)| {
            serde_json::json!({
                "url": url,
                "bare_path": path.display().to_string()
            })
        }).collect::<Vec<_>>()
    });
    fs::write(&report_path, serde_json::to_string_pretty(&report)?)?;
    println!("Report saved to: {}", report_path.display());

    Ok(())
}

fn find_all_git_urls(root: &Path, max_depth: usize) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    let mut visited = HashSet::new();

    fn scan_dir(dir: &Path, depth: usize, max_depth: usize, visited: &mut HashSet<PathBuf>, urls: &mut Vec<String>) {
        if depth > max_depth { return; }

        let canonical = match dir.canonicalize() { Ok(p) => p, Err(_) => return };
        if !visited.insert(canonical.clone()) { return; }

        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

                if path.is_file() && name == "Cargo.toml" {
                    // Extract repository from package
                    if let Some(url) = extract_repo_url(&path) {
                        if url.contains("github.com") || url.contains("gitlab") {
                            urls.push(url);
                        }
                    }
                    // Extract git dependencies
                    if let Ok(dep_urls) = extract_git_deps(&path) {
                        for url in dep_urls {
                            if url.contains("github.com") || url.contains("gitlab") || url.contains("bitbucket") {
                                urls.push(url);
                            }
                        }
                    }
                } else if path.is_dir() {
                    if name == ".git" || name.starts_with('.') || name == "target" || name == "node_modules" {
                        continue;
                    }
                    scan_dir(&path, depth + 1, max_depth, visited, urls);
                }
            }
        }
    }

    scan_dir(root, 0, max_depth, &mut visited, &mut urls);
    urls.sort();
    urls.dedup();
    Ok(urls)
}

fn extract_repo_url(cargo_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml).ok()?;

    // Try toml parsing
    if let Ok(parsed) = content.parse::<toml::Value>() {
        if let Some(repo) = parsed.get("package")
            .and_then(|p| p.get("repository"))
            .and_then(|r| r.as_str())
        {
            return Some(repo.to_string());
        }
    }

    // Fallback to regex
    let re = regex::Regex::new(r#"repository\s*=\s*"([^"]+)"#).ok()?;
    re.captures(&content).and_then(|c| c.get(1)).map(|m| m.as_str().to_string())
}

fn extract_git_deps(cargo_toml: &Path) -> Result<Vec<String>> {
    let content = fs::read_to_string(cargo_toml)?;
    let parsed: toml::Value = content.parse().context("Failed to parse Cargo.toml")?;

    let mut urls = Vec::new();

    // Extract from [dependencies] table
    if let Some(deps) = parsed.get("dependencies").and_then(|d| d.as_table()) {
        extract_urls_from_table(deps, &mut urls);
    }

    // Extract from [dev-dependencies] table
    if let Some(deps) = parsed.get("dev-dependencies").and_then(|d| d.as_table()) {
        extract_urls_from_table(deps, &mut urls);
    }

    // Extract from [build-dependencies] table
    if let Some(deps) = parsed.get("build-dependencies").and_then(|d| d.as_table()) {
        extract_urls_from_table(deps, &mut urls);
    }

    Ok(urls)
}

fn extract_urls_from_table(table: &toml::map::Map<String, toml::Value>, urls: &mut Vec<String>) {
    let git_url_re = regex::Regex::new(r#"https?://[a-zA-Z0-9._/-]+/([a-zA-Z0-9_-]+)/([a-zA-Z0-9._-]+)"#).unwrap();

    for (name, value) in table {
        // Check for git = "url" format
        if let Some(table) = value.as_table() {
            if let Some(git_url) = table.get("git").and_then(|v| v.as_str()) {
                if git_url.contains("github.com") || git_url.contains("gitlab") || git_url.contains("bitbucket") {
                    // Extract clean URL without .git suffix
                    let clean_url = git_url.trim_end_matches(".git");
                    if !urls.contains(&clean_url.to_string()) {
                        urls.push(clean_url.to_string());
                    }
                }
            }
        }
        // Also check inline tables: name = { git = "url", ... }
        // And string values that might be git URLs
        if let Some(s) = value.as_str() {
            if s.contains("github.com") || s.contains("gitlab") || s.contains("bitbucket") {
                for cap in git_url_re.find_iter(s) {
                    let url = cap.as_str().trim_end_matches(".git").to_string();
                    if !urls.contains(&url) {
                        urls.push(url);
                    }
                }
            }
        }
    }
}

fn normalize_url(url: &str) -> String {
    let url = url.trim_end_matches(".git");
    let url = url.trim_end_matches("/");
    // Remove tree/ branch paths
    if let Some(pos) = url.find("/tree/") {
        return url[..pos].to_string();
    }
    url.to_string()
}

fn clone_to_mirrors(url: &str, mirrors_dir: &Path) -> Result<PathBuf> {
    let (host, owner, repo) = parse_url(url)?;
    
    let target_dir = mirrors_dir.join(&host).join(&owner);
    fs::create_dir_all(&target_dir)?;
    
    let bare_path = target_dir.join(format!("{}.git", repo));
    
    if bare_path.exists() {
        return Ok(bare_path);
    }
    
    let output = Command::new("git")
        .args(["clone", "--bare", url, bare_path.to_str().unwrap()])
        .output()
        .context(format!("Failed to clone {}", url))?;
    
    if !output.status.success() {
        anyhow::bail!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    Ok(bare_path)
}

fn parse_url(url: &str) -> Result<(String, String, String)> {
    let url = url.trim_end_matches(".git");
    let parts: Vec<&str> = url.rsplit('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid URL: {}", url);
    }
    
    let repo = parts[0].to_string();
    let owner = parts.get(1).unwrap_or(&"unknown").to_string();
    
    let host = if url.contains("github.com") {
        "github.com"
    } else if url.contains("gitlab.com") {
        "gitlab.com"
    } else if url.contains("gitlab.redox-os.org") {
        "gitlab.redox-os.org"
    } else {
        url.split("://").nth(1).unwrap_or("unknown").split('/').next().unwrap_or("unknown")
    };
    
    Ok((host.to_string(), owner, repo))
}

fn add_submodule(url: &str, bare_path: &Path, newroot_dir: &Path) -> Result<()> {
    let (_, owner, repo) = parse_url(url)?;
    
    let submodule_path = newroot_dir.join(&repo);
    
    // Remove if exists
    if submodule_path.exists() {
        return Ok(());
    }
    
    // Add submodule
    let output = Command::new("git")
        .args(["submodule", "add", bare_path.to_str().unwrap(), submodule_path.to_str().unwrap()])
        .current_dir(newroot_dir)
        .output()
        .context("Failed to add submodule")?;
    
    if !output.status.success() {
        // Try alternative: just create a symlink or clone
        println!("  Submodule add failed, cloning directly: {}", repo);
        let output = Command::new("git")
            .args(["clone", bare_path.to_str().unwrap(), submodule_path.to_str().unwrap()])
            .output();
        if output.is_err() {
            return Err(anyhow::anyhow!("Failed to clone"));
        }
    }
    
    Ok(())
}

fn checkout_for_scan(bare_path: &Path) -> Result<PathBuf> {
    // Create a temp directory to checkout
    let temp_dir = std::env::temp_dir().join(format!("scan_{}", rand_string()));
    fs::create_dir_all(&temp_dir)?;
    
    let output = Command::new("git")
        .args(["clone", bare_path.to_str().unwrap(), temp_dir.to_str().unwrap()])
        .output()
        .context("Failed to checkout for scan")?;
    
    if !output.status.success() {
        fs::remove_dir_all(&temp_dir).ok();
        anyhow::bail!("Checkout failed");
    }
    
    Ok(temp_dir)
}

fn rand_string() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let seed = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    format!("{:x}", seed)
}

fn has_cargo_toml(dir: &Path) -> bool {
    if dir.join("Cargo.toml").exists() {
        return true;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                let name = entry.file_name();
                let name_str = name.to_str().unwrap_or("");
                if name_str != ".git" && !name_str.starts_with('.') {
                    if has_cargo_toml(&entry.path()) {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn count_repos(mirrors_dir: &Path) -> usize {
    fs::read_dir(mirrors_dir)
        .map(|entries| {
            entries.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| {
                    fs::read_dir(e.path())
                        .map(|sub| sub.flatten().filter(|s| s.path().ends_with(".git")).count())
                        .unwrap_or(0)
                })
                .sum()
        })
        .unwrap_or(0)
}

trait PathExt {
    fn expand_user(self) -> Result<PathBuf>;
}

impl PathExt for PathBuf {
    fn expand_user(self) -> Result<PathBuf> {
        if let Some(s) = self.to_str() {
            if s.starts_with("~") {
                let home = dirs::home_dir().context("No home directory")?;
                return Ok(home.join(s.trim_start_matches("~").trim_start_matches('/')));
            }
        }
        Ok(self)
    }
}