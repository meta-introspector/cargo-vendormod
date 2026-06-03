//! # DASL Pipeline — Full Workflow Orchestrator
//!
//! ISO 9001 / ITIL-compliant pipeline for generating Nix flakes from DASL
//! Rust workspaces. This module provides three operations that form a
//! reproducible build pipeline:
//!
//! 1. **Generate lockfiles**: Scan workspaces under `dasl_data`, run
//!    `cargo generate-lockfile` for any that lack a `Cargo.lock`, then
//!    copy the generated locks to a lockfiles directory.
//!
//! 2. **Aggregate flake**: Read the list of discovered crate directories
//!    and generate a top-level `flake.nix` that imports every crate's
//!    `package.nix` with the correct source path and workspace
//!    `Cargo.lock` reference.
//!
//! 3. **Full pipeline**: (1) → (2) in sequence.
//!
//! ## Usage via processing binary
//!
//! ```text
//! # Generate lockfiles only
//! processing generate-lockfiles --dasl-data ~/dasl/data --output-dir build-dasl-flakes
//!
//! # Generate aggregate flake only
//! processing aggregate-flake --crate-list build-dasl-flakes/discovered_crates.txt \
//!   --output-dir build-dasl-flakes
//!
//! # Full pipeline
//! processing dasl-pipeline --dasl-data ~/dasl/data --output-dir build-dasl-flakes
//! ```

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;

// ── Constants ──────────────────────────────────────────────────────────────

/// Maximum search depth for Cargo.toml discovery under dasl_data.
const MAX_DISCOVERY_DEPTH: usize = 4;

// ── Lockfile generation ────────────────────────────────────────────────────

/// Result from generating lockfiles for a single workspace.
#[derive(Debug)]
pub struct LockfileResult {
    pub workspace_name: String,
    pub workspace_path: PathBuf,
    pub status: LockfileStatus,
}

#[derive(Debug)]
pub enum LockfileStatus {
    /// Cargo.lock already existed; copied to output dir
    AlreadyExists,
    /// Successfully generated via `cargo generate-lockfile`
    Generated,
    /// Skipped (no Cargo.toml or virtual manifest)
    Skipped,
    /// `cargo generate-lockfile` failed
    Failed(String),
}

/// Scan all Rust workspace directories under `dasl_data`, generate Cargo.lock
/// for any that lack one, and copy all locks to `lockfiles_dir`.
///
/// Returns a vector of per-workspace results for reporting.
///
/// # Arguments
///
/// * `dasl_data` — Root directory containing Rust workspace subdirectories
/// * `lockfiles_dir` — Output directory for Cargo.lock copies
/// * `verbose` — Enable verbose logging to stderr
/// * `max_depth` — Maximum search depth (default: 4)
pub fn generate_workspace_lockfiles(
    dasl_data: &Path,
    lockfiles_dir: &Path,
    verbose: bool,
) -> Result<Vec<LockfileResult>> {
    if !dasl_data.exists() {
        bail!("dasl_data directory does not exist: {}", dasl_data.display());
    }

    std::fs::create_dir_all(lockfiles_dir)
        .with_context(|| format!("Failed to create lockfiles dir: {}", lockfiles_dir.display()))?;

    let workspaces = discover_workspaces(dasl_data, MAX_DISCOVERY_DEPTH)?;
    if verbose {
        eprintln!("  Discovered {} potential workspace directories", workspaces.len());
    }

    let mut results = Vec::with_capacity(workspaces.len());

    for ws_dir in &workspaces {
        let ws_name = ws_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        let lock_path = ws_dir.join("Cargo.lock");

        // Ensure target directory exists
        let target_dir = lockfiles_dir.join(&ws_name);
        std::fs::create_dir_all(&target_dir)?;

        let result = if lock_path.exists() {
            // Copy existing lock
            std::fs::copy(&lock_path, target_dir.join("Cargo.lock"))?;
            if verbose {
                eprintln!("  [COPY]  {} — Cargo.lock already exists", ws_name);
            }
            LockfileResult {
                workspace_name: ws_name,
                workspace_path: ws_dir.clone(),
                status: LockfileStatus::AlreadyExists,
            }
        } else if !ws_dir.join("Cargo.toml").exists() {
            if verbose {
                eprintln!("  [SKIP]  {} — no Cargo.toml", ws_name);
            }
            LockfileResult {
                workspace_name: ws_name,
                workspace_path: ws_dir.clone(),
                status: LockfileStatus::Skipped,
            }
        } else {
            // Generate lockfile via `cargo generate-lockfile`
            if verbose {
                eprintln!("  [GEN]   {} — generating Cargo.lock", ws_name);
            }

            let cargo_toml = ws_dir.join("Cargo.toml");
            let output = Command::new("cargo")
                .args(["generate-lockfile", "--manifest-path"])
                .arg(&cargo_toml)
                .output()
                .with_context(|| format!("Failed to run cargo generate-lockfile for {}", ws_name))?;

            if output.status.success() && lock_path.exists() {
                std::fs::copy(&lock_path, target_dir.join("Cargo.lock"))?;
                LockfileResult {
                    workspace_name: ws_name,
                    workspace_path: ws_dir.clone(),
                    status: LockfileStatus::Generated,
                }
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                LockfileResult {
                    workspace_name: ws_name,
                    workspace_path: ws_dir.clone(),
                    status: LockfileStatus::Failed(stderr),
                }
            }
        };

        results.push(result);
    }

    Ok(results)
}

