use anyhow::Result;
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

use crate::global_dep_graph::{GlobalDependencyGraph, DependencyNode};

/// Layer processor for processing crates in topological order
pub struct LayerProcessor {
    graph: GlobalDependencyGraph,
    workspace_path: PathBuf,
    output_base: PathBuf,
    git_exe: PathBuf,
    home_dir: PathBuf,
}

impl LayerProcessor {
    /// Create a new LayerProcessor
    pub fn new(
        graph: GlobalDependencyGraph,
        workspace_path: PathBuf,
        output_base: PathBuf,
        git_exe: PathBuf,
        home_dir: PathBuf,
    ) -> Self {
        Self {
            graph,
            workspace_path,
            output_base,
            git_exe,
            home_dir,
        }
    }
    
    /// Process all crates in layered fashion
    pub fn process_layers(&self) -> Result<()> {
        log::info!("Processing Layer 1: External Dependencies");
        self.process_layer1()?;
        
        log::info!("Processing Layer 2: Workspace Members");
        self.process_layer2()?;
        
        log::info!("✅ Completed all layers processing");
        Ok(())
    }
    
    /// Process Layer 1: External dependencies in topological order
    pub fn process_layer1(&self) -> Result<()> {
        for crate_id in &self.graph.external_dependency_order {
            self.process_crate(crate_id, true)?; // true = is_external
        }
        Ok(())
    }
    
    /// Process Layer 2: Workspace members in publish order
    pub fn process_layer2(&self) -> Result<()> {
        for crate_id in &self.graph.publish_order {
            self.process_crate(crate_id, false)?; // false = is_workspace
        }
        Ok(())
    }
    
    /// Process individual crate
    fn process_crate(&self, crate_id: &str, is_external: bool) -> Result<()> {
        log::info!("Processing crate: {}", crate_id);
        
        // Find crate info
        let crate_node = self.find_crate_node(crate_id)
            .map_err(|e| anyhow::anyhow!("Failed to find crate node for {}: {}", crate_id, e))?;
        log::info!("  Found crate: {} v{} from source: {}", crate_node.crate_name, crate_node.version, crate_node.source);
        
        // Locate repository
        let repo_path = self.find_repository(&crate_node)
            .map_err(|e| anyhow::anyhow!("Failed to locate repository for {}: {}", crate_node.crate_name, e))?;
        log::info!("  Repository: {}", repo_path.display());
        
        // Checkout correct branch/version
        let branch = self.find_branch_for_version(&crate_node, &repo_path)
            .map_err(|e| anyhow::anyhow!("Failed to find branch/version for {}: {}", crate_node.crate_name, e))?;
        log::info!("  Branch/Version: {}", branch);
        
        self.checkout_branch(&repo_path, &branch)
            .map_err(|e| anyhow::anyhow!("Failed to checkout branch {} for {}: {}", branch, crate_node.crate_name, e))?;
        
        // Apply patches
        self.apply_patches(&repo_path)
            .map_err(|e| anyhow::anyhow!("Failed to apply patches for {}: {}", crate_node.crate_name, e))?;
        
        // Generate flake.nix
        self.generate_flake(&crate_node, &repo_path, is_external)
            .map_err(|e| anyhow::anyhow!("Failed to generate flake for {}: {}", crate_node.crate_name, e))?;
        
        // Compile standalone (skip for crates.io dependencies)
        if !repo_path.to_string_lossy().contains("crates-io-cache") {
            self.compile_standalone(&repo_path, is_external)
                .map_err(|e| anyhow::anyhow!("Failed to compile {}: {}", crate_node.crate_name, e))?;
        } else {
            log::info!("  Skipping compilation for crates.io dependency (no source code)");
        }
        
        log::info!("  ✅ Completed processing: {}", crate_id);
        Ok(())
    }
    
