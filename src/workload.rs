use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;
use std::collections::HashMap;
use toml_edit::{Document, DocumentMut};

use crate::context::AppContext;
use crate::repo_collection::{RepoIdentifier, collect_repo_info};
use crate::actions::{RepoAction, create_actions_plan};
use crate::workspace::{find_workspace_cargo_files, discover_all_git_repositories};
use crate::zkperf_integration::ZkperfIntegrator;

/// Enhanced workload vendoring that discovers ALL repositories (not just workspace members)
pub fn vendor_workload(
    ctx: &AppContext,
    workspace_path: &PathBuf,
    fork_dir: &PathBuf,
    apply_zkperf: bool,
) -> Result<()> {
    println!("Starting enhanced workload vendoring...");
    println!("Workspace path: {}", workspace_path.display());
    println!("Fork directory: {}", fork_dir.display());
    if apply_zkperf {
        println!("Will apply zkperf annotations");
    }
    
    // Step 1: Process ALL Cargo.toml files in the project (not just workspace members)
    let cargo_files = find_workspace_cargo_files(workspace_path, true)?;
    println!("Found {} Cargo.toml files in project", cargo_files.len());
    
    // Step 2: Discover ALL git repositories in the project
    let git_repos = discover_all_git_repositories(workspace_path)?;
    println!("Found {} git repositories in project", git_repos.len());
    
    // Step 3: Collect all git dependencies from all Cargo.toml files
    let mut all_repo_info = HashMap::new();
    
    for cargo_file in &cargo_files {
        if ctx.verbose {
            println!("Processing: {}", cargo_file.display());
        }
        
        // Temporarily update context to use this Cargo.toml
        let mut temp_ctx = ctx.clone();
        temp_ctx.manifest_path = cargo_file.clone();
        temp_ctx.root_dir = cargo_file.parent().unwrap().to_path_buf();
        
        // Collect repo info for this member
        let repo_info = collect_repo_info(&temp_ctx)?;
        
        // Merge into our master collection
        for (name, info) in repo_info {
            all_repo_info.insert(name, info);
        }
    }
    
    // Step 4: Add any git repositories that don't have Cargo.toml entries
    for git_repo in &git_repos {
        let repo_name = git_repo
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        if !all_repo_info.contains_key(repo_name) {
            println!("Adding git repository not found in Cargo.toml: {}", repo_name);
            // Add basic info for this repository
            all_repo_info.insert(repo_name.to_string(), RepoIdentifier {
                url: format!("local://{}", git_repo.display()),
                version: "0.0.0".to_string(),
                commit: None,
                owner: "local".to_string(),
                repo_name: repo_name.to_string(),
            });
        }
    }
    
    println!("Collected {} unique git dependencies from project", all_repo_info.len());
    
    // Step 5: Create local forks of each repository
    create_local_forks(&all_repo_info, fork_dir, &ctx.git_executable_path)?;
    
    // Step 6: Create actions plan for all dependencies
    let actions_plan = create_actions_plan(ctx, &all_repo_info);
    
    // Step 7: Execute the vendoring process using local forks
    do_workload_vendoring(ctx, actions_plan, fork_dir)?;
    
    // Step 8: Apply zkperf annotations if requested
    if apply_zkperf {
        apply_zkperf_annotations_with_integrator(fork_dir, &ctx.git_executable_path)?;
    }
    
    // Step 9: Create patches for all submodules
    create_workload_patches(ctx, fork_dir)?;
    
    println!("Enhanced workload vendoring completed successfully!");
    Ok(())
}

