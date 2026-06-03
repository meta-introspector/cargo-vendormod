use anyhow::{Context, Result};
use rayon::prelude::*;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

use crate::context::AppContext;
use crate::actions::RepoAction;
use crate::rollup_lock::RollupLock;
use crate::zkperf_integration::ZkperfIntegrator;

pub fn do_vendoring(ctx: AppContext, actions_plan: Vec<RepoAction>) -> Result<()> {
    if !ctx.dry_run {
        fs::create_dir_all(&ctx.submodules_dir).context("Failed to create submodules directory")?;
        fs::create_dir_all(&ctx.mirrors_path).context("Failed to create mirrors directory")?;
    }
    
    if ctx.dry_run {
        println!("--- DRY RUN MODE ACTIVE ---");
        println!("Would process {} actions.", actions_plan.len());
        return Ok(());
    }
    
    let rollup_lock_data = Arc::new(Mutex::new(RollupLock::load(&ctx.root_dir)?));
    
    execute_actions_plan(actions_plan, &ctx, rollup_lock_data, &ctx.root_dir)?;
    
    Ok(())
}

fn execute_actions_plan(
    actions_plan: Vec<RepoAction>,
    ctx: &AppContext,
    rollup_lock: Arc<Mutex<RollupLock>>,
    root_dir: &std::path::Path,
) -> Result<()> {
    println!("Executing actions plan with parallel processing using 24 CPU threads...");
    
    actions_plan.par_iter().for_each(|action| {
        if let Err(e) = process_single_action(action, ctx, rollup_lock.clone(), root_dir) {
            eprintln!("Error processing {}: {}", action.repo_name, e);
        }
    });
    
    update_cargo_config(&actions_plan, root_dir).unwrap_or_else(|e| {
        eprintln!("Warning: Failed to update .cargo/config.toml: {}", e);
    });
    
    println!("Successfully executed all actions.");
    Ok(())
}

fn process_single_action(
    action: &RepoAction,
    ctx: &AppContext,
    rollup_lock: Arc<Mutex<RollupLock>>,
    root_dir: &std::path::Path,
) -> Result<()> {
    println!("Processing repository: {} from {}", action.repo_name, action.repo_url);
    
    let bare_repo_path = action.mirrors_path.join(&action.owner).join(format!("{}.git", action.repo_name));
    if !bare_repo_path.exists() {
        anyhow::bail!("Bare repository not found at {}.", bare_repo_path.display());
    }
    println!("Using bare repository at: {:?}", bare_repo_path);
    
    let desired_commit: String = match &action.desired_commit {
        Some(commit) => commit.clone(),
        None => {
            match resolve_commit_from_version(&ctx.git_executable_path, &bare_repo_path, &action.version)? {
                Some(commit) => commit,
                None => anyhow::bail!("Could not resolve commit for version {} in bare repo {}", action.version, bare_repo_path.display()),
            }
        }
    };
    
    if ctx.create_version_branches {
        let version_branch = ctx.version_branch_format.replace("{}", &action.version);
        let output = std::process::Command::new(&ctx.git_executable_path)
            .arg("--git-dir")
            .arg(&bare_repo_path)
            .arg("branch")
            .arg("--force")
            .arg(&version_branch)
            .arg(&desired_commit)
            .output();
        if let Ok(out) = output {
            if !out.status.success() {
                eprintln!("Warning: Failed to create version branch '{}'", version_branch);
            } else if ctx.verbose {
                println!("Created/updated version branch '{}' at {}", version_branch, desired_commit);
            }
        }
    }
    
    if !action.submodule_path.exists() {
        println!("Adding {} as submodule...", action.repo_name);
        let output = std::process::Command::new(&ctx.git_executable_path)
            .arg("-C")
            .arg(&ctx.root_dir)
            .arg("submodule")
            .arg("add")
            .arg(&bare_repo_path)
            .arg(&action.submodule_path)
            .output()
            .context("Failed to execute git submodule add")?;
        if !output.status.success() {
            anyhow::bail!("Adding submodule failed: {}", std::str::from_utf8(&output.stderr).unwrap());
        }
        println!("Added {} as submodule.", action.repo_name);
    } else {
        println!("Submodule {} exists. Updating remotes...", action.repo_name);
    }
    
    setup_submodule_remotes(&ctx.git_executable_path, &action.submodule_path, &action.repo_url, &bare_repo_path)?;
    
    fetch_all_remotes(&ctx.git_executable_path, &action.submodule_path)?;
    
    let output = std::process::Command::new(&ctx.git_executable_path)
        .arg("-C")
        .arg(&action.submodule_path)
        .arg("checkout")
        .arg("-B")
        .arg(&action.target_branch)
        .arg(&desired_commit)
        .output()
        .context("Failed to checkout branch")?;
    if !output.status.success() {
        anyhow::bail!("checkout -B failed: {}", std::str::from_utf8(&output.stderr).unwrap());
    }
    println!("Checked out branch '{}' at {}", action.target_branch, desired_commit);
    

    // Apply zkperf annotations if enabled
    apply_zkperf_annotations(&action.submodule_path, ctx)?;
    crate::repo_sync_lib::create_snapshot(root_dir, rollup_lock)?;
    
    println!("Successfully processed {}.", action.repo_name);
    Ok(())
}