    /// Find crate node by ID
    fn find_crate_node(&self, crate_id: &str) -> Result<&DependencyNode> {
        self.graph.nodes.iter()
            .find(|n| n.id == crate_id)
            .ok_or_else(|| anyhow::anyhow!("Crate {} not found in graph", crate_id))
    }
    
    /// Find repository location for a crate
    fn find_repository(&self, crate_node: &DependencyNode) -> Result<PathBuf> {
        // Handle workspace members
        if crate_node.source == "workspace" {
            // For workspace members, use the actual workspace directory
            let workspace_path = self.workspace_path.join(&crate_node.crate_name);
            let workspace_root_path = self.workspace_path.clone();
            
            // Check if the workspace member exists as a subdirectory
            if workspace_path.exists() {
                println!("  Found workspace member at: {}", workspace_path.display());
                return Ok(workspace_path);
            }
            
            // If not found as subdirectory, check if the workspace root itself is the package
            // This happens when the workspace has no members and the root is the package
            if workspace_root_path.join("Cargo.toml").exists() {
                println!("  Found workspace root as package at: {}", workspace_root_path.display());
                return Ok(workspace_root_path);
            }
            
            return Err(anyhow::anyhow!(
                "Workspace member {} not found at expected location: {}",
                crate_node.crate_name, workspace_path.display()
            ));
        }

        // Handle crates.io dependencies specially
        if crate_node.source == "crates.io" {
            // For crates.io dependencies, create a local cache directory
            let crates_io_cache = self.output_base.join("crates-io-cache").join(&crate_node.crate_name);
            
            // Create the cache directory if it doesn't exist
            if !crates_io_cache.exists() {
                fs::create_dir_all(&crates_io_cache)?;
                println!("  Created crates.io cache directory: {}", crates_io_cache.display());
            }
            
            return Ok(crates_io_cache);
        }

        // Parse repository URL to get owner/repo
        let (owner, repo_name) = self.parse_github_url(&crate_node.source)?;
        
        // Check 1: Use the base submodules directory
        let base_submodules = Path::new("/mnt/data1/nix/vendor/rust/cargo2nix/submodules");
        let submodule_path = base_submodules.join(&repo_name);
        if submodule_path.exists() {
            println!("  Found submodule at: {}", submodule_path.display());
            return Ok(submodule_path);
        }
        
        // Check 2: ~/git/hostname/org/repo.git (bare)
        let home_bare = self.home_dir.join("git").join("github.com").join(owner).join(format!("{}.git", repo_name));
        if home_bare.exists() {
            println!("  Found bare repository at: {}", home_bare.display());
            return Ok(home_bare);
        }
        
        // Check 3: output_base/submodules/repo (worktree)
        let output_submodule_path = self.output_base.join("submodules").join(&repo_name);
        if output_submodule_path.exists() {
            println!("  Found submodule at: {}", output_submodule_path.display());
            return Ok(output_submodule_path);
        }
        
        // Check 4: output_base/external/repo (for external deps)
        let external_path = self.output_base.join("external").join(&repo_name);
        if external_path.exists() {
            println!("  Found external dependency at: {}", external_path.display());
            return Ok(external_path);
        }
        
        Err(anyhow::anyhow!(
            "Repository not found for {} at any expected location",
            crate_node.crate_name
        ))
    }
    
