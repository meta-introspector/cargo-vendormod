//! # Recursive Repository Scanner
//!
//! Scans directories for Cargo.toml files, extracts git URLs, clones to mirrors, 
//! and recursively processes each clone.

use anyhow::{Context, Result};
use clap::Parser;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "scanner")]
#[command(about = "Recursive git repository scanner", long_about = None)]
struct ScannerArgs {
    /// Directory to scan
    #[arg(long, default_value = ".")]
    scan_dir: PathBuf,

    /// Mirrors directory for bare clones
    #[arg(long, default_value = "~/git/host")]
    mirrors_dir: PathBuf,

    /// Maximum recursion depth
    #[arg(long, default_value = "5")]
    max_depth: usize,

    /// Maximum iterations (passes)
    #[arg(long, default_value = "8")]
    max_iterations: usize,

    /// Skip cloning, just list URLs
    #[arg(long)]
    dry_run: bool,
}

fn main() -> Result<()> {
    let args = ScannerArgs::parse();

    let mirrors_dir = args.mirrors_dir.expand_user()?;
    let scan_dir = args.scan_dir.canonicalize()?;

    println!("=== Recursive Git Repository Scanner ===");
    println!("Scan directory: {}", scan_dir.display());
    println!("Mirrors directory: {}", mirrors_dir.display());
    println!("Max depth: {}", args.max_depth);
    println!("Max iterations: {}", args.max_iterations);

    fs::create_dir_all(&mirrors_dir)?;

    let mut processed_urls: HashSet<String> = HashSet::new();
    let mut scan_dirs: Vec<PathBuf> = vec![scan_dir.clone()];
    let mut total_cloned = 0;

    for iteration in 1..=args.max_iterations {
        println!("\n=== Iteration {} of {} ===", iteration, args.max_iterations);
        
        let mut new_dirs: Vec<PathBuf> = Vec::new();
        let mut cloned_this_iteration = 0;

        for dir in &scan_dirs {
            let urls = find_git_urls_in_dir(dir, args.max_depth)?;
            
            for url in urls {
                if processed_urls.contains(&url) {
                    continue;
                }
                processed_urls.insert(url.clone());

                if args.dry_run {
                    println!("  Found: {}", url);
                    continue;
                }

                // Clone to mirrors
                match clone_to_mirrors(&url, &mirrors_dir) {
                    Ok(work_dir) => {
                        cloned_this_iteration += 1;
                        total_cloned += 1;
                        
                        // Add to next iteration's scan dirs
                        if has_cargo_toml(&work_dir) {
                            new_dirs.push(work_dir);
                        }
                    }
                    Err(e) => {
                        println!("  Failed to clone {}: {}", url, e);
                    }
                }
            }
        }

        println!("Cloned {} repos this iteration", cloned_this_iteration);

        if cloned_this_iteration == 0 {
            println!("No new repos found, stopping.");
            break;
        }

        scan_dirs = new_dirs;
    }

    println!("\n=== Summary ===");
    println!("Total unique URLs: {}", processed_urls.len());
    println!("Total cloned: {}", total_cloned);
    println!("Mirrors: {} repos", count_repos(&mirrors_dir));

    Ok(())
}

fn find_git_urls_in_dir(root: &Path, max_depth: usize) -> Result<Vec<String>> {
    let mut urls = Vec::new();
    let mut visited = HashSet::new();

    fn scan_recursive(
        dir: &Path,
        depth: usize,
        max_depth: usize,
        visited: &mut HashSet<PathBuf>,
        urls: &mut Vec<String>,
    ) {
        if depth > max_depth {
            return;
        }

        let canonical = match dir.canonicalize() {
            Ok(p) => p,
            Err(_) => return,
        };

        if !visited.insert(canonical.clone()) {
            return;
        }

        // Look for Cargo.toml
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                
                if path.is_file() && path.file_name().map(|n| n == "Cargo.toml").unwrap_or(false) {
                    if let Some(url) = extract_repo_url(&path) {
                        if url.contains("github.com") || url.contains("gitlab.com") {
                            urls.push(url);
                        }
                    }
                } else if path.is_dir() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    
                    // Skip certain directories
                    if name == ".git" || name.starts_with('.') || name == "target" {
                        continue;
                    }
                    
                    scan_recursive(&path, depth + 1, max_depth, visited, urls);
                }
            }
        }
    }

    scan_recursive(root, 0, max_depth, &mut visited, &mut urls);
    
    // Deduplicate
    urls.sort();
    urls.dedup();
    
    Ok(urls)
}

