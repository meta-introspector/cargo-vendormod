//! Git Submodule Scanner
//!
//! Scans directories for git repositories and submodules.
//! Integrates with language-specific scanners (CBOR, etc.)

use anyhow::Result;
use clap::Parser;
use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command as SysCommand;
use walkdir::WalkDir;

// ============================================================
// Data Structures
// ============================================================

/// Information about a git repository
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitRepo {
    pub path: PathBuf,
    pub url: Option<String>,
    pub branch: Option<String>,
    pub commit: Option<String>,
    pub is_submodule: bool,
    pub submodules: Vec<GitRepo>,
    pub languages: HashSet<String>,
}

/// Scanner configuration for different file types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannerConfig {
    pub name: String,
    /// File extensions to scan
    pub extensions: Vec<String>,
    /// Command to run for scanning
    pub command: String,
    /// Output directory
    pub output_dir: String,
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub scanners: Vec<ScannerConfig>,
    pub output_dir: String,
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            scanners: vec![
                ScannerConfig {
                    name: "cbor_scanner".to_string(),
                    extensions: vec!["cbor".to_string()],
                    command: "cargo run --bin cbor_scanner -- extract -i {dir} -o {output}/cbor".to_string(),
                    output_dir: "cbor".to_string(),
                },
            ],
            output_dir: "scanner_output".to_string(),
        }
    }
}

// ============================================================
// Git Repository Discovery
// ============================================================

/// Discover all git repositories in a directory tree
pub fn discover_git_repos(root: &Path) -> Result<Vec<GitRepo>> {
    let mut repos = Vec::new();
    
    for entry in WalkDir::new(root)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Check for .git directory (indicates a git repo)
        if path.is_dir() && path.join(".git").exists() {
            let repo = scan_git_repo(path)?;
            repos.push(repo);
        }
    }
    
    Ok(repos)
}

/// Scan a single git repository for metadata
fn scan_git_repo(path: &Path) -> Result<GitRepo> {
    let mut repo = GitRepo {
        path: path.to_path_buf(),
        url: None,
        branch: None,
        commit: None,
        is_submodule: false,
        submodules: Vec::new(),
        languages: HashSet::new(),
    };
    
    // Read git config for URL
    let config_path = path.join(".git").join("config");
    if let Ok(config) = fs::read_to_string(&config_path) {
        for line in config.lines() {
            if line.trim().starts_with("url = ") {
                repo.url = Some(line.trim().trim_start_matches("url = ").to_string());
            }
        }
    }
    
    // Get current branch and commit
    if let Ok(branch) = get_git_branch(path) {
        repo.branch = Some(branch);
    }
    if let Ok(commit) = get_git_commit(path) {
        repo.commit = Some(commit);
    }
    
    // Check if this is a submodule (has .git file that's a reference)
    let git_file = path.join(".git");
    if git_file.is_file() {
        if let Ok(content) = fs::read_to_string(&git_file) {
            if content.contains("gitdir: ") {
                repo.is_submodule = true;
            }
        }
    }
    
    // Find submodules
    let submodule_path = path.join(".gitmodules");
    if submodule_path.exists() {
        repo.submodules = find_submodules(path)?;
    }
    
    // Detect languages based on file extensions
    repo.languages = detect_languages(path)?;
    
    Ok(repo)
}

/// Get current git branch
fn get_git_branch(path: &Path) -> Result<String> {
    let output = SysCommand::new("git")
        .arg("-C")
        .arg(path)
        .arg("branch")
        .arg("--show-current")
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Get current git commit
fn get_git_commit(path: &Path) -> Result<String> {
    let output = SysCommand::new("git")
        .arg("-C")
        .arg(path)
        .arg("rev-parse")
        .arg("HEAD")
        .output()?;
    
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Ok("unknown".to_string())
    }
}

/// Find git submodules
fn find_submodules(path: &Path) -> Result<Vec<GitRepo>> {
    let mut submodules = Vec::new();
    
    // Read .gitmodules file
    let gitmodules_path = path.join(".gitmodules");
    if !gitmodules_path.exists() {
        return Ok(submodules);
    }
    
    let content = fs::read_to_string(&gitmodules_path)?;
    
    // Parse .gitmodules to find submodule paths
    let mut current_path = None;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("path = ") {
            current_path = Some(path.join(trimmed.trim_start_matches("path = ")));
        } else if trimmed.starts_with("url = ") && current_path.is_some() {
            let submodule_path = current_path.take().unwrap();
            if submodule_path.exists() {
                submodules.push(scan_git_repo(&submodule_path)?);
            }
        }
    }
    
    Ok(submodules)
}