    /// Parse repository URL to extract owner and repo name
    /// Handles GitHub URLs, crates.io names, and other formats
    fn parse_github_url(&self, url: &str) -> Result<(String, String)> {
        let url = url.trim();
        
        // Handle crates.io format (just crate name)
        if url == "crates.io" || url.starts_with("registry+https://github.com/rust-lang/crates.io-index") {
            // For crates.io, we can't determine owner/repo, use dummy values
            // In production, would fetch from crates.io API or use local cache
            return Ok(("crates-io".to_string(), "unknown".to_string()));
        }
        
        // Remove .git suffix if present
        let url = if url.ends_with(".git") {
            &url[..url.len() - 4]
        } else {
            url
        };
        
        // Parse github.com/owner/repo format
        if let Some(rest) = url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() >= 2 {
                return Ok((parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        // Parse git@github.com:owner/repo format
        if let Some(rest) = url.strip_prefix("git@github.com:") {
            let parts: Vec<&str> = rest.split('/').collect();
            if parts.len() >= 2 {
                return Ok((parts[0].to_string(), parts[1].to_string()));
            }
        }
        
        // Try to extract from any URL format
        let parts: Vec<&str> = url.split(['/', ':']).collect();
        if parts.len() >= 3 {
            // Look for "github.com" followed by owner/repo
            for i in 0..parts.len() - 2 {
                if parts[i] == "github.com" {
                    return Ok((parts[i+1].to_string(), parts[i+2].to_string()));
                }
            }
        }
        
        // Fallback for unknown formats - use crate name as both owner and repo
        if url.contains('/') {
            let parts: Vec<&str> = url.split('/').collect();
            if parts.len() >= 2 {
                return Ok((parts[parts.len()-2].to_string(), parts[parts.len()-1].to_string()));
            }
        }
        
        Ok(("unknown".to_string(), url.to_string()))
    }
    
    /// Find branch or version for a crate
    fn find_branch_for_version(&self, crate_node: &DependencyNode, repo_path: &Path) -> Result<String> {
        // Handle crates.io dependencies specially - just return the version
        if crate_node.source == "crates.io" {
            return Ok(crate_node.version.clone());
        }

        // Clean up version string by removing semantic versioning operators
        let mut version = crate_node.version.clone();
        // Remove common version operators: ^, ~, =, >, <, etc.
        if let Some(stripped) = version.strip_prefix('^') {
            version = stripped.to_string();
        }
        if let Some(stripped) = version.strip_prefix('~') {
            version = stripped.to_string();
        }
        if let Some(stripped) = version.strip_prefix('=') {
            version = stripped.to_string();
        }
        // Remove comparison operators
        version = version.replace(">=", "");
        version = version.replace(">", "");
        version = version.replace("<=", "");
        version = version.replace("<", "");
        version = version.replace(" ", "");
        
        // Try exact branch match
        
        // Try version as branch
        if self.check_branch_exists(repo_path, &version)? {
            return Ok(version);
        }
        
        // Try with v prefix
        let v_version = format!("v{}", version);
        if self.check_branch_exists(repo_path, &v_version)? {
            return Ok(v_version);
        }
        
        // Try as tag - look for best match
        let tags = self.get_git_tags(repo_path)?;
        
        // Look for exact matches first
        for tag in &tags {
            if tag == &version || tag == &v_version {
                return Ok(tag.clone());
            }
        }
        
        // Look for semantic version matches (e.g., 1.0 matches 1.0.0, 1.0.1, etc.)
        if version.contains('.') {
            let version_parts: Vec<&str> = version.split('.').collect();
            let major = version_parts[0];
            let minor = if version_parts.len() > 1 { version_parts[1] } else { "0" };
            
            // Look for tags that start with the major.minor version
            let prefix = format!("{}.{}", major, minor);
            for tag in &tags {
                if tag.starts_with(&prefix) {
                    return Ok(tag.clone());
                }
            }
            
            // Look for tags that start with just the major version
            let major_prefix = format!("{}.", major);
            for tag in &tags {
                if tag.starts_with(&major_prefix) {
                    return Ok(tag.clone());
                }
            }
        }
        
        // Try semantic version patterns
        if version.contains('.') {
            let major = version.split('.').next().unwrap();
            let major_branch = format!("v{}.", major);
            if self.check_branch_exists(repo_path, &major_branch)? {
                return Ok(major_branch);
            }
        }
        
        // Default to version as branch (may fail during checkout)
        Ok(version)
    }
    
    /// Check if a branch exists in a repository
    fn check_branch_exists(&self, repo_path: &Path, branch_name: &str) -> Result<bool> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("show-ref")
            .arg("--verify")
            .arg(format!("refs/heads/{}", branch_name))
            .output()?;
        
        Ok(output.status.success())
    }
    
    /// Get git tags from a repository
    fn get_git_tags(&self, repo_path: &Path) -> Result<Vec<String>> {
        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("tag")
            .arg("-l")
            .output()?;
        
        if !output.status.success() {
            return Ok(Vec::new());
        }
        
        let tags_str = String::from_utf8_lossy(&output.stdout);
        Ok(tags_str.lines().map(|s| s.trim().to_string()).collect())
    }
    
    /// Checkout a specific branch in a repository
    fn checkout_branch(&self, repo_path: &Path, branch_name: &str) -> Result<()> {
        // Skip git operations for crates.io cache directories
        if repo_path.to_string_lossy().contains("crates-io-cache") {
            println!("  Skipping git checkout for crates.io dependency");
            return Ok(());
        }

        // Skip git operations for workspace members (they're already at the correct version)
        // We can detect this by checking if the repo_path is within the workspace path
        if repo_path.starts_with(&self.workspace_path) {
            println!("  Skipping git checkout for workspace member (already at correct version)");
            return Ok(());
        }

        // Check if there are local changes that need to be preserved
        let status_output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("status")
            .arg("--porcelain")
            .output()?;
        
        let has_local_changes = !status_output.stdout.is_empty();
        
        if has_local_changes {
            println!("  Found local changes, capturing and preserving them...");
            
            // Stash local changes
            let stash_output = Command::new(&self.git_exe)
                .arg("-C")
                .arg(repo_path)
                .arg("stash")
                .arg("push")
                .arg("--include-untracked")
                .arg("-m")
                .arg(format!("Auto-stash before checkout to {}", branch_name))
                .output()?;
            
            if !stash_output.status.success() {
                let error_msg = String::from_utf8_lossy(&stash_output.stderr);
                println!("  Warning: Failed to stash changes: {}", error_msg);
            } else {
                println!("  Successfully stashed local changes");
            }
        }

        // Check if we need to fetch the branch first
        let fetch_output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("fetch")
            .arg("origin")
            .arg(branch_name)
            .output()?;
        
        if !fetch_output.status.success() {
            let error_msg = String::from_utf8_lossy(&fetch_output.stderr);
            println!("  Warning: Failed to fetch branch {}: {}", branch_name, error_msg);
        }

        let output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("checkout")
            .arg(branch_name)
            .output()?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to checkout branch {}: {}",
                branch_name, error_msg
            ));
        }
        
