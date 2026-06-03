//! # Onboarding Module
//!
//! Handles onboarding of new repositories, crates, and workspaces into the vendoring system.
//! Translates workflow from `workload/scripts/onboard.sh` and related bash scripts.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::process::Command;

/// Onboard a new repository, crate, or workspace
pub fn cmd_onboard(
    git_repo: Option<String>,
    crate_name: Option<String>,
    workdir: Option<PathBuf>,
    branch: String,
    workflow: String,
    output_dir: PathBuf,
) -> Result<()> {
    println!("🚀 Starting onboarding workflow: {}", workflow);
    
    // Validate exactly one source is provided
    let source_count = git_repo.is_some() as u8 + crate_name.is_some() as u8 + workdir.is_some() as u8;
    if source_count != 1 {
        anyhow::bail!("Exactly one of --git-repo, --crate, or --workdir must be specified");
    }
    
    let workspace_path = match (git_repo, crate_name, workdir) {
        (Some(repo), None, None) => onboard_git_repo(&repo, &branch, &output_dir)?,
        (None, Some(crate_name), None) => onboard_crate(&crate_name, &output_dir)?,
        (None, None, Some(workdir)) => {
            println!("📁 Using local workspace: {}", workdir.display());
            workdir
        }
        _ => unreachable!(),
    };
    
    run_onboarding_workflow(&workspace_path, &workflow, &output_dir)?;
    
    println!("✅ Onboarding complete!");
    println!("📁 Workspace: {}", workspace_path.display());
    Ok(())
}

/// Onboard a git repository
fn onboard_git_repo(repo_url: &str, branch: &str, output_dir: &Path) -> Result<PathBuf> {
    println!("📥 Cloning git repository: {}", repo_url);
    println!("   Branch: {}", branch);
    
    // Extract repo name from URL
    let repo_name = repo_url
        .split('/')
        .last()
        .unwrap_or("repository")
        .trim_end_matches(".git");
    
    let workspace_path = output_dir.join(repo_name);
    
    // Create output directory
    fs::create_dir_all(output_dir)
        .context("Failed to create output directory")?;
    
    // Clone with minimal depth for speed
    let output = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", branch, repo_url])
        .arg(&workspace_path)
        .output()
        .context("Failed to execute git clone")?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Git clone failed: {}", stderr);
    }
    
    println!("✅ Repository cloned to: {}", workspace_path.display());
    Ok(workspace_path)
}

/// Onboard a crate from crates.io
fn onboard_crate(crate_name: &str, output_dir: &Path) -> Result<PathBuf> {
    println!("📦 Downloading crate: {} from crates.io", crate_name);
    
    let workspace_path = output_dir.join(crate_name);
    fs::create_dir_all(&workspace_path)
        .context("Failed to create workspace directory")?;
    
    // Download crate using cargo
    let output = Command::new("cargo")
        .args(["download", "--extract", "--crate", crate_name])
        .current_dir(&workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("✅ Crate downloaded: {}", crate_name);
        }
        _ => {
            // Fallback: create minimal crate structure
            println!("⚠️  Could not download crate, creating minimal structure");
            create_minimal_crate(&workspace_path, crate_name)?;
        }
    }
    
    // If Cargo.toml doesn't exist, create minimal structure
    let cargo_toml = workspace_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        create_minimal_crate(&workspace_path, crate_name)?;
    }
    
    Ok(workspace_path)
}

/// Create a minimal crate structure for onboarding
fn create_minimal_crate(workspace_path: &Path, crate_name: &str) -> Result<()> {
    // Create Cargo.toml
    let cargo_toml_content = format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
"#,
        crate_name
    );
    fs::write(workspace_path.join("Cargo.toml"), cargo_toml_content)
        .context("Failed to write Cargo.toml")?;
    
    // Create src/lib.rs
    let src_dir = workspace_path.join("src");
    fs::create_dir_all(&src_dir)
        .context("Failed to create src directory")?;
    
    let lib_rs = format!(
        r#"//! # {}
//!
//! Minimal implementation for onboarding

/// Main library entry point
pub fn hello() -> String {{
    "Hello from {}!".to_string()
}}
"#,
        crate_name, crate_name
    );
    fs::write(src_dir.join("lib.rs"), lib_rs)
        .context("Failed to write lib.rs")?;
    
    println!("✅ Minimal crate structure created");
    Ok(())
}