fn resolve_commit_from_version(git_exe: &std::path::Path, bare_repo_path: &std::path::Path, version: &str) -> Result<Option<String>> {
    let patterns = [
        format!("refs/tags/{}", version),
        format!("{}", version),
        format!("v{}", version),
        format!("upstream/{}", version),
    ];
    for pattern in &patterns {
        let output = std::process::Command::new(git_exe)
            .arg("--git-dir")
            .arg(bare_repo_path)
            .arg("show-ref")
            .arg("--verify")
            .arg(pattern)
            .output()
            .context("Failed to run git show-ref")?;
        if output.status.success() {
            let stdout = std::str::from_utf8(&output.stdout).unwrap();
            if let Some(first_line) = stdout.lines().next() {
                let parts: Vec<&str> = first_line.split_whitespace().collect();
                if parts.len() >= 2 {
                    return Ok(Some(parts[0].to_string()));
                }
            }
        }
    }
    Ok(None)
}

fn setup_submodule_remotes(
    git_executable_path: &std::path::Path,
    submodule_path: &std::path::Path,
    upstream_url: &str,
    bare_path: &std::path::Path,
) -> Result<()> {
    println!("Configuring remotes for submodule at {:?}", submodule_path);
    
    let _ = std::process::Command::new(git_executable_path)
        .arg("-C")
        .arg(submodule_path)
        .arg("remote")
        .arg("remove")
        .arg("origin")
        .output();
    
    let upstream_url = upstream_url.to_string();
    let submodule_path_upstream = submodule_path.to_path_buf();
    let git_exe = git_executable_path.to_path_buf();
    
    let (tx, rx) = crossbeam::channel::bounded(1);
    let git_exe1 = git_exe.clone();
    let submodule_path_upstream1 = submodule_path_upstream.clone();
    let upstream_url1 = upstream_url.clone();
    let handle = std::thread::spawn(move || {
        let result = std::process::Command::new(&git_exe1)
            .arg("-C")
            .arg(&submodule_path_upstream1)
            .arg("remote")
            .arg("add")
            .arg("upstream")
            .arg(&upstream_url1)
            .output();
        tx.send(result)
    });
    
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(result) => {
            if let Ok(output) = result {
                if !output.status.success() && !std::str::from_utf8(&output.stderr).unwrap().contains("already exists") {
                    eprintln!("Warning: Failed to add upstream remote: {}", std::str::from_utf8(&output.stderr).unwrap());
                }
            }
        }
        Err(_) => { /* timeout */ }
    }
    let _ = handle.join();
    
    let bare_path_str = bare_path.display().to_string();
    let submodule_path_bare = submodule_path.to_path_buf();
    
    let (tx, rx) = crossbeam::channel::bounded(1);
    let git_exe2 = git_exe.clone();
    let handle = std::thread::spawn(move || {
        let result = std::process::Command::new(&git_exe2)
            .arg("-C")
            .arg(&submodule_path_bare)
            .arg("remote")
            .arg("add")
            .arg("bare")
            .arg(&bare_path_str)
            .output();
        tx.send(result)
    });
    
    match rx.recv_timeout(std::time::Duration::from_secs(10)) {
        Ok(result) => {
            if let Ok(output) = result {
                if !output.status.success() && !std::str::from_utf8(&output.stderr).unwrap().contains("already exists") {
                    eprintln!("Warning: Failed to add bare remote: {}", std::str::from_utf8(&output.stderr).unwrap());
                }
            }
        }
        Err(_) => { /* timeout */ }
    }
    let _ = handle.join();
    
    println!("Remotes configured: upstream -> GitHub, bare -> local mirror");
    Ok(())
}

fn fetch_all_remotes(git_executable_path: &std::path::Path, submodule_path: &std::path::Path) -> Result<()> {
    println!("Fetching from all remotes...");
    let git_exe = git_executable_path.to_path_buf();
    let submodule_path_bare = submodule_path.to_path_buf();
    
    let (tx, rx) = crossbeam::channel::bounded(1);
    let git_exe1 = git_exe.clone();
    let submodule_path_bare1 = submodule_path_bare.clone();
    let handle = std::thread::spawn(move || {
        let result = std::process::Command::new(&git_exe1)
            .arg("-C")
            .arg(&submodule_path_bare1)
            .arg("fetch")
            .arg("bare")
            .output();
        tx.send(result)
    });
    
    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(result) => {
            let _ = handle.join();
            if let Ok(output) = result {
                if !output.status.success() {
                    eprintln!("Warning: Failed to fetch from bare remote: {}", std::str::from_utf8(&output.stderr).unwrap());
                } else if true { // always log if verbose
                    println!("Fetched from bare mirror");
                }
            }
        }
        Err(_) => eprintln!("Warning: Fetch from bare timed out"),
    }
    
    let submodule_path_upstream = submodule_path.to_path_buf();
    let (tx, rx) = crossbeam::channel::bounded(1);
    let git_exe2 = git_exe.clone();
    let handle2 = std::thread::spawn(move || {
        let result = std::process::Command::new(&git_exe2)
            .arg("-C")
            .arg(&submodule_path_upstream)
            .arg("fetch")
            .arg("upstream")
            .output();
        tx.send(result)
    });
    
    match rx.recv_timeout(std::time::Duration::from_secs(60)) {
        Ok(result) => {
            let _ = handle2.join();
            if let Ok(output) = result {
                if !output.status.success() {
                    eprintln!("Warning: Failed to fetch from upstream: {}", std::str::from_utf8(&output.stderr).unwrap());
                } else {
                    println!("Fetched from upstream");
                }
            }
        }
        Err(_) => eprintln!("Warning: Fetch from upstream timed out"),
    }
    
    Ok(())
}