/// Detect programming languages in a repository
fn detect_languages(path: &Path) -> Result<HashSet<String>> {
    let mut languages = HashSet::new();
    
    // Common file extensions for languages
    let language_extensions: &[(&str, &str)] = &[
        ("rs", "Rust"),
        ("py", "Python"),
        ("go", "Go"),
        ("js", "JavaScript"),
        ("ts", "TypeScript"),
        ("java", "Java"),
        ("cpp", "C++"),
        ("c", "C"),
        ("h", "C/C++"),
        ("rb", "Ruby"),
        ("php", "PHP"),
        ("swift", "Swift"),
        ("kt", "Kotlin"),
        ("scala", "Scala"),
        ("hs", "Haskell"),
        ("ml", "OCaml"),
        ("erl", "Erlang"),
        ("exs", "Elixir"),
        ("clj", "Clojure"),
        ("lisp", "Lisp"),
        ("sh", "Shell"),
        ("zsh", "Zsh"),
        ("toml", "TOML"),
        ("yaml", "YAML"),
        ("yml", "YAML"),
        ("json", "JSON"),
        ("cbor", "CBOR"),
    ];
    
    for (ext, lang) in language_extensions {
        for entry in WalkDir::new(path)
            .follow_links(false)
            .max_depth(3)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().extension().map(|s| s.to_string_lossy() == *ext).unwrap_or(false) {
                languages.insert(lang.to_string());
                break; // Found at least one file of this type
            }
        }
    }
    
    // Check for Cargo.toml (Rust)
    if path.join("Cargo.toml").exists() {
        languages.insert("Rust".to_string());
    }
    
    // Check for package.json (Node.js)
    if path.join("package.json").exists() {
        languages.insert("JavaScript/Node.js".to_string());
    }
    
    // Check for go.mod (Go)
    if path.join("go.mod").exists() {
        languages.insert("Go".to_string());
    }
    
    // Check for .cbor files
    if path.join(".cbor").exists() || 
       WalkDir::new(path).into_iter().any(|e| 
           e.ok().map(|e| e.path().extension().map(|s| s == "cbor").unwrap_or(false)).unwrap_or(false)
       ) {
        languages.insert("CBOR".to_string());
    }
    
    Ok(languages)
}

// ============================================================
// Scanner Integration
// ============================================================

/// Run a scanner on files in a directory
pub fn run_scanner(
    dir: &Path,
    config: &ScannerConfig,
    output_base: &Path,
) -> Result<Vec<PathBuf>> {
    let output_dir = output_base.join(&config.output_dir);
    fs::create_dir_all(&output_dir)?;
    
    // Find all files with matching extensions
    let mut files = Vec::new();
    for entry in WalkDir::new(dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if config.extensions.contains(&ext.to_string_lossy().to_string()) {
                    files.push(path.to_path_buf());
                }
            }
        }
    }
    
    if files.is_empty() {
        log::info!("No files found for scanner: {}", config.name);
        return Ok(files);
    }
    
    log::info!("Found {} files for {} scanner", files.len(), config.name);
    
    // Execute the scanner command
    let cmd = config.command
        .replace("{dir}", dir.to_string_lossy().as_ref())
        .replace("{output}", output_dir.to_string_lossy().as_ref());
    
    log::info!("Running: {}", cmd);
    
    let output = if cfg!(target_os = "windows") {
        SysCommand::new("cmd")
            .args(&["/C", &cmd])
            .current_dir(dir)
            .output()?
    } else {
        SysCommand::new("sh")
            .arg("-c")
            .arg(&cmd)
            .current_dir(dir)
            .output()?
    };
    
    if !output.status.success() {
        log::error!("Scanner failed: {}", String::from_utf8_lossy(&output.stderr));
    } else {
        log::info!("Scanner output: {}", String::from_utf8_lossy(&output.stdout));
    }
    
    Ok(files)
}