/// Create local forks (mirrors) of all repositories
fn create_local_forks(
    repo_info: &HashMap<String, RepoIdentifier>,
    fork_dir: &PathBuf,
    git_exe: &PathBuf,
) -> Result<()> {
    println!("Creating local forks of {} repositories...", repo_info.len());
    
    fs::create_dir_all(fork_dir).context("Failed to create fork directory")?;
    
    for (repo_name, identifier) in repo_info {
        println!("Forking {} from {}", repo_name, identifier.url);
        
        let fork_path = fork_dir.join(repo_name);
        
        // Create bare clone first
        let bare_path = fork_path.with_extension(".git");
        
        let clone_output = Command::new(git_exe)
            .arg("clone")
            .arg("--bare")
            .arg(&identifier.url)
            .arg(&bare_path)
            .output()
            .context(format!("Failed to clone {} as bare repo", repo_name))?;
        
        if !clone_output.status.success() {
            eprintln!("Warning: Failed to clone {}: {}", 
                repo_name, 
                String::from_utf8_lossy(&clone_output.stderr));
            continue;
        }
        
        // Create working copy
        let worktree_output = Command::new(git_exe)
            .arg("clone")
            .arg(&bare_path)
            .arg(&fork_path)
            .output()
            .context(format!("Failed to create working copy for {}", repo_name))?;
        
        if !worktree_output.status.success() {
            eprintln!("Warning: Failed to create working copy for {}: {}", 
                repo_name, 
                String::from_utf8_lossy(&worktree_output.stderr));
            continue;
        }
        
        // Checkout the specific version/commit if available
        if let Some(commit) = &identifier.commit {
            let checkout_output = Command::new(git_exe)
                .arg("-C")
                .arg(&fork_path)
                .arg("checkout")
                .arg(commit)
                .output();
            
            if let Ok(out) = checkout_output {
                if !out.status.success() {
                    eprintln!("Warning: Failed to checkout commit {} for {}: {}", 
                        commit, repo_name, String::from_utf8_lossy(&out.stderr));
                }
            }
        } else {
            // Try to checkout by version tag
            let version_tag = format!("v{}", identifier.version);
            let tag_output = Command::new(git_exe)
                .arg("-C")
                .arg(&fork_path)
                .arg("checkout")
                .arg(&version_tag)
                .output();
            
            if let Ok(out) = tag_output {
                if !out.status.success() {
                    // Try without 'v' prefix
                    let _ = Command::new(git_exe)
                        .arg("-C")
                        .arg(&fork_path)
                        .arg("checkout")
                        .arg(&identifier.version)
                        .output();
                }
            }
        }
        
        println!("Successfully forked {} to {}", repo_name, fork_path.display());
    }
    
    Ok(())
}

/// Execute vendoring using local forks instead of remote repositories
fn do_workload_vendoring(
    ctx: &AppContext,
    actions_plan: Vec<RepoAction>,
    fork_dir: &PathBuf,
) -> Result<()> {
    println!("Executing workload vendoring with local forks...");
    
    if !ctx.dry_run {
        fs::create_dir_all(&ctx.submodules_dir).context("Failed to create submodules directory")?;
    }
    
    if ctx.dry_run {
        println!("--- DRY RUN MODE ACTIVE ---");
        println!("Would process {} actions using local forks.", actions_plan.len());
        return Ok(());
    }
    
    for action in actions_plan {
        println!("Processing {} using local fork...", action.repo_name);
        
        let fork_path = fork_dir.join(&action.repo_name);
        let bare_fork_path = fork_path.with_extension(".git");
        
        if !bare_fork_path.exists() {
            eprintln!("Local fork not found for {}: {}", action.repo_name, bare_fork_path.display());
            continue;
        }
        
        // Add as submodule using local fork
        if !action.submodule_path.exists() {
            println!("Adding {} as submodule from local fork...", action.repo_name);
            
            let output = Command::new(&ctx.git_executable_path)
                .arg("-C")
                .arg(&ctx.root_dir)
                .arg("submodule")
                .arg("add")
                .arg(&bare_fork_path)
                .arg(&action.submodule_path)
                .output()
                .context("Failed to execute git submodule add")?;
            
            if !output.status.success() {
                eprintln!("Failed to add submodule {}: {}", 
                    action.repo_name, 
                    String::from_utf8_lossy(&output.stderr));
                continue;
            }
            
            println!("Added {} as submodule from local fork.", action.repo_name);
        }
        
        // Setup remotes pointing to both local fork and original upstream
        setup_workload_remotes(
            &ctx.git_executable_path,
            &action.submodule_path,
            &action.repo_url,
            &bare_fork_path
        )?;
        
        // Checkout the target branch
        let checkout_output = Command::new(&ctx.git_executable_path)
            .arg("-C")
            .arg(&action.submodule_path)
            .arg("checkout")
            .arg("-B")
            .arg(&action.target_branch)
            .output();
        
        if let Ok(out) = checkout_output {
            if out.status.success() {
                println!("Checked out branch '{}' for {}", action.target_branch, action.repo_name);
            } else {
                eprintln!("Warning: Failed to checkout branch for {}: {}", 
                    action.repo_name, 
                    String::from_utf8_lossy(&out.stderr));
            }
        }
    }
    
    Ok(())
}