// ── Workspace discovery ────────────────────────────────────────────────────

/// Discover Rust workspace directories under `root`.
///
/// Looks for `Cargo.toml` files up to `max_depth` deep. Returns deduplicated
/// sorted list of parent directories.
fn discover_workspaces(root: &Path, max_depth: usize) -> Result<Vec<PathBuf>> {
    let mut workspaces = Vec::new();

    // Walk up to max_depth levels
    let walk = walkdir::WalkDir::new(root)
        .max_depth(max_depth)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "Cargo.toml");

    for entry in walk {
        if let Some(parent) = entry.path().parent() {
            workspaces.push(parent.to_path_buf());
        }
    }

    // Deduplicate by canonical path
    workspaces.sort();
    workspaces.dedup();

    Ok(workspaces)
}

// ── Aggregate flake generation ─────────────────────────────────────────────

/// A single crate entry for the aggregate flake.
#[derive(Debug)]
pub struct CrateEntry {
    pub name: String,
    pub source_path: PathBuf,
    pub lock_path: Option<PathBuf>,
    pub version: String,
}

/// Generate a top-level `flake.nix` that aggregates all crate flakes.
///
/// Reads the crate list from `crate_list_path` (one crate directory per line),
/// resolves each to its `Cargo.toml` for metadata, and writes a Nix expression
/// at `output_path` that exposes every crate as a package.
///
/// # Arguments
///
/// * `crate_list_path` — Path to a file containing crate dir paths (one per line)
/// * `output_path` — Where to write the aggregate `flake.nix`
/// * `verbose` — Enable verbose logging to stderr
pub fn generate_aggregate_flake(
    crate_list_path: &Path,
    output_path: &Path,
    verbose: bool,
) -> Result<()> {
    if !crate_list_path.exists() {
        bail!("Crate list file does not exist: {}", crate_list_path.display());
    }

    let content = std::fs::read_to_string(crate_list_path)
        .with_context(|| format!("Failed to read crate list: {}", crate_list_path.display()))?;

    let crate_paths: Vec<PathBuf> = content
        .lines()
        .map(PathBuf::from)
        .filter(|p| p.exists() && p.join("Cargo.toml").exists())
        .collect();

    if crate_paths.is_empty() {
        bail!("No valid crate directories found in crate list");
    }

    if verbose {
        eprintln!("  Resolving {} crate entries...", crate_paths.len());
    }

    // Resolve each crate to its metadata
    let entries = resolve_crate_entries(&crate_paths, verbose)?;

    if verbose {
        eprintln!("  Resolved {} crates for aggregate flake", entries.len());
    }

    // Generate the flake.nix content
    let flake_content = render_aggregate_flake(&entries)?;

    // Write output
    if let Some(parent) = output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(output_path, &flake_content)
        .with_context(|| format!("Failed to write aggregate flake: {}", output_path.display()))?;

    if verbose {
        eprintln!("  Wrote aggregate flake to: {}", output_path.display());
    }

    Ok(())
}