fn extract_repo_url(cargo_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(cargo_toml).ok()?;
    
    // Try toml parsing
    if let Ok(parsed) = content.parse::<toml::Value>() {
        // Try [package] repository
        if let Some(repo) = parsed.get("package")
            .and_then(|p| p.get("repository"))
            .and_then(|r| r.as_str())
        {
            return Some(repo.to_string());
        }
        
        // Try [workspace.package] repository
        if let Some(repo) = parsed.get("workspace")
            .and_then(|w| w.get("package"))
            .and_then(|p| p.get("repository"))
            .and_then(|r| r.as_str())
        {
            return Some(repo.to_string());
        }
    }
    
    // Fallback to regex
    let repo_re = regex::Regex::new(r#"repository\s*=\s*"([^"]+)""#).ok()?;
    repo_re.captures(&content)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
}

fn clone_to_mirrors(url: &str, mirrors_dir: &Path) -> Result<PathBuf> {
    // Parse URL to get owner/repo
    let owner_repo = parse_owner_repo(url)?;
    let host = get_host(url);
    
    let target_dir = mirrors_dir.join(&host).join(&owner_repo.0);
    fs::create_dir_all(&target_dir)?;
    
    let bare_path = target_dir.join(format!("{}.git", owner_repo.1));
    
    if bare_path.exists() {
        // Return work dir for existing repo
        let work_dir = bare_path.parent().unwrap().to_path_buf();
        return Ok(work_dir);
    }
    
    println!("  Cloning: {} -> {}", url, bare_path.display());
    
    let output = Command::new("git")
        .args(["clone", "--bare", url, bare_path.to_str().unwrap()])
        .output()
        .context(format!("Failed to clone {}", url))?;
    
    if !output.status.success() {
        anyhow::bail!("Git clone failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    let work_dir = bare_path.parent().unwrap().to_path_buf();
    Ok(work_dir)
}

fn parse_owner_repo(url: &str) -> Result<(String, String)> {
    // Handle various URL formats
    // https://github.com/owner/repo.git
    // https://github.com/owner/repo
    // git@github.com:owner/repo.git
    
    let url = url.trim_end_matches(".git");
    
    let parts: Vec<&str> = url.rsplit('/').collect();
    if parts.len() < 2 {
        anyhow::bail!("Invalid URL: {}", url);
    }
    
    let repo = parts[0].to_string();
    let owner = parts.get(1).unwrap_or(&"unknown").to_string();
    
    Ok((owner, repo))
}

fn get_host(url: &str) -> String {
    // Extract host from URL
    if url.contains("github.com") {
        "github.com".to_string()
    } else if url.contains("gitlab.com") {
        "gitlab.com".to_string()
    } else if url.contains("gitlab.redox-os.org") {
        "gitlab.redox-os.org".to_string()
    } else {
        // Extract from URL
        url.split("://")
            .nth(1)
            .unwrap_or("unknown")
            .split('/')
            .next()
            .unwrap_or("unknown")
            .to_string()
    }
}

fn has_cargo_toml(dir: &Path) -> bool {
    if dir.join("Cargo.toml").exists() {
        return true;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if has_cargo_toml(&path) {
                    return true;
                }
            }
        }
    }
    
    false
}

fn count_repos(mirrors_dir: &Path) -> usize {
    std::fs::read_dir(mirrors_dir)
        .map(|entries| {
            entries.flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| {
                    std::fs::read_dir(e.path())
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