/// Setup remotes for workload submodules (local fork + upstream)
fn setup_workload_remotes(
    git_exe: &PathBuf,
    submodule_path: &PathBuf,
    upstream_url: &str,
    bare_fork_path: &PathBuf,
) -> Result<()> {
    println!("Configuring remotes for {}...", submodule_path.display());
    
    // Remove origin if it exists
    let _ = Command::new(git_exe)
        .arg("-C")
        .arg(submodule_path)
        .arg("remote")
        .arg("remove")
        .arg("origin")
        .output();
    
    // Add upstream (original repository)
    let upstream_output = Command::new(git_exe)
        .arg("-C")
        .arg(submodule_path)
        .arg("remote")
        .arg("add")
        .arg("upstream")
        .arg(upstream_url)
        .output();
    
    if let Ok(out) = upstream_output {
        if !out.status.success() && !String::from_utf8_lossy(&out.stderr).contains("already exists") {
            eprintln!("Warning: Failed to add upstream remote: {}", 
                String::from_utf8_lossy(&out.stderr));
        }
    }
    
    // Add local fork as 'fork' remote
    let fork_path_str = bare_fork_path.display().to_string();
    let fork_output = Command::new(git_exe)
        .arg("-C")
        .arg(submodule_path)
        .arg("remote")
        .arg("add")
        .arg("fork")
        .arg(&fork_path_str)
        .output();
    
    if let Ok(out) = fork_output {
        if !out.status.success() && !String::from_utf8_lossy(&out.stderr).contains("already exists") {
            eprintln!("Warning: Failed to add fork remote: {}", 
                String::from_utf8_lossy(&out.stderr));
        }
    }
    
    println!("Remotes configured: upstream -> original, fork -> local");
    Ok(())
}

/// Apply cargo zkperf annotations to all forked repositories using the integrator
fn apply_zkperf_annotations_with_integrator(fork_dir: &PathBuf, git_exe: &PathBuf) -> Result<()> {
    println!("Applying cargo zkperf annotations to all forks...");
    
    // Try to use external zkperf tool first, fall back to built-in
    let zkperf_path = PathBuf::from("/home/mdupont/projects/cargo/submodules/zkperf/cargo-zkperf");
    let integrator = ZkperfIntegrator::new(Some(zkperf_path), true);
    
    if let Ok(entries) = fs::read_dir(fork_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.extension().map_or(false, |ext| ext == "git") {
                if let Some(repo_name) = path.file_name().and_then(|n| n.to_str()) {
                    println!("Processing {} with zkperf...", repo_name);
                    integrator.apply_zkperf_annotations(&path)?;
                }
            }
        }
    }
    
    Ok(())
}

/// Create cargo patches for workload using local forks
fn create_workload_patches(ctx: &AppContext, fork_dir: &PathBuf) -> Result<()> {
    println!("Creating cargo patches for workload...");
    
    if ctx.dry_run {
        println!("[DRY RUN] Would create patches using local forks");
        return Ok(());
    }
    
    let cargo_config_dir = ctx.root_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir).context("Failed to create .cargo directory")?;
    let cargo_config_path = cargo_config_dir.join("config.toml");
    
    let mut config_doc = if cargo_config_path.exists() {
        let content = fs::read_to_string(&cargo_config_path)
            .with_context(|| format!("Failed to read {:?}", cargo_config_path))?;
        content
            .parse::<DocumentMut>()
            .context("Failed to parse .cargo/config.toml")?
    } else {
        DocumentMut::new()
    };
    
    let patch = config_doc
        .entry("patch")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("patch entry is not a table")?;
    
    let patch_crates_io = patch
        .entry("crates-io")
        .or_insert(toml_edit::table())
        .as_table_mut()
        .context("crates-io entry is not a table")?;
    
    // Add patches for all forked repositories
    if let Ok(entries) = fs::read_dir(fork_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.extension().map_or(false, |ext| ext == "git") {
                if let Some(repo_name) = path.file_name().and_then(|n| n.to_str()) {
                    let relative_path = pathdiff::diff_paths(&path, &ctx.root_dir)
                        .unwrap_or(path.clone());
                    let path_str = relative_path.to_string_lossy().to_string();
                    
                    let dep_value = toml_edit::Item::Value(toml_edit::Value::from(path_str));
                    patch_crates_io.insert(repo_name, dep_value);
                    
                    if ctx.verbose {
                        println!("Added patch for {} -> {}", repo_name, relative_path.display());
                    }
                }
            }
        }
    }
    
    let config_content = config_doc.to_string();
    fs::write(&cargo_config_path, config_content)
        .with_context(|| format!("Failed to write to {:?}", cargo_config_path))?;
    
    println!("Successfully created cargo patches for workload.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_repo_identifier_creation() {
        let info = RepoIdentifier {
            url: "https://github.com/rust-lang/cargo".to_string(),
            version: "1.0.0".to_string(),
            commit: Some("abc123".to_string()),
            owner: "rust-lang".to_string(),
            repo_name: "cargo".to_string(),
        };
        assert_eq!(info.owner, "rust-lang");
        assert_eq!(info.repo_name, "cargo");
    }

    #[test]
    fn test_repo_identifier_local_url() {
        let info = RepoIdentifier {
            url: "local:///path/to/repo".to_string(),
            version: "0.0.0".to_string(),
            commit: None,
            owner: "local".to_string(),
            repo_name: "repo".to_string(),
        };
        assert!(info.url.starts_with("local://"));
    }
}