/// Resolve crate directory paths to `CrateEntry` with name, version, source, lock.
fn resolve_crate_entries(crate_paths: &[PathBuf], verbose: bool) -> Result<Vec<CrateEntry>> {
    let mut entries = Vec::with_capacity(crate_paths.len());

    for crate_path in crate_paths {
        let cargo_toml_path = crate_path.join("Cargo.toml");

        // Parse Cargo.toml for package name and version
        let toml_raw = match std::fs::read_to_string(&cargo_toml_path) {
            Ok(s) => s,
            Err(e) => {
                if verbose {
                    eprintln!("  Warning: cannot read {}: {}", cargo_toml_path.display(), e);
                }
                continue;
            }
        };

        let cargo_toml: serde_json::Value = match toml::de::from_str(&toml_raw) {
            Ok(v) => v,
            Err(e) => {
                if verbose {
                    eprintln!("  Warning: cannot parse {}: {}", cargo_toml_path.display(), e);
                }
                continue;
            }
        };

        // Extract package name
        let pkg_name = cargo_toml
            .get("package")
            .and_then(|p| p.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                // Use directory name as fallback
                crate_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string()
            });

        // Extract version
        let pkg_version = cargo_toml
            .get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0")
            .to_string();

        // Find Cargo.lock — check crate dir, then walk up
        let lock_path = find_lock_upward(crate_path);

        // Find source path — the directory containing the Cargo.toml
        let source_path = if let Some(parent) = cargo_toml_path.parent() {
            parent.to_path_buf()
        } else {
            crate_path.clone()
        };

        entries.push(CrateEntry {
            name: pkg_name,
            source_path,
            lock_path,
            version: pkg_version,
        });
    }

    Ok(entries)
}

/// Search upward from `dir` to find the nearest Cargo.lock.
fn find_lock_upward(dir: &Path) -> Option<PathBuf> {
    let mut current = Some(dir);
    while let Some(d) = current {
        let lock = d.join("Cargo.lock");
        if lock.exists() {
            return Some(lock);
        }
        current = d.parent();
    }
    None
}

// ── Aggregate flake rendering ──────────────────────────────────────────────

/// Render the aggregate flake.nix content.
fn render_aggregate_flake(entries: &[CrateEntry]) -> Result<String> {
    let mut lines = Vec::new();

    // ── Header ──
    lines.push("# THIS FILE IS AUTO-GENERATED by cargo-vendormod dasl-pipeline".to_string());
    lines.push("# DO NOT EDIT MANUALLY — regenerate with:".to_string());
    lines.push("#   processing dasl-pipeline --dasl-data <path> --output-dir <dir>".to_string());
    lines.push(String::new());
    lines.push("{".to_string());
    lines.push("  description = \"DASL Crates — aggregated Rust packages from the DASL codebase\";".to_string());
    lines.push(String::new());

    // ── Inputs ──
    lines.push("  inputs = {".to_string());
    lines.push("    nixpkgs.url = \"github:NixOS/nixpkgs/nixpkgs-unstable\";".to_string());
    lines.push("    flake-utils.url = \"github:numtide/flake-utils\";".to_string());
    lines.push("  };".to_string());
    lines.push(String::new());

    // ── Outputs ──
    lines.push("  outputs = { self, nixpkgs, flake-utils }:" .to_string());
    lines.push("    flake-utils.lib.eachDefaultSystem (system:".to_string());
    lines.push("      let".to_string());
    lines.push("        pkgs = import nixpkgs { inherit system; };".to_string());
    lines.push("      in {".to_string());
    lines.push("        packages = {".to_string());

    // ── Per-crate packages ──
    for entry in entries {
        let safe_name = &entry.name;
        let src_path = entry.source_path.display();
        let version = &entry.version;

        match &entry.lock_path {
            Some(lock) => {
                let lock_path = lock.display();
                lines.push(format!(
                    "          {} = pkgs.rustPlatform.buildRustPackage {{",
                    safe_name
                ));
                lines.push(format!("            pname = \"{}\";", safe_name));
                lines.push(format!("            version = \"{}\";", version));
                lines.push(format!(
                    "            src = builtins.path {{ path = {}; name = \"{}-src\"; }};",
                    src_path, safe_name
                ));
                lines.push(format!(
                    "            cargoLock.lockFile = builtins.path {{ path = {}; name = \"{}-lock\"; }};",
                    lock_path, safe_name
                ));
                lines.push("          };".to_string());
            }
            None => {
                lines.push(format!(
                    "          {} = pkgs.rustPlatform.buildRustPackage {{",
                    safe_name
                ));
                lines.push(format!("            pname = \"{}\";", safe_name));
                lines.push(format!("            version = \"{}\";", version));
                lines.push(format!(
                    "            src = builtins.path {{ path = {}; name = \"{}-src\"; }};",
                    src_path, safe_name
                ));
                lines.push("            # cargoLock.lockFile = ...;  # no Cargo.lock found — add manually".to_string());
                lines.push("          };".to_string());
            }
        }
    }

    // ── Footer ──
    lines.push("        };".to_string()); // close packages
    lines.push("        devShells.default = pkgs.mkShell {".to_string());
    lines.push("          buildInputs = with pkgs; [ rustc cargo clippy rustfmt ];".to_string());
    lines.push("        };".to_string());
    lines.push("      });".to_string()); // close eachDefaultSystem
    lines.push("}".to_string()); // close outputs

    Ok(lines.join("\n") + "\n")
}

