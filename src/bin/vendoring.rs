//! # Vendoring Binary
//!
//! Handles git submodule operations - the core vendoring functionality.

use anyhow::{Context, Result};
use cargo_vendormod::config::Config;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::Command;
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "vendoring")]
#[command(about = "Git submodule vendoring operations", long_about = None)]
struct VendoringArgs {
    /// Path to vendormod config file
    #[arg(long)]
    config: Option<PathBuf>,

    /// Source repository to discover submodules from
    #[arg(long, default_value = ".")]
    source_repo: PathBuf,

    /// Target submodules directory
    #[arg(long)]
    submodules_path: Option<PathBuf>,

    /// Target mirrors directory
    #[arg(long)]
    mirrors_path: Option<PathBuf>,

    /// Target branch for submodules
    #[arg(long)]
    target_branch: Option<String>,

    /// Create version branches automatically
    #[arg(long)]
    create_version_branches: Option<bool>,

    /// Include dev dependencies
    #[arg(long, default_value = "true")]
    include_dev: bool,

    /// Include build dependencies
    #[arg(long, default_value = "true")]
    include_build: bool,

    /// Include optional dependencies
    #[arg(long, default_value = "true")]
    include_optional: bool,

    /// Dry run mode
    #[arg(long)]
    dry_run: bool,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,

    #[command(subcommand)]
    command: Option<VendoringCommands>,
}

#[derive(Subcommand, Debug)]
enum VendoringCommands {
    /// Initialize vendoring from source repository
    Init,
    /// Fetch from all upstream remotes
    FetchUpstream,
    /// Rebase submodules onto upstream
    Rebase,
    /// Fetch tags and create version branches
    Releases,
    /// Show submodule status
    Status,
    /// Full sync: fetch + rebase + patch
    Sync,
    /// Generate .cargo/config.toml patches
    Patch,
}

fn main() -> Result<()> {
    let args = VendoringArgs::parse();
    let config = Config::load(args.config.as_deref())?;

    let root_dir = std::env::current_dir()?;

    match args.command {
        Some(VendoringCommands::Init) => cmd_init(&args, &config, &root_dir)?,
        Some(VendoringCommands::FetchUpstream) => cmd_fetch_upstream(&args, &config, &root_dir)?,
        Some(VendoringCommands::Rebase) => cmd_rebase(&args, &config, &root_dir)?,
        Some(VendoringCommands::Releases) => cmd_releases(&args, &config, &root_dir)?,
        Some(VendoringCommands::Status) => cmd_status(&args, &config, &root_dir)?,
        Some(VendoringCommands::Sync) => cmd_sync(&args, &config, &root_dir)?,
        Some(VendoringCommands::Patch) => cmd_patch(&args, &config, &root_dir)?,
        None => {
            // Default to init
            cmd_init(&args, &config, &root_dir)?;
        }
    }

    Ok(())
}

fn cmd_init(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("=== Initializing vendoring ===");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));
    let mirrors_path = args.mirrors_path.clone()
        .unwrap_or_else(|| config.mirrors_dir.clone());

    println!("Submodules: {}", submodules_path.display());
    println!("Mirrors: {}", mirrors_path.display());

    if !args.dry_run {
        std::fs::create_dir_all(&submodules_path)?;
        std::fs::create_dir_all(&mirrors_path)?;
    }

    // Discover submodules from source
    

    println!("Discovering submodules from {}", args.source_repo.display());
    let submodules = cargo_vendormod::submodule_discovery::discover_submodules(&args.source_repo)?;
    println!("Found {} submodules", submodules.len());

    for sm in &submodules {
        println!("  - {} -> {}", sm.path.display(), sm.url);
    }

    if !args.dry_run {
        let git_exe = cargo_vendormod::submodule_discovery::get_git_executable();
        cargo_vendormod::submodule_discovery::clone_submodules_to_target(submodules, &submodules_path, &git_exe)?;
    }

    println!("=== Vendoring initialized ===");
    Ok(())
}