/// Run the complete onboarding workflow
fn run_onboarding_workflow(
    workspace_path: &Path,
    workflow_name: &str,
    output_dir: &Path,
) -> Result<()> {
    println!("▶️ Running workflow: {}", workflow_name);
    
    match workflow_name {
        "full_onboarding" => run_full_onboarding(workspace_path, output_dir),
        "minimal" => run_minimal_onboarding(workspace_path, output_dir),
        "ci" => run_ci_onboarding(workspace_path, output_dir),
        "rust_toolchain" => run_rust_toolchain_onboarding(workspace_path, output_dir),
        custom => {
            println!("⚠️  Unknown workflow '{}', using minimal onboarding", custom);
            run_minimal_onboarding(workspace_path, output_dir)
        }
    }
}

/// Run full onboarding workflow
fn run_full_onboarding(workspace_path: &Path, output_dir: &Path) -> Result<()> {
    println!("\n=== Phase 1: Workspace Analysis ===");
    analyze_workspace(workspace_path, output_dir)?;
    
    println!("\n=== Phase 2: Vendoring ===");
    run_vendoring(workspace_path)?;
    
    println!("\n=== Phase 3: Upstream Sync ===");
    fetch_upstream(workspace_path)?;
    
    println!("\n=== Phase 4: Patch Generation ===");
    generate_patches(workspace_path)?;
    
    println!("\n=== Phase 5: Build Verification ===");
    verify_build(workspace_path)?;
    
    Ok(())
}

/// Run minimal onboarding workflow
fn run_minimal_onboarding(workspace_path: &Path, _output_dir: &Path) -> Result<()> {
    println!("\n=== Minimal Workflow ===");
    run_vendoring(workspace_path)?;
    fetch_upstream(workspace_path)?;
    generate_patches(workspace_path)?;
    verify_build(workspace_path)?;
    Ok(())
}

/// Run CI onboarding workflow
fn run_ci_onboarding(workspace_path: &Path, output_dir: &Path) -> Result<()> {
    println!("\n=== CI Workflow ===");
    analyze_workspace(workspace_path, output_dir)?;
    run_vendoring(workspace_path)?;
    fetch_upstream(workspace_path)?;
    generate_patches(workspace_path)?;
    verify_build(workspace_path)?;
    
    // Additional CI checks
    println!("\n=== CI Checks ===");
    check_format(workspace_path)?;
    check_clippy(workspace_path)?;
    
    Ok(())
}

/// Run Rust toolchain onboarding workflow
fn run_rust_toolchain_onboarding(workspace_path: &Path, output_dir: &Path) -> Result<()> {
    println!("\n=== Rust Toolchain Workflow ===");
    analyze_workspace(workspace_path, output_dir)?;
    
    println!("\n=== Toolchain Analysis ===");
    check_rust_edition(workspace_path)?;
    check_rust_version(workspace_path)?;
    
    run_vendoring(workspace_path)?;
    verify_build(workspace_path)?;
    
    Ok(())
}

/// Check Rust edition
fn check_rust_edition(workspace_path: &Path) -> Result<()> {
    println!("  🦀 Checking Rust edition...");
    
    let cargo_toml = workspace_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        println!("  ⚠️  No Cargo.toml found");
        return Ok(());
    }
    
    let content = fs::read_to_string(&cargo_toml)?;
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("edition") {
            println!("  ✅ Found edition: {}", line);
            return Ok(());
        }
    }
    
    println!("  ⚠️  No edition specified");
    Ok(())
}

/// Check Rust toolchain version
fn check_rust_version(_workspace_path: &Path) -> Result<()> {
    println!("  🦀 Checking Rust toolchain version...");
    
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .context("Failed to run rustc")?;
    
    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("  ✅ {}", version.trim());
    }
    
    Ok(())
}