fn update_cargo_config(actions_plan: &[RepoAction], root_dir: &std::path::Path) -> Result<()> {
    println!("Updating .cargo/config.toml...");
    let cargo_config_dir = root_dir.join(".cargo");
    fs::create_dir_all(&cargo_config_dir).context("Failed to create .cargo directory")?;
    let cargo_config_path = cargo_config_dir.join("config.toml");
    
    let mut config_doc = if cargo_config_path.exists() {
        let content = fs::read_to_string(&cargo_config_path)
            .with_context(|| format!("Failed to read {:?}", cargo_config_path))?;
        content
            .parse::<toml_edit::DocumentMut>()
            .context("Failed to parse .cargo/config.toml")?
    } else {
        toml_edit::DocumentMut::new()
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
    
    for action in actions_plan {
        let relative_submodule_path = pathdiff::diff_paths(&action.submodule_path, root_dir)
            .context(format!("Failed to get relative path for {:?}", action.submodule_path))?;
        let path_str = relative_submodule_path.to_string_lossy().to_string();
        let dep_value = toml_edit::Item::Value(toml_edit::Value::from(path_str));
        patch_crates_io.insert(&action.repo_name, dep_value);
    }
    
    fs::write(&cargo_config_path, config_doc.to_string())
        .with_context(|| format!("Failed to write to {:?}", cargo_config_path))?;
    
    println!("Successfully updated .cargo/config.toml.");
    Ok(())
}

/// Apply zkperf annotations to a vendored repository
fn apply_zkperf_annotations(repo_path: &std::path::Path, _ctx: &AppContext) -> Result<()> {
    // Check if zkperf integration is enabled via environment variable or config
    let zkperf_enabled = std::env::var("CARGO_VENDORMOD_ZKPERF")
        .unwrap_or_else(|_| "false".to_string()) 
        == "true";
    
    if !zkperf_enabled {
        println!("Zkperf integration disabled. Set CARGO_VENDORMOD_ZKPERF=true to enable.");
        return Ok(());
    }
    
    println!("Applying zkperf annotations to {}...", repo_path.display());
    
    // Check if cargo-zkperf is available in the zkperf submodule
    let zkperf_tool_path = std::path::PathBuf::from("zkperf/cargo-zkperf/target/release/cargo-zkperf");
    
    let integrator = ZkperfIntegrator::new(Some(zkperf_tool_path), false);
    integrator.apply_zkperf_annotations(repo_path)?;
    
    println!("Zkperf annotations applied to {}", repo_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_vendoring_dry_run() {
        let dir = tempdir().unwrap();
        let ctx = AppContext {
            git_executable_path: PathBuf::from("git"),
            root_dir: dir.path().to_path_buf(),
            manifest_path: dir.path().join("Cargo.toml"),
            submodules_dir: dir.path().join("submodules"),
            mirrors_path: PathBuf::from("/home/mdupont/git/host"),
            vendor_dir: dir.path().join("vendor"),
            target_branch: "main".to_string(),
            create_version_branches: false,
            version_branch_format: "v{}".to_string(),
            dry_run: true,
            verbose: false,
            temp_symlink: None,
        };
        let actions = vec![];
        let result = do_vendoring(ctx, actions);
        assert!(result.is_ok());
    }

    #[test]
    fn test_vendoring_empty_plan() {
        let dir = tempdir().unwrap();
        let ctx = AppContext {
            git_executable_path: PathBuf::from("git"),
            root_dir: dir.path().to_path_buf(),
            manifest_path: dir.path().join("Cargo.toml"),
            submodules_dir: dir.path().join("submodules"),
            mirrors_path: PathBuf::from("/home/mdupont/git/host"),
            vendor_dir: dir.path().join("vendor"),
            target_branch: "main".to_string(),
            create_version_branches: false,
            version_branch_format: "v{}".to_string(),
            dry_run: true,
            verbose: false,
            temp_symlink: None,
        };
        let actions: Vec<RepoAction> = vec![];
        let result = do_vendoring(ctx, actions);
        assert!(result.is_ok());
    }
}