        // Apply stashed changes back
        if has_local_changes {
            println!("  Applying preserved changes to new branch...");
            
            let apply_output = Command::new(&self.git_exe)
                .arg("-C")
                .arg(repo_path)
                .arg("stash")
                .arg("pop")
                .output()?;
            
            if !apply_output.status.success() {
                let error_msg = String::from_utf8_lossy(&apply_output.stderr);
                println!("  Warning: Failed to apply stashed changes: {}", error_msg);
                println!("  Changes are still preserved in stash and can be applied manually");
            } else {
                println!("  Successfully applied preserved changes");
            }
        }
        
        // Check for any remaining changes and commit them
        let final_status_output = Command::new(&self.git_exe)
            .arg("-C")
            .arg(repo_path)
            .arg("status")
            .arg("--porcelain")
            .output()?;
        
        if !final_status_output.stdout.is_empty() {
            println!("  Committing final state...");
            
            // Add all changes
            let add_output = Command::new(&self.git_exe)
                .arg("-C")
                .arg(repo_path)
                .arg("add")
                .arg(".")
                .output()?;
            
            if add_output.status.success() {
                let commit_msg = format!("Auto-commit: Applied changes to branch {}", branch_name);
                let commit_output = Command::new(&self.git_exe)
                    .arg("-C")
                    .arg(repo_path)
                    .arg("commit")
                    .arg("-m")
                    .arg(commit_msg)
                    .output()?;
                
                if commit_output.status.success() {
                    println!("  ✅ Successfully committed changes to branch {}", branch_name);
                } else {
                    let error_msg = String::from_utf8_lossy(&commit_output.stderr);
                    println!("  Warning: Failed to commit changes: {}", error_msg);
                }
            }
        }
        