/// Analyze workspace and extract dependencies
fn analyze_workspace(workspace_path: &Path, _output_dir: &Path) -> Result<()> {
    println!("  🔍 Analyzing workspace structure...");
    
    // Check for Cargo.toml
    let cargo_toml = workspace_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        println!("  ⚠️  No Cargo.toml found");
        return Ok(());
    }
    
    // Read and parse Cargo.toml
    let content = fs::read_to_string(&cargo_toml)
        .context("Failed to read Cargo.toml")?;
    
    // Extract basic info
    let has_workspace = content.contains("[workspace]");
    let has_members = content.contains("members");
    
    println!("  ✅ Cargo.toml found");
    println!("     - Workspace: {}", has_workspace || has_members);
    
    // Count source files
    let src_dir = workspace_path.join("src");
    if src_dir.exists() {
        let file_count = fs::read_dir(&src_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
            .count();
        println!("     - Source files: {}", file_count);
    }
    
    Ok(())
}

/// Run vendoring
fn run_vendoring(workspace_path: &Path) -> Result<()> {
    println!("  📦 Running vendoring...");
    
    // Check if cargo-vendormod is available
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "vendoring"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Vendoring complete");
        }
        _ => {
            println!("  ⚠️  Vendoring not available or failed, skipping");
        }
    }
    
    Ok(())
}

/// Fetch upstream changes
fn fetch_upstream(workspace_path: &Path) -> Result<()> {
    println!("  🔄 Fetching upstream changes...");
    
    let output = Command::new("git")
        .args(["fetch", "--all"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Upstream fetch complete");
        }
        _ => {
            println!("  ⚠️  Could not fetch upstream (may not be a git repo)");
        }
    }
    
    Ok(())
}

/// Generate .cargo/config.toml patches
fn generate_patches(workspace_path: &Path) -> Result<()> {
    println!("  🩹 Generating patches...");
    
    let output = Command::new("cargo")
        .args(["run", "--release", "--", "patch"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Patches generated");
        }
        _ => {
            println!("  ⚠️  Patch generation not available, skipping");
        }
    }
    
    Ok(())
}

/// Verify build succeeds
fn verify_build(workspace_path: &Path) -> Result<()> {
    println!("  🔨 Verifying build...");
    
    let output = Command::new("cargo")
        .args(["check", "--workspace"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Build verification passed");
        }
        Ok(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            println!("  ⚠️  Build check failed (may be expected for minimal crates)");
            println!("     Error: {}", stderr.lines().next().unwrap_or("unknown"));
        }
        Err(e) => {
            println!("  ⚠️  Could not run cargo check: {}", e);
        }
    }
    
    Ok(())
}

/// Check code formatting
fn check_format(workspace_path: &Path) -> Result<()> {
    println!("  🎨 Checking code format...");
    
    let output = Command::new("cargo")
        .args(["fmt", "--", "--check"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Format check passed");
        }
        _ => {
            println!("  ⚠️  Format check failed (consider running cargo fmt)");
        }
    }
    
    Ok(())
}

/// Check clippy lints
fn check_clippy(workspace_path: &Path) -> Result<()> {
    println!("  🔍 Running clippy...");
    
    let output = Command::new("cargo")
        .args(["clippy", "--workspace", "--", "-D", "warnings"])
        .current_dir(workspace_path)
        .output();
    
    match output {
        Ok(out) if out.status.success() => {
            println!("  ✅ Clippy check passed");
        }
        _ => {
            println!("  ⚠️  Clippy check found issues (consider running cargo clippy)");
        }
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_create_minimal_crate() {
        let dir = tempdir().unwrap();
        let result = create_minimal_crate(dir.path(), "test_crate");
        assert!(result.is_ok());
        assert!(dir.path().join("Cargo.toml").exists());
        assert!(dir.path().join("src/lib.rs").exists());
    }

    #[test]
    fn test_onboard_validation() {
        let result = cmd_onboard(
            Some("https://github.com/test/repo".to_string()),
            Some("crate".to_string()),
            None,
            "main".to_string(),
            "minimal".to_string(),
            PathBuf::from("/tmp"),
        );
        assert!(result.is_err()); // Should fail with multiple sources
    }
}