// ── Reporting helpers ──────────────────────────────────────────────────────

/// Format lockfile generation results for display.
pub fn format_lockfile_summary(results: &[LockfileResult]) -> String {
    let total = results.len();
    let generated = results.iter().filter(|r| matches!(r.status, LockfileStatus::Generated)).count();
    let existing = results.iter().filter(|r| matches!(r.status, LockfileStatus::AlreadyExists)).count();
    let skipped = results.iter().filter(|r| matches!(r.status, LockfileStatus::Skipped)).count();
    let failed: Vec<&LockfileResult> = results.iter().filter(|r| matches!(r.status, LockfileStatus::Failed(_))).collect();

    let mut out = format!(
        "Lockfile generation summary:\n\
         \x20 Total workspaces: {}\n\
         \x20 Generated:        {}\n\
         \x20 Already existed:  {}\n\
         \x20 Skipped:          {}\n\
         \x20 Failed:           {}\n",
        total, generated, existing, skipped, failed.len()
    );

    if !failed.is_empty() {
        out.push_str("\nFailed workspaces:\n");
        for r in &failed {
            if let LockfileStatus::Failed(ref msg) = r.status {
                out.push_str(&format!("  - {}: {}\n", r.workspace_name, msg));
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_discover_workspaces() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("ws1")).unwrap();
        fs::write(tmp.path().join("ws1").join("Cargo.toml"), "[package]\nname=\"ws1\"\n").unwrap();
        fs::create_dir_all(tmp.path().join("sub/ws2")).unwrap();
        fs::write(tmp.path().join("sub/ws2").join("Cargo.toml"), "[package]\nname=\"ws2\"\n").unwrap();

        let workspaces = discover_workspaces(tmp.path(), 3).unwrap();
        assert!(workspaces.iter().any(|p| p.ends_with("ws1")));
        assert!(workspaces.iter().any(|p| p.ends_with("ws2")));
    }

    #[test]
    fn test_render_aggregate_flake() {
        let entries = vec![
            CrateEntry {
                name: "crate-a".to_string(),
                source_path: PathBuf::from("/src/a"),
                lock_path: Some(PathBuf::from("/src/a/Cargo.lock")),
                version: "1.0.0".to_string(),
            },
            CrateEntry {
                name: "crate-b".to_string(),
                source_path: PathBuf::from("/src/b"),
                lock_path: None,
                version: "0.5.0".to_string(),
            },
        ];

        let result = render_aggregate_flake(&entries).unwrap();
        assert!(result.contains("crate-a"));
        assert!(result.contains("crate-b"));
        assert!(result.contains("Cargo.lock"));
        assert!(result.contains("no Cargo.lock found"));
    }

    #[test]
    fn test_resolve_crate_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let crate_dir = tmp.path().join("test-crate");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            "[package]\nname = \"test-crate\"\nversion = \"0.1.0\"\n",
        ).unwrap();
        fs::write(crate_dir.join("Cargo.lock"), "# dummy").unwrap();

        let entries = resolve_crate_entries(&[crate_dir.clone()], false).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "test-crate");
        assert_eq!(entries[0].version, "0.1.0");
        assert!(entries[0].lock_path.is_some());
    }
}