fn cmd_fetch_upstream(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Fetching upstream from all mirrors...");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));
    let mirrors_path = args.mirrors_path.clone()
        .unwrap_or_else(|| config.mirrors_dir.clone());

    let submodules = find_submodules(&submodules_path);
    let threads = config.default_threads;

    println!("Found {} submodules, using {} threads", submodules.len(), threads);

    if args.dry_run {
        println!("[DRY RUN] Would fetch upstream for all repos");
        return Ok(());
    }

    let git_exe = config.git_path.clone();
    submodules.par_iter().for_each(|submodule| {
        let bare_path = mirrors_path.join(&submodule.owner).join(format!("{}.git", &submodule.name));
        if !bare_path.exists() {
            eprintln!("Bare repo not found: {}", bare_path.display());
            return;
        }

        let output = Command::new(&git_exe)
            .arg("--git-dir")
            .arg(&bare_path)
            .arg("fetch")
            .arg("upstream")
            .arg("--prune")
            .arg("--tags")
            .output();

        match output {
            Ok(out) if !out.status.success() => {
                eprintln!("Failed to fetch {}: {}", submodule.name, String::from_utf8_lossy(&out.stderr));
            }
            Ok(_) if args.verbose => {
                println!("Fetched upstream for {}", submodule.name);
            }
            _ => {}
        }
    });

    println!("Upstream fetch complete.");
    Ok(())
}

