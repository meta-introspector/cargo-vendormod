use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use glob::glob;
use toml_edit::{Document, DocumentMut, Value};
use std::collections::HashMap;

use crate::repo_collection::{collect_repo_info, RepoIdentifier};
use crate::actions::create_actions_plan;
use crate::context::AppContext;

/// Discover ALL git repositories in the project (including submodules)
pub fn discover_all_git_repositories(project_path: &PathBuf) -> Result<Vec<PathBuf>> {
    let mut git_repos = Vec::new();
    
    // Find all .git directories using find command for better compatibility
    let output = std::process::Command::new("find")
        .arg(".")
        .arg("-name")
        .arg(".git")
        .arg("-type")
        .arg("d")
        .current_dir(project_path)  // Set working directory to project path
        .output()
        .context("Failed to execute find command")?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            let path = project_path.join(line.trim());
            if let Some(repo_path) = path.parent() {
                // Exclude the .git directory itself and the project root
                if repo_path != project_path && repo_path.file_name() != Some(std::ffi::OsStr::new(".git")) {
                    git_repos.push(repo_path.to_path_buf());
                }
            }
        }
        println!("Found {} git repositories using find command", git_repos.len());
    } else {
        eprintln!("Find command failed, falling back to glob pattern");
        // Fallback to glob pattern if find fails
        let pattern = format!("{}/**/.git", project_path.display());
        for entry in glob::glob(&pattern)? {
            if let Ok(path) = entry {
                if path.is_dir() {
                    // Get the parent directory (the actual repository)
                    if let Some(repo_path) = path.parent() {
                        // Exclude the .git directory itself
                        if repo_path != project_path && repo_path.file_name() != Some(std::ffi::OsStr::new(".git")) {
                            git_repos.push(repo_path.to_path_buf());
                        }
                    }
                }
            }
        }
        println!("Found {} git repositories using glob pattern", git_repos.len());
    }
    
    // Remove duplicates and sort
    git_repos.sort();
    git_repos.dedup();
    
    Ok(git_repos)
}

/// Process workspace members and extract git dependencies
pub fn cmd_workspace(ctx: &AppContext, workspace_path: &PathBuf, recursive: bool) -> Result<()> {
    println!("Processing workspace at: {}", workspace_path.display());
    
    // Find all Cargo.toml files in the workspace
    let cargo_files = find_workspace_cargo_files(workspace_path, recursive)?;
    println!("Found {} Cargo.toml files in workspace", cargo_files.len());
    
    // Collect all git dependencies from all workspace members
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
    
    println!("Collected {} unique git dependencies from workspace", all_repo_info.len());
    
    // Create actions plan for all dependencies
    let actions_plan = create_actions_plan(ctx, &all_repo_info);
    
    // Execute the vendoring process
    crate::vendoring::do_vendoring(ctx.clone(), actions_plan)?;
    
    // Create patches for all submodules
    crate::patch::cmd_patch(ctx.clone(), vec![])?;
    
    Ok(())
}

/// Find ALL Cargo.toml files in a project (including workspace members and all submodules)
pub fn find_workspace_cargo_files(workspace_path: &PathBuf, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut cargo_files = Vec::new();
    
    // First, check if this is a workspace by looking for workspace members
    let workspace_manifest = workspace_path.join("Cargo.toml");
    if workspace_manifest.exists() {
        let content = fs::read_to_string(&workspace_manifest)
            .context("Failed to read workspace Cargo.toml")?;
        let doc = content.parse::<DocumentMut>()
            .context("Failed to parse workspace Cargo.toml")?;
        
        // Check for workspace.members
        if let Some(ws) = doc.get("workspace") {
            if let Some(members) = ws.get("members") {
                if let Some(member_array) = members.as_array() {
                    for member in member_array {
                        if let Some(member_str) = member.as_str() {
                            let member_path = workspace_path.join(member_str);
                            let cargo_toml = if member_path.is_dir() {
                                member_path.join("Cargo.toml")
                            } else if member_path.ends_with("Cargo.toml") {
                                member_path
                            } else {
                                member_path.with_extension("toml")
                            };
                            
                            if cargo_toml.exists() {
                                cargo_files.push(cargo_toml);
                            }
                        }
                    }
                }
            }
        }
    }
    
    // Find ALL Cargo.toml files recursively in the entire project
    let pattern = format!("{}/**/Cargo.toml", workspace_path.display());
    for entry in glob(&pattern)? {
        if let Ok(path) = entry {
            if path.is_file() && !cargo_files.contains(&path) {
                cargo_files.push(path);
            }
        }
    }
    
    // Always include the root Cargo.toml if it exists
    let root_cargo = workspace_path.join("Cargo.toml");
    if root_cargo.exists() && !cargo_files.contains(&root_cargo) {
        cargo_files.push(root_cargo);
    }
    
    // Remove duplicates and sort
    cargo_files.sort();
    cargo_files.dedup();
    
    Ok(cargo_files)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_discover_git_repositories_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = discover_all_git_repositories(&temp_dir.path().to_path_buf());
        assert!(result.is_ok());
        let repos = result.unwrap();
        assert!(repos.is_empty());
    }

    #[test]
    fn test_find_workspace_cargo_files_nonexistent() {
        let temp_dir = TempDir::new().unwrap();
        let result = find_workspace_cargo_files(&temp_dir.path().to_path_buf(), false);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn test_find_workspace_cargo_files_with_cargo_toml() {
        let temp_dir = TempDir::new().unwrap();
        let cargo_toml = temp_dir.path().join("Cargo.toml");
        fs::write(&cargo_toml, "[package]\nname = \"test\"\nversion = \"0.1.0\"").unwrap();
        
        let result = find_workspace_cargo_files(&temp_dir.path().to_path_buf(), false);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0], cargo_toml);
    }

    #[test]
    fn test_find_workspace_cargo_files_nested() {
        let temp_dir = TempDir::new().unwrap();
        let nested_dir = temp_dir.path().join("nested");
        fs::create_dir(&nested_dir).unwrap();
        
        let cargo_toml1 = temp_dir.path().join("Cargo.toml");
        let cargo_toml2 = nested_dir.join("Cargo.toml");
        
        fs::write(&cargo_toml1, "[package]\nname = \"root\"\nversion = \"0.1.0\"").unwrap();
        fs::write(&cargo_toml2, "[package]\nname = \"nested\"\nversion = \"0.1.0\"").unwrap();
        
        let result = find_workspace_cargo_files(&temp_dir.path().to_path_buf(), true);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_find_workspace_cargo_files_workspace_members() {
        let temp_dir = TempDir::new().unwrap();
        let member_dir = temp_dir.path().join("member");
        fs::create_dir(&member_dir).unwrap();
        
        // Create workspace Cargo.toml
        let workspace_content = r#"
[workspace]
members = ["member"]

[package]
name = "workspace-test"
version = "0.1.0"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), workspace_content).unwrap();
        
        // Create member Cargo.toml
        fs::write(member_dir.join("Cargo.toml"), "[package]\nname = \"member\"\nversion = \"0.1.0\"").unwrap();
        
        let result = find_workspace_cargo_files(&temp_dir.path().to_path_buf(), false);
        assert!(result.is_ok());
        let files = result.unwrap();
        assert!(files.len() >= 1);
    }
}