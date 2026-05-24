//! # Git Wrapper
//!
//! Unified git interface using direct git commands for all repository operations.
//!
//! ## Overview
//!
//! Provides a consistent API for git operations using the standard git CLI.
//! This is a simplified version that uses direct git commands exclusively,
//! without cargo-rail integration.
//!
//! ## Design
//!
//! - **`GitWrapper`**: Main struct holding configuration
//! - **Direct Git Only**: Uses `git` CLI with `-C` flag for directory specification
//! - **Thread-safe**: All operations are synchronous and stateless
//!
//! ## Supported Operations
//!
//! - `clone()` - Clone repositories
//! - `checkout()` - Switch branches/commits
//! - `get_branches()` - List available branches
//! - `get_tags()` - List available tags
//! - `branch_exists()` - Check branch presence
//!
//! ## Usage
//!
//! ```no_run
//! use cargo_vendormod::git_wrapper::GitWrapper;
//! use std::path::PathBuf;
//!
//! let wrapper = GitWrapper::new(PathBuf::from("git"));
//! wrapper.clone("https://github.com/user/repo.git", &PathBuf::from("./repo"))?;
//! wrapper.checkout(&PathBuf::from("./repo"), "main")?;
//! ```

use anyhow::Result;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Git wrapper using direct git commands
pub struct GitWrapper {
    git_exe: PathBuf,
}

impl GitWrapper {
    /// Create a new GitWrapper
    pub fn new(git_exe: PathBuf) -> Self {
        Self { git_exe }
    }

    /// Get the git executable path
    pub fn git_exe(&self) -> &PathBuf {
        &self.git_exe
    }

    /// Clone a repository
    pub fn clone(&self, url: &str, target_dir: &Path) -> Result<()> {
        self.direct_git_clone(url, target_dir)
    }

    /// Checkout a branch or commit
    pub fn checkout(&self, repo_path: &Path, refspec: &str) -> Result<()> {
        self.direct_git_checkout(repo_path, refspec)
    }

    /// Get list of branches
    pub fn get_branches(&self, repo_path: &Path) -> Result<Vec<String>> {
        self.direct_git_get_branches(repo_path)
    }

    /// Get list of tags
    pub fn get_tags(&self, repo_path: &Path) -> Result<Vec<String>> {
        self.direct_git_get_tags(repo_path)
    }

    /// Check if a branch exists
    pub fn branch_exists(&self, repo_path: &Path, branch_name: &str) -> Result<bool> {
        self.direct_git_branch_exists(repo_path, branch_name)
    }

    // Direct git implementations
    fn direct_git_clone(&self, url: &str, target_dir: &Path) -> Result<()> {
        let output = Command::new(&self.git_exe)
            .arg("clone")
            .arg(url)
            .arg(target_dir)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git clone failed: {}", error_msg));
        }
        Ok(())
    }

    fn direct_git_checkout(&self, repo_path: &Path, refspec: &str) -> Result<()> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("checkout")
            .arg(refspec)
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git checkout failed: {}", error_msg));
        }
        Ok(())
    }

    fn direct_git_get_branches(&self, repo_path: &Path) -> Result<Vec<String>> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("branch")
            .arg("--list")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git branch list failed: {}", error_msg));
        }

        let branches = String::from_utf8_lossy(&output.stdout);
        Ok(branches
            .lines()
            .map(|line| line.trim().trim_start_matches("* ").to_string())
            .collect())
    }

    fn direct_git_get_tags(&self, repo_path: &Path) -> Result<Vec<String>> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("tag")
            .arg("--list")
            .output()?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Git tag list failed: {}", error_msg));
        }

        let tags = String::from_utf8_lossy(&output.stdout);
        Ok(tags.lines().map(|line| line.trim().to_string()).collect())
    }

    fn direct_git_branch_exists(&self, repo_path: &Path, branch_name: &str) -> Result<bool> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("show-ref")
            .arg("--verify")
            .arg(format!("refs/heads/{}", branch_name))
            .output()?;

        Ok(output.status.success())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_git_wrapper_new() {
        let wrapper = GitWrapper::new(PathBuf::from("git"));
        assert_eq!(wrapper.git_exe, PathBuf::from("git"));
    }

    #[test]
    fn test_git_wrapper_with_path() {
        let wrapper = GitWrapper::new(PathBuf::from("/usr/bin/git"));
        assert_eq!(wrapper.git_exe, PathBuf::from("/usr/bin/git"));
    }

    #[test]
    fn test_git_exe_getter() {
        let wrapper = GitWrapper::new(PathBuf::from("/custom/path/git"));
        assert_eq!(wrapper.git_exe(), &PathBuf::from("/custom/path/git"));
    }
}