fn cmd_rebase(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Rebasing submodules onto upstream...");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));
    let target_branch = args.target_branch.clone()
        .unwrap_or_else(|| config.target_branch.clone());

    let submodules = find_submodules(&submodules_path);

    if args.dry_run {
        println!("[DRY RUN] Would rebase {} submodules", submodules.len());
        return Ok(());
    }

    let git_exe = config.git_path.clone();
    submodules.par_iter().for_each(|submodule| {
        let sub_path = submodules_path.join(&submodule.name);
        if !sub_path.exists() {
            eprintln!("Submodule not found: {}", sub_path.display());
            return;
        }

        // Fetch latest
        let _ = Command::new(&git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("fetch")
            .arg("upstream")
            .output();

        // Rebase
        let output = Command::new(&git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("rebase")
            .arg(&format!("upstream/{}", target_branch))
            .output();

        match output {
            Ok(out) if !out.status.success() => {
                eprintln!("Rebase failed for {}: {}", submodule.name, String::from_utf8_lossy(&out.stderr));
                let _ = Command::new(&git_exe)
                    .arg("-C")
                    .arg(&sub_path)
                    .arg("rebase")
                    .arg("--abort")
                    .output();
            }
            Ok(_) if args.verbose => {
                println!("Rebased {} onto upstream/{}", submodule.name, target_branch);
            }
            _ => {}
        }
    });

    println!("Rebase complete.");
    Ok(())
}

fn cmd_releases(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Fetching release tags and creating version branches...");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));
    let mirrors_path = args.mirrors_path.clone()
        .unwrap_or_else(|| config.mirrors_dir.clone());
    let create_branches = args.create_version_branches.unwrap_or(config.create_version_branches);

    let submodules = find_submodules(&submodules_path);

    if !create_branches {
        println!("Version branch creation is disabled");
        return Ok(());
    }

    if args.dry_run {
        println!("[DRY RUN] Would create version branches for {} submodules", submodules.len());
        return Ok(());
    }

    let git_exe = config.git_path.clone();
    submodules.par_iter().for_each(|submodule| {
        let bare_path = mirrors_path.join(&submodule.owner).join(format!("{}.git", &submodule.name));
        if !bare_path.exists() {
            return;
        }

        // Fetch tags
        let _ = Command::new(&git_exe)
            .arg("--git-dir")
            .arg(&bare_path)
            .arg("fetch")
            .arg("upstream")
            .arg("--tags")
            .output();

        // Create version branch
        let version_branch = config.version_branch(&submodule.version);
        let _ = Command::new(&git_exe)
            .arg("--git-dir")
            .arg(&bare_path)
            .arg("branch")
            .arg("--force")
            .arg(&version_branch)
            .arg(&format!("upstream/{}", submodule.version))
            .output();
    });

    println!("Release processing complete.");
    Ok(())
}

fn cmd_status(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Submodule status:");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));

    let submodules = find_submodules(&submodules_path);
    let git_exe = config.git_path.clone();

    submodules.par_iter().for_each(|submodule| {
        let sub_path = submodules_path.join(&submodule.name);
        if !sub_path.exists() {
            println!("{}: NOT FOUND", submodule.name);
            return;
        }

        let current = Command::new(&git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "N/A".to_string());

        let upstream = Command::new(&git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("rev-parse")
            .arg("upstream/main")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "N/A".to_string());

        let status = if current == upstream && current != "N/A" {
            "✓ up-to-date"
        } else {
            "✗ diverged"
        };

        println!("{}: {} (current: {}, upstream: {})",
            submodule.name, status,
            current.chars().take(8).collect::<String>(),
            upstream.chars().take(8).collect::<String>());
    });

    Ok(())
}

fn cmd_sync(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Running full sync: fetch-upstream → rebase → patch");

    cmd_fetch_upstream(args, config, root_dir)?;
    cmd_rebase(args, config, root_dir)?;
    cmd_patch(args, config, root_dir)?;

    println!("Full sync complete.");
    Ok(())
}

fn cmd_patch(args: &VendoringArgs, config: &Config, root_dir: &PathBuf) -> Result<()> {
    println!("Generating cargo patches...");

    let submodules_path = args.submodules_path.clone()
        .unwrap_or_else(|| root_dir.join(&config.submodules_dir));

    let cargo_config_dir = root_dir.join(".cargo");
    std::fs::create_dir_all(&cargo_config_dir)?;

    let cargo_config_path = cargo_config_dir.join("config.toml");
    let mut config_doc = if cargo_config_path.exists() {
        let content = std::fs::read_to_string(&cargo_config_path)?;
        content.parse::<toml_edit::DocumentMut>()?
    } else {
        toml_edit::DocumentMut::new()
    };

    let patch = config_doc
        .entry("patch")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("Failed to get patch table")?;

    let patch_crates_io = patch
        .entry("crates-io")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("Failed to get crates-io patch table")?;

    let mut count = 0;
    if let Ok(entries) = std::fs::read_dir(&submodules_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let relative = pathdiff::diff_paths(&path, root_dir)
                        .unwrap_or_else(|| path.clone());
                    patch_crates_io.insert(dir_name, toml_edit::value(relative.to_string_lossy().to_string()));
                    count += 1;
                }
            }
        }
    }

    std::fs::write(&cargo_config_path, config_doc.to_string())?;
    println!("Updated .cargo/config.toml with {} patches", count);
    Ok(())
}

#[derive(Debug, Clone)]
struct SubmoduleInfo {
    name: String,
    owner: String,
    version: String,
}

fn find_submodules(root: &PathBuf) -> Vec<SubmoduleInfo> {
    let mut submodules = Vec::new();

    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join(".git").exists() {
                // Try to get info from .git/config or Cargo.toml
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to read version from Cargo.toml
                let version = std::fs::read_to_string(path.join("Cargo.toml"))
                    .ok()
                    .and_then(|c| c.parse::<toml::Value>().ok())
                    .and_then(|doc| {
                        doc.get("package")
                            .and_then(|p| p.get("version"))
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                // Extract owner from git remote if possible
                let owner = std::process::Command::new("git")
                    .args(["-C", path.to_str().unwrap(), "config", "--get", "remote.upstream.url"])
                    .output()
                    .ok()
                    .and_then(|o| o.status.success().then_some(o))
                    .map(|o| {
                        let url = String::from_utf8_lossy(&o.stdout);
                        url.split('/')
                            .nth(url.matches('/').count().saturating_sub(1))
                            .unwrap_or("unknown")
                            .to_string()
                    })
                    .unwrap_or_else(|| "unknown".to_string());

                submodules.push(SubmoduleInfo { name, owner, version });
            }
        }
    }

    submodules
}