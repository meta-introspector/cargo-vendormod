use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;
use std::sync::{Arc, Mutex};

pub fn create_snapshot(root_dir: &Path, rollup_lock: Arc<Mutex<crate::RollupLock>>) -> Result<()> {
    println!("Creating snapshot of current state...");

    // Check if git status is clean
    if !is_git_status_clean(root_dir)? {
        anyhow::bail!("Git repository has uncommitted changes. Please commit or stash before creating snapshot.");
    }

    // Update submodules with their current commit hashes
    update_submodule_commits(root_dir, &rollup_lock)?;

    // Add all changes to git
    git_add_all(root_dir)?;

    // Commit the changes
    let commit_message = format!("Update submodules and dependencies snapshot - {}", 
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"));
    git_commit(root_dir, &commit_message)?;

    // Update rollup lock with timestamp
    let mut lock = rollup_lock.lock().unwrap();
    lock.metadata.insert("last_snapshot".to_string(), 
        chrono::Utc::now().to_rfc3339());
    drop(lock);

    println!("Snapshot created successfully.");
    Ok(())
}

fn is_git_status_clean(root_dir: &Path) -> Result<bool> {
    let output = Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .current_dir(root_dir)
        .output()
        .context("Failed to execute git status")?;

    if output.status.success() {
        let status_output = String::from_utf8_lossy(&output.stdout);
        // Clean if output is empty or only contains untracked files (which we'll handle)
        Ok(status_output.trim().is_empty() || status_output.lines().all(|line| line.starts_with("??")))
    } else {
        anyhow::bail!("Git status failed: {}", String::from_utf8_lossy(&output.stderr))
    }
}

fn git_add_all(root_dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .arg("add")
        .arg("-A")
        .current_dir(root_dir)
        .output()
        .context("Failed to execute git add")?;

    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git add failed: {}", String::from_utf8_lossy(&output.stderr))
    }
}

fn git_commit(root_dir: &Path, message: &str) -> Result<()> {
    let output = Command::new("git")
        .arg("commit")
        .arg("-m")
        .arg(message)
        .current_dir(root_dir)
        .output()
        .context("Failed to execute git commit")?;

    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!("Git commit failed: {}", String::from_utf8_lossy(&output.stderr))
    }
}

fn update_submodule_commits(root_dir: &Path, rollup_lock: &Arc<Mutex<crate::RollupLock>>) -> Result<()> {
    let submodules_to_update = {
        let lock = rollup_lock.lock().unwrap();
        let mut updates = Vec::new();
        
        for (name, submodule) in &lock.submodules {
            let submodule_path = root_dir.join(&submodule.path);
            
            if submodule_path.exists() {
                // Get current commit hash of the submodule
                if let Ok(commit) = get_git_commit_hash(&submodule_path) {
                    updates.push((name.clone(), submodule.clone(), commit));
                }
            }
        }
        updates
    };
    
    // Apply all updates after releasing the lock
    for (name, mut submodule, commit) in submodules_to_update {
        submodule.commit = commit;
        let mut lock = rollup_lock.lock().unwrap();
        lock.submodules.insert(name, submodule);
        drop(lock);
    }
    
    Ok(())
}

fn get_git_commit_hash(repo_path: &Path) -> Result<String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("HEAD")
        .current_dir(repo_path)
        .output()
        .context("Failed to get git commit hash")?;

    if output.status.success() {
        let hash = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(hash)
    } else {
        anyhow::bail!("Failed to get commit hash: {}", String::from_utf8_lossy(&output.stderr))
    }
}