/// Scan all repositories with all configured scanners
pub fn scan_all(
    repos: &[GitRepo],
    config: &WorkflowConfig,
    output_base: &Path,
) -> Result<HashMap<String, Vec<PathBuf>>> {
    let mut results = HashMap::new();
    
    fs::create_dir_all(output_base)?;
    
    for repo in repos {
        log::info!("Scanning repository: {}", repo.path.display());
        
        for scanner in &config.scanners {
            let scanned = run_scanner(&repo.path, scanner, output_base)?;
            results.insert(
                format!("{}:{}", repo.path.display(), scanner.name),
                scanned,
            );
        }
    }
    
    Ok(results)
}

// ============================================================
// CLI Interface
// ============================================================

#[derive(Parser, Debug)]
#[command(name = "git_scanner")]
#[command(author = "dasl")]
#[command(version = "0.1.0")]
#[command(about = "Scan git repositories and submodules for language-specific files")]
struct Args {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Discover all git repositories in a directory
    Discover {
        /// Root directory to scan
        #[arg(short, long, default_value = ".")]
        root: PathBuf,
        
        /// Output JSON file
        #[arg(short, long, default_value = "repos.json")]
        output: PathBuf,
        
        /// Include submodules
        #[arg(short = 'R', long)]
        recursive: bool,
    },
    
    /// Scan repositories with configured scanners
    Scan {
        /// Root directory or repos JSON file
        #[arg(short, long, default_value = ".")]
        input: PathBuf,
        
        /// Config file (optional)
        #[arg(short, long, default_value = "scanner_config.json")]
        config: PathBuf,
        
        /// Output directory
        #[arg(short, long, default_value = "scanner_output")]
        output: PathBuf,
    },
    
    /// List all languages found in repositories
    ListLanguages {
        /// Root directory or repos JSON file
        #[arg(short, long, default_value = ".")]
        input: PathBuf,
        
        /// Output JSON file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    env_logger::Builder::from_default_env()
        .format_timestamp(None)
        .init();
    
    let args = Args::parse();
    
    match args.command {
        Command::Discover { root, output, recursive } => {
            let repos = if recursive {
                // Discover repos recursively, including submodules
                let all_repos = discover_git_repos(&root)?;
                
                // Expand submodules
                let mut expanded = Vec::new();
                for repo in &all_repos {
                    expanded.push(repo.clone());
                    expanded.extend(repo.submodules.clone());
                }
                expanded
            } else {
                discover_git_repos(&root)?
            };
            
            let json = serde_json::to_string_pretty(&repos)?;
            fs::write(&output, json)?;
            
            println!("Found {} git repositories", repos.len());
            for repo in &repos {
                println!("  {}: {} ({:?})", 
                    repo.path.display(),
                    repo.url.as_deref().unwrap_or("unknown"),
                    repo.languages);
            }
            
            println!("Saved to {}", output.display());
        }
        
        Command::Scan { input, config, output } => {
            fs::create_dir_all(&output)?;
            
            // Load repos
            let repos: Vec<GitRepo> = if input.is_file() && input.extension().map(|s| s == "json").unwrap_or(false) {
                let content = fs::read_to_string(&input)?;
                serde_json::from_str(&content)?
            } else {
                discover_git_repos(&input)?
            };
            
            // Load config
            let config: WorkflowConfig = if config.exists() {
                let content = fs::read_to_string(&config)?;
                serde_json::from_str(&content).unwrap_or_default()
            } else {
                WorkflowConfig::default()
            };
            
            scan_all(&repos, &config, &output)?;
            
            println!("Scan complete. Output in: {}", output.display());
        }
        
        Command::ListLanguages { input, output } => {
            let repos: Vec<GitRepo> = if input.is_file() && input.extension().map(|s| s == "json").unwrap_or(false) {
                let content = fs::read_to_string(&input)?;
                serde_json::from_str(&content)?
            } else {
                discover_git_repos(&input)?
            };
            
            let mut all_languages = HashSet::new();
            for repo in &repos {
                all_languages.extend(&repo.languages);
            }
            
            let mut languages: Vec<_> = all_languages.into_iter().collect();
            languages.sort();
            
            println!("Languages found:");
            for lang in &languages {
                println!("  - {}", lang);
            }
            
            if let Some(out_path) = output {
                let json = serde_json::to_string_pretty(&languages)?;
                fs::write(&out_path, json)?;
                println!("Saved to {}", out_path.display());
            }
        }
    }
    
    Ok(())
}
