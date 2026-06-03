//! Shared vendoring command operations — callable directly from both main.rs and vendoring.rs.
//! No sub-process delegation; all logic runs in-process for WASM/blockchain compatibility.

use anyhow::{Context, Result};
use std::path::Path;

/// Info about a discovered submodule.
#[derive(Debug, Clone)]
pub struct SubmoduleInfo {
    pub name: String,
    pub owner: String,
    pub version: String,
}

/// Discover submodules inside `root` by looking for directories containing a `.git` entry.
pub fn find_submodules(root: &Path) -> Vec<SubmoduleInfo> {
    let mut submodules = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.join(".git").exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();
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

/// Initialize vendoring: create submodules & mirrors dirs, discover submodules, clone them.
pub fn cmd_init(
    source_repo: &Path,
    submodules_path: &Path,
    mirrors_path: &Path,
    dry_run: bool,
) -> Result<()> {
    println!("=== Initializing vendoring ===");
    println!("Source:      {}", source_repo.display());
    println!("Submodules:  {}", submodules_path.display());
    println!("Mirrors:     {}", mirrors_path.display());

    if !dry_run {
        std::fs::create_dir_all(submodules_path)
            .context("Failed to create submodules directory")?;
        std::fs::create_dir_all(mirrors_path)
            .context("Failed to create mirrors directory")?;
    }

    let submodules = crate::submodule_discovery::discover_submodules(source_repo)?;
    println!("Found {} submodules", submodules.len());
    for sm in &submodules {
        println!("  - {} -> {}", sm.path.display(), sm.url);
    }

    if !dry_run {
        let git_exe = crate::submodule_discovery::get_git_executable();
        crate::submodule_discovery::clone_submodules_to_target(submodules, submodules_path, &git_exe)?;
    }

    println!("=== Vendoring initialized ===");
    Ok(())
}

/// Fetch upstream for all submodules via their bare mirrors.
pub fn cmd_fetch_upstream(
    submodules_path: &Path,
    mirrors_path: &Path,
    git_exe: &Path,
    dry_run: bool,
    verbose: bool,
    default_threads: usize,
) -> Result<()> {
    println!("Fetching upstream from all mirrors...");
    let submodules = find_submodules(submodules_path);
    println!("Found {} submodules, using {} threads", submodules.len(), default_threads);

    if dry_run {
        println!("[DRY RUN] Would fetch upstream for all repos");
        return Ok(());
    }

    use rayon::prelude::*;
    submodules.par_iter().for_each(|submodule| {
        let bare_path = mirrors_path
            .join(&submodule.owner)
            .join(format!("{}.git", &submodule.name));
        if !bare_path.exists() {
            eprintln!("Bare repo not found: {}", bare_path.display());
            return;
        }
        let output = std::process::Command::new(git_exe)
            .arg("--git-dir")
            .arg(&bare_path)
            .arg("fetch")
            .arg("upstream")
            .arg("--prune")
            .arg("--tags")
            .output();
        match output {
            Ok(out) if !out.status.success() => {
                eprintln!(
                    "Failed to fetch {}: {}",
                    submodule.name,
                    String::from_utf8_lossy(&out.stderr)
                );
            }
            Ok(_) if verbose => {
                println!("Fetched upstream for {}", submodule.name);
            }
            _ => {}
        }
    });
    println!("Upstream fetch complete.");
    Ok(())
}

/// Rebase all submodules onto upstream target branch.
pub fn cmd_rebase(
    submodules_path: &Path,
    target_branch: &str,
    git_exe: &Path,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    println!("Rebasing submodules onto upstream...");
    let submodules = find_submodules(submodules_path);

    if dry_run {
        println!("[DRY RUN] Would rebase {} submodules", submodules.len());
        return Ok(());
    }

    use rayon::prelude::*;
    submodules.par_iter().for_each(|submodule| {
        let sub_path = submodules_path.join(&submodule.name);
        if !sub_path.exists() {
            eprintln!("Submodule not found: {}", sub_path.display());
            return;
        }
        let _ = std::process::Command::new(git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("fetch")
            .arg("upstream")
            .output();

        let output = std::process::Command::new(git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("rebase")
            .arg(&format!("upstream/{}", target_branch))
            .output();
        match output {
            Ok(out) if !out.status.success() => {
                eprintln!(
                    "Rebase failed for {}: {}",
                    submodule.name,
                    String::from_utf8_lossy(&out.stderr)
                );
                let _ = std::process::Command::new(git_exe)
                    .arg("-C")
                    .arg(&sub_path)
                    .arg("rebase")
                    .arg("--abort")
                    .output();
            }
            Ok(_) if verbose => {
                println!("Rebased {} onto upstream/{}", submodule.name, target_branch);
            }
            _ => {}
        }
    });
    println!("Rebase complete.");
    Ok(())
}

/// Fetch release tags and create version branches in bare mirrors.
pub fn cmd_releases(
    submodules_path: &Path,
    mirrors_path: &Path,
    git_exe: &Path,
    version_branch_format: &str,
    create_version_branches: bool,
    dry_run: bool,
) -> Result<()> {
    println!("Fetching release tags and creating version branches...");
    let submodules = find_submodules(submodules_path);

    if !create_version_branches {
        println!("Version branch creation is disabled");
        return Ok(());
    }
    if dry_run {
        println!("[DRY RUN] Would create version branches for {} submodules", submodules.len());
        return Ok(());
    }

    use rayon::prelude::*;
    submodules.par_iter().for_each(|submodule| {
        let bare_path = mirrors_path
            .join(&submodule.owner)
            .join(format!("{}.git", &submodule.name));
        if !bare_path.exists() {
            return;
        }
        let _ = std::process::Command::new(git_exe)
            .arg("--git-dir")
            .arg(&bare_path)
            .arg("fetch")
            .arg("upstream")
            .arg("--tags")
            .output();

        let version_branch = version_branch_format.replace("{}", &submodule.version);
        let _ = std::process::Command::new(git_exe)
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

/// Show status of all submodules vs upstream.
pub fn cmd_status(submodules_path: &Path, git_exe: &Path) -> Result<()> {
    println!("Submodule status:");
    let submodules = find_submodules(submodules_path);

    use rayon::prelude::*;
    submodules.par_iter().for_each(|submodule| {
        let sub_path = submodules_path.join(&submodule.name);
        if !sub_path.exists() {
            println!("{}: NOT FOUND", submodule.name);
            return;
        }
        let current = std::process::Command::new(git_exe)
            .arg("-C")
            .arg(&sub_path)
            .arg("rev-parse")
            .arg("HEAD")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_else(|_| "N/A".to_string());
        let upstream = std::process::Command::new(git_exe)
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
        println!(
            "{}: {} (current: {}, upstream: {})",
            submodule.name,
            status,
            current.chars().take(8).collect::<String>(),
            upstream.chars().take(8).collect::<String>()
        );
    });
    Ok(())
}

/// Full sync: fetch upstream → rebase → patch.
pub fn cmd_sync(
    submodules_path: &Path,
    mirrors_path: &Path,
    target_branch: &str,
    git_exe: &Path,
    dry_run: bool,
    verbose: bool,
    default_threads: usize,
) -> Result<()> {
    println!("Running full sync: fetch-upstream → rebase → patch");
    cmd_fetch_upstream(submodules_path, mirrors_path, git_exe, dry_run, verbose, default_threads)?;
    cmd_rebase(submodules_path, target_branch, git_exe, dry_run, verbose)?;
    cmd_patch(submodules_path)?;
    println!("Full sync complete.");
    Ok(())
}

/// Generate/update .cargo/config.toml patches for vendored crates.
pub fn cmd_patch(submodules_path: &Path) -> Result<()> {
    println!("Generating cargo patches...");
    let root_dir = std::env::current_dir()?;
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
    if let Ok(entries) = std::fs::read_dir(submodules_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let relative = pathdiff::diff_paths(&path, &root_dir)
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
