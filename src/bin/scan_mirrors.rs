//! Scan existing mirrors for transitive git dependencies

use anyhow::{Context, Result};
use clap::Parser;
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "scan-mirrors")]
#[command(about = "Scan mirrors for transitive git dependencies")]
struct Args {
    /// Mirrors directory
    #[arg(long, default_value = "~/git/host")]
    mirrors_dir: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mirrors_dir = args.mirrors_dir.expand_user()?;

    println!("=== Scanning mirrors for transitive git deps ===");
    println!("Mirrors: {}", mirrors_dir.display());

    let mut all_urls = Vec::new();

    // Scan each repo in mirrors
    for entry in walkdir::WalkDir::new(&mirrors_dir)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().ends_with(".git"))
    {
        let repo_dir = entry.path().parent().unwrap().to_path_buf();
        
        // Check for Cargo.lock
        let lock_path = repo_dir.join("Cargo.lock");
        if !lock_path.exists() {
            // Try to fetch the default branch first
            continue;
        }

        // Parse Cargo.lock for git dependencies
        if let Ok(content) = fs::read_to_string(&lock_path) {
            // Look for git URLs
            let url_re = regex::Regex::new(r"git\+(https?://[^#]+)#").unwrap();
            for cap in url_re.captures_iter(&content) {
                let url = cap.get(1).unwrap().as_str();
                if url.contains("github.com") || url.contains("gitlab") {
                    all_urls.push(url.to_string());
                }
            }
        }
    }

    // Deduplicate
    all_urls.sort();
    all_urls.dedup();

    println!("\nFound {} unique transitive git URLs:", all_urls.len());
    for url in &all_urls {
        println!("  {}", url);
    }

    Ok(())
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