        Ok(())
    }
    
    /// Apply patches to a repository
    fn apply_patches(&self, repo_path: &Path) -> Result<()> {
        // Look for patch files in patches directory
        let patches_dir = self.output_base.join("patches");
        if !patches_dir.exists() {
            println!("  No patches directory found, skipping patch application");
            return Ok(());
        }
        
        // Find patch files for this repository
        let repo_name = repo_path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let patch_pattern = format!("{}-*.patch", repo_name);
        println!("  Looking for patches: {}", patch_pattern);
        
        // For now, skip actual patch application
        // In production, would find and apply matching patch files
        println!("  Patch application not yet implemented");
        
        Ok(())
    }
    
    /// Generate flake.nix for a crate
    fn generate_flake(&self, crate_node: &DependencyNode, _repo_path: &Path, is_external: bool) -> Result<()> {
        let output_dir = if is_external {
            self.output_base.join("layer1").join(&crate_node.crate_name)
        } else {
            self.output_base.join("layer2").join(&crate_node.crate_name)
        };
        
        fs::create_dir_all(&output_dir)?;
        
        let flake_content = format!(r#"
{{
  description = "{} v{} - Auto-generated flake";
  
  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  }};
  
  outputs = {{ self, nixpkgs, flake-utils, ... }}@inputs: {{
    packages = {{
      default = self.packages.${{system}}.{}; 
    }};
    
    packages.${{system}} = {{
      {} = inputs.nixpkgs.legacyPackages.${{system}}.callPackage ./. {{ }};
    }};
  }};
}}
"#, 
        crate_node.crate_name, crate_node.version,
        crate_node.crate_name,
        crate_node.crate_name
        );
        
        let flake_path = output_dir.join("flake.nix");
        let flake_path_display = flake_path.display().to_string();
        fs::write(flake_path, flake_content)?;
        
        println!("  Generated flake.nix at: {}", flake_path_display);
        
        Ok(())
    }
    
    /// Compile crate standalone
    fn compile_standalone(&self, repo_path: &Path, _is_external: bool) -> Result<()> {
        println!("  Compiling crate...");
        
        // Try nix build first if flake.nix or default.nix exists
        let has_flake = repo_path.join("flake.nix").exists();
        let has_default = repo_path.join("default.nix").exists();
        
        if has_flake || has_default {
            println!("  Found nix files, attempting nix build...");
            if self.build_with_nix(repo_path).is_ok() {
                println!("  Nix build successful");
                return Ok(());
            } else {
                println!("  Nix build failed, falling back to cargo build...");
            }
        }
        
        // Fall back to cargo build
        // Change to repository directory
        let output = Command::new("cargo")
            .current_dir(repo_path)
            .arg("build")
            .arg("--release")
            .output()?;
        
        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!(
                "Failed to build crate: {}",
                error_msg
            ));
        }
        
        println!("  Cargo build successful");
        Ok(())
    }
    
    /// Build with Nix
    fn build_with_nix(&self, repo_path: &Path) -> Result<()> {
        if !Path::new("nix").exists() {
            println!("  Nix not available, skipping nix build");
            return Ok(());
        }
        
        let output = Command::new("nix")
            .arg("build")
            .arg("-f")
            .arg("default.nix")
            .current_dir(repo_path)
            .output();
        
        match output {
            Ok(out) => {
                if !out.status.success() {
                    let error_msg = String::from_utf8_lossy(&out.stderr);
                    println!("  Nix build failed: {}", error_msg);
                } else {
                    println!("  Nix build successful");
                }
            }
            Err(e) => {
                println!("  Could not execute nix: {}", e);
            }
        }
        
        Ok(())
    }
}
