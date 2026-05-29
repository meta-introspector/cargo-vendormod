//! # Process DASL Index — Scan index files and apply crate2nix onboarding
//!
//! Reads all `.txt` files in `~/dasl/index/`, parses the various formats to
//! extract file paths, deduplicates, and runs the full crate2nix onboarding
//! pipeline on each Cargo project found.
//!
//! ## Supported Index Formats
//!
//! | File | Format | Example |
//! |---|---|---|
//! | `cargo_toml.txt` | `source_file:path/to/Cargo.toml` | `all_test2.txt:/home/.../Cargo.toml` |
//! | `repos2.txt` | `relative/path/to/repo/` | `IMPL/users/atproto/atproto_repos/repo/` |
//! | `full_list.txt` | `/abs/path origin <url> (push)` | Git remote format |
//! | `gitconfig.txt` | `source_file:path/.git/config` | `all_test2.txt:/home/.../.git/config` |
//! | `cbor.txt` | `path:search match` | `./atproto_repos/.../Cargo.toml:name = "cbor"` |
//! | `cfiles.txt` | `path/to/file.c` | `./atproto_repos/.../file.c` |
//! | `untracked.txt` | `path/` | `atproto_repos/repo_name/` |
//!
//! ## Usage
//!
//! ```bash
//! cargo run --release --bin process_dasl_index -- \
//!   --index-dir ~/dasl/index \
//!   --dasl-root ~/dasl \
//!   --output-dir /mnt/data1/dasl-out \
//!   [--generate-lockfiles] \
//!   [--generate-cargo-nix] \
//!   [--generate-flakes] \
//!   [--build] \
//!   [--verbose]
//! ```

use anyhow::{bail, Context, Result};

/// Expand a leading ~/ or ~ in a path to the user's home directory.
fn expand_tilde(path: &str) -> std::path::PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    } else if path == "~" {
        if let Some(home) = dirs::home_dir() {
            return home;
        }
    }
    std::path::PathBuf::from(path)
}
use std::collections::{BTreeSet, HashMap};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

// ── CLI Arguments ──────────────────────────────────────────────────────────

#[derive(clap::Parser, Debug)]
#[command(name = "process_dasl_index", about = "Process DASL index files with crate2nix onboarding")]
struct Args {
    /// Directory containing index .txt files
    #[arg(long, default_value = "~/dasl/index")]
    index_dir: String,

    /// Root of the DASL source tree (for resolving relative paths)
    #[arg(long, default_value = "~/dasl")]
    dasl_root: String,

    /// Output directory for generated files per project
    #[arg(long, default_value = "/mnt/data1/dasl-out")]
    output_dir: String,

    /// Generate Cargo.lock files where missing
    #[arg(long)]
    generate_lockfiles: bool,

    /// Run crate2nix generate on Cargo projects
    #[arg(long)]
    generate_cargo_nix: bool,

    /// Generate flake.nix for each project
    #[arg(long)]
    generate_flakes: bool,

    /// Build projects via nix build
    #[arg(long)]
    build: bool,

    /// Limit to first N projects (for testing)
    #[arg(long)]
    limit: Option<usize>,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Skip projects inside .cargo/registry/ (crates.io copies)
    #[arg(long)]
    skip_registry: bool,

    /// Path to crate2nix binary (default: "crate2nix" on PATH)
    #[arg(long, default_value = "crate2nix")]
    crate2nix_path: String,

    /// Path to cargo binary (default: "cargo" on PATH)
    #[arg(long, default_value = "cargo")]
    cargo_path: String,

    /// Path to nix binary (default: "nix" on PATH, used for --build)
    #[arg(long, default_value = "nix")]
    nix_path: String,

    /// Number of parallel workers
    #[arg(long, default_value = "4")]
    workers: usize,
}

// ── Index Parser ───────────────────────────────────────────────────────────

/// A parsed entry from any index file format.
#[derive(Debug, Clone)]
enum IndexEntry {
    /// A Cargo.toml file path
    CargoToml(PathBuf),
    /// A git repository directory
    GitRepo(PathBuf),
    /// A .git/config file path
    GitConfig(PathBuf),
    /// A C source file
    CFile(PathBuf),
    /// A CBOR-related file
    CborRelated(PathBuf),
    /// An untracked repo directory
    Untracked(PathBuf),
}

/// Parse a single line from any index file format, returning detected entries.
fn parse_index_line(line: &str, dasl_root: &Path) -> Vec<IndexEntry> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Vec::new();
    }

    let mut entries = Vec::new();

    // Format 1: `source_file:/absolute/path` (cargo_toml.txt, gitconfig.txt style)
    if let Some((_source, path_part)) = line.split_once(':') {
        let path = path_part.trim();
        if path.ends_with("Cargo.toml") {
            entries.push(IndexEntry::CargoToml(PathBuf::from(path)));
        } else if path.ends_with(".git/config") {
            // Extract the repo dir from the .git/config path
            entries.push(IndexEntry::GitConfig(PathBuf::from(path)));
            if let Some(repo_path) = path.strip_suffix("/.git/config")
                .or_else(|| path.strip_suffix("/.git/config"))
            {
                if repo_path.contains('/') || repo_path.contains('.') {
                    entries.push(IndexEntry::GitRepo(PathBuf::from(repo_path)));
                }
            }
        }
        return entries;
    }

    // Format 2: `/absolute/path origin <url> (push)` (full_list.txt)
    if let Some((path, _rest)) = line.split_once(" origin ") {
        let p = PathBuf::from(path.trim());
        if p.is_absolute() {
            entries.push(IndexEntry::GitRepo(p));
            return entries;
        }
    }

    // Format 3: `./relative/path/to/repo/` (repos2.txt, untracked.txt)
    if line.ends_with('/') || line.chars().filter(|&c| c == '/').count() >= 2 {
        let path_str = if line.starts_with("./") {
            &line[2..]
        } else {
            line
        };
        let path = if Path::new(path_str).is_absolute() {
            PathBuf::from(path_str)
        } else {
            dasl_root.join(path_str)
        };
        if path_str.ends_with('/') {
            entries.push(IndexEntry::GitRepo(path));
        } else {
            entries.push(IndexEntry::Untracked(path));
        }
        return entries;
    }

    // Format 4: `./path:search match` (cbor.txt)
    if let Some((path, _rest)) = line.split_once(':') {
        let path = path.trim();
        if path.ends_with("Cargo.toml") || path.ends_with('"') || path.ends_with('.') {
            // Could be a Cargo.toml reference
            let full_path = if Path::new(path).is_absolute() {
                PathBuf::from(path)
            } else {
                dasl_root.join(path.trim_start_matches("./"))
            };
            if path.ends_with("Cargo.toml") {
                entries.push(IndexEntry::CargoToml(full_path));
            } else {
                entries.push(IndexEntry::CborRelated(full_path));
            }
        }
        return entries;
    }

    // Format 5: `.//path` or `./path.c` (cfiles.txt, raw paths)
    if line.ends_with(".c") || line.ends_with(".rs") {
        let path = if Path::new(line).is_absolute() {
            PathBuf::from(line)
        } else {
            dasl_root.join(line.trim_start_matches("./"))
        };
        entries.push(IndexEntry::CFile(path));
    }

    entries
}

/// Scan all index files in a directory and collect unique Cargo projects.
fn scan_index_files(index_dir: &Path, dasl_root: &Path, limit: Option<usize>, verbose: bool) -> Result<Vec<PathBuf>> {
    let mut cargo_projects: BTreeSet<PathBuf> = BTreeSet::new();
    let mut repos: BTreeSet<PathBuf> = BTreeSet::new();

    let mut file_count = 0usize;
    let mut line_count = 0usize;

    for entry in std::fs::read_dir(index_dir).context("Failed to read index directory")? {
        let entry = entry?;
        let path = entry.path();

        if path.extension().map_or(false, |e| e == "txt" || e == "json") {
            if path.file_name().and_then(|n| n.to_str()) == Some("url_scan_report.json") {
                continue; // Skip JSON report
            }

            file_count += 1;
            let file = std::fs::File::open(&path)
                .with_context(|| format!("Failed to open {}", path.display()))?;
            let reader = BufReader::new(file);

            let mut local_lines = 0usize;
            for line_result in reader.lines() {
                let line = line_result?;
                local_lines += 1;
                let entries = parse_index_line(&line, dasl_root);

                for ie in entries {
                    match ie {
                        IndexEntry::CargoToml(p) => {
                            if let Some(parent) = p.parent() {
                                cargo_projects.insert(parent.to_path_buf());
                            }
                        }
                        IndexEntry::GitRepo(p) => {
                            repos.insert(p);
                        }
                        _ => {}
                    }

                    if let Some(l) = limit {
                        if cargo_projects.len() >= l {
                            break;
                        }
                    }
                }
            }
            line_count += local_lines;

            if verbose {
                eprintln!("  {}: {} lines, {} repos, {} Cargo projects",
                    path.file_name().unwrap_or_default().to_string_lossy(),
                    local_lines,
                    repos.len(),
                    cargo_projects.len(),
                );
            }

            if let Some(l) = limit {
                if cargo_projects.len() >= l {
                    break;
                }
            }
        }
    }

    // Also scan repos for Cargo.toml files we might have missed
    for repo in &repos {
        scan_for_cargo(repo, &mut cargo_projects, 3);
        if let Some(l) = limit {
            if cargo_projects.len() >= l {
                break;
            }
        }
    }

    let projects: Vec<PathBuf> = cargo_projects.into_iter().collect();
    if verbose {
        eprintln!("\n  Scanned {} files, {} lines total", file_count, line_count);
        eprintln!("  Found {} unique Cargo projects", projects.len());
    }

    Ok(projects)
}

/// Recursively scan a directory for Cargo.toml files up to `depth` levels.
fn scan_for_cargo(dir: &Path, projects: &mut BTreeSet<PathBuf>, depth: usize) {
    if depth == 0 || !dir.is_dir() {
        return;
    }

    let cargo_toml = dir.join("Cargo.toml");
    if cargo_toml.exists() {
        projects.insert(dir.to_path_buf());
        return; // Don't recurse into workspace members — crate2nix handles them
    }

    if let Ok(read_dir) = std::fs::read_dir(dir) {
        for entry in read_dir.flatten() {
            let path = entry.path();
            if path.is_dir() && !path.is_symlink() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "target" && name != "node_modules" {
                    scan_for_cargo(&path, projects, depth - 1);
                }
            }
        }
    }
}

// ── Onboarding Pipeline Steps ──────────────────────────────────────────────

/// Statistics tracker
#[derive(Debug, Default)]
struct PipelineStats {
    total: AtomicUsize,
    lockfile_generated: AtomicUsize,
    lockfile_skipped: AtomicUsize,
    cargo_nix_generated: AtomicUsize,
    flake_generated: AtomicUsize,
    built: AtomicUsize,
    failed: AtomicUsize,
}

/// Generate Cargo.lock for a project.
fn generate_lockfile(project: &Path, cargo_path: &str, verbose: bool) -> Result<bool> {
    let cargo_toml = project.join("Cargo.toml");
    let cargo_lock = project.join("Cargo.lock");

    if !cargo_toml.exists() {
        return Ok(false);
    }

    if cargo_lock.exists() {
        if verbose {
            eprintln!("    [SKIP] Cargo.lock already exists");
        }
        return Ok(false);
    }

    if verbose {
        eprintln!("    [GEN]  generating Cargo.lock...");
    }

    let output = Command::new(cargo_path)
        .args(["generate-lockfile", "--offline", "--manifest-path"])
        .arg(&cargo_toml)
        .output()
        .context("Failed to run cargo generate-lockfile")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("cargo generate-lockfile failed: {}", stderr.trim())
    }
}

/// Run crate2nix generate to produce Cargo.nix.
fn generate_cargo_nix(project: &Path, crate2nix_path: &str, verbose: bool) -> Result<bool> {
    let cargo_toml = project.join("Cargo.toml");
    let cargo_nix = project.join("Cargo.nix");

    if !cargo_toml.exists() {
        return Ok(false);
    }

    if cargo_nix.exists() && cargo_lock_exists(project) {
        if verbose {
            eprintln!("    [SKIP] Cargo.nix already exists");
        }
        return Ok(false);
    }

    if verbose {
        eprintln!("    [GEN]  running crate2nix generate...");
    }

    let output = Command::new(crate2nix_path)
        .args(["generate", "-f"])
        .arg(&cargo_toml)
        .arg("-o")
        .arg(&cargo_nix)
        .output()
        .context("Failed to run crate2nix generate")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // If crate2nix isn't found on PATH, report as skipped
        if stderr.contains("not found") || stderr.contains("No such file") {
            eprintln!("    [WARN] crate2nix not available on PATH (--crate2nix-path), skipping");
            Ok(false)
        } else {
            // Print the actual error but don't fail the whole pipeline
            eprintln!("    [WARN] crate2nix generate had issues: {}", stderr.trim().lines().next().unwrap_or("unknown error"));
            Ok(false)
        }
    }
}

fn cargo_lock_exists(project: &Path) -> bool {
    project.join("Cargo.lock").exists()
}

/// Generate a minimal flake.nix for a project.
fn generate_flake(project: &Path, output_dir: &Path, verbose: bool) -> Result<bool> {
    let project_name = project
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let flake_path = project.join("flake.nix");
    if flake_path.exists() {
        if verbose {
            eprintln!("    [SKIP] flake.nix already exists");
        }
        return Ok(false);
    }

    if verbose {
        eprintln!("    [GEN]  generating flake.nix...");
    }

    let flake_content = format!(
        r#"{{
  description = "{} — Cargo project from DASL";

  inputs = {{
    nixpkgs.url = "git+file:///mnt/data1/git/github.com/NixOS/nixpkgs.git?ref=master";
    crate2nix = {{
      url = "path:/tmp/flake-local/crate2nix";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    flake-parts = {{
      url = "path:/tmp/flake-local/flake-parts";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    rust-overlay = {{
      url = "path:/tmp/flake-local/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
  }};

  outputs = inputs: let
    pkgs = inputs.nixpkgs.legacyPackages.x86_64-linux.extend
      inputs.rust-overlay.overlays.default;
  in {{
    packages.x86_64-linux.default = pkgs.rustPlatform.buildRustPackage {{
      pname = "{}";
      version = "0.1.0";
      src = builtins.path {{ path = ./.; name = "{}"; }};
      cargoLock.lockFile = builtins.path {{
        path = ./Cargo.lock;
        name = "lock";
      }};
    }};
  }};
}}
"#,
        project_name, project_name, project_name
    );

    std::fs::write(&flake_path, flake_content)
        .with_context(|| format!("Failed to write flake.nix at {}", flake_path.display()))?;

    Ok(true)
}

/// Build a project via nix build.
fn build_project(project: &Path, nix_path: &str, verbose: bool) -> Result<bool> {
    if !project.join("Cargo.nix").exists() && !project.join("flake.nix").exists() {
        if verbose {
            eprintln!("    [SKIP] no Cargo.nix or flake.nix to build");
        }
        return Ok(false);
    }

    if verbose {
        eprintln!("    [BUILD] running nix build...");
    }

    let output = Command::new(nix_path)
        .args(["build", ".#default", "--no-link", "--print-out-paths"])
        .current_dir(project)
        .output()
        .context("Failed to run nix build")?;

    if output.status.success() {
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("nix build failed: {}", stderr.trim())
    }
}

// ── Main ──────────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    use clap::Parser;
    let args = Args::parse();

    let index_dir = expand_tilde(&args.index_dir).to_string_lossy().to_string();
    let dasl_root = expand_tilde(&args.dasl_root).to_string_lossy().to_string();
    let output_dir = expand_tilde(&args.output_dir).to_string_lossy().to_string();

    let index_path = Path::new(&index_dir);
    let dasl_path = Path::new(&dasl_root);
    let out_path = Path::new(&output_dir);

    // Discover all the things
    eprintln!("[SCAN] Scanning index files in {}...", index_path.display());
    let start = Instant::now();

    let mut projects = scan_index_files(index_path, dasl_path, args.limit, args.verbose)?;

    // Filter out registry projects if requested
    if args.skip_registry {
        let before = projects.len();
        projects.retain(|p| !p.to_string_lossy().contains(".cargo/registry/"));
        let removed = before - projects.len();
        if removed > 0 { eprintln!("  Filtered out {} registry projects", removed); }
    }

    eprintln!("[SCAN] Found {} Cargo projects in {:.2}s", projects.len(), start.elapsed().as_secs_f64());

    if projects.is_empty() {
        eprintln!("[DONE] No Cargo projects found. Exiting.");
        return Ok(());
    }

    // Create output directories
    std::fs::create_dir_all(out_path)
        .context("Failed to create output directory")?;

    let stats = Arc::new(PipelineStats::default());
    let total = projects.len();

    // Process each project sequentially (parallelism via workers would use rayon)
    for (i, project) in projects.iter().enumerate() {
        let project_name = project
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        eprintln!(
            "\n[{}/{}] {} — {}",
            i + 1,
            total,
            project_name,
            project.display()
        );

        // Step 1: Generate lockfile
        if args.generate_lockfiles {
            match generate_lockfile(project, &args.cargo_path, args.verbose) {
                Ok(true) => {
                    stats.lockfile_generated.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ✅ Cargo.lock generated");
                }
                Ok(false) => {
                    stats.lockfile_skipped.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ❌ Lockfile generation failed: {}", e);
                    if args.verbose {
                        continue;
                    }
                }
            }
        }

        // Step 2: Generate Cargo.nix
        if args.generate_cargo_nix {
            match generate_cargo_nix(project, &args.crate2nix_path, args.verbose) {
                Ok(true) => {
                    stats.cargo_nix_generated.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ✅ Cargo.nix generated");
                }
                Ok(false) => {}
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ❌ crate2nix generate failed: {}", e);
                    if args.verbose {
                        continue;
                    }
                }
            }
        }

        // Step 3: Generate flake.nix
        if args.generate_flakes {
            match generate_flake(project, out_path, args.verbose) {
                Ok(true) => {
                    stats.flake_generated.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ✅ flake.nix generated");
                }
                Ok(false) => {}
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ❌ Flake generation failed: {}", e);
                }
            }
        }

        // Step 4: Build
        if args.build {
            match build_project(project, &args.nix_path, args.verbose) {
                Ok(true) => {
                    stats.built.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ✅ Build succeeded");
                }
                Ok(false) => {}
                Err(e) => {
                    stats.failed.fetch_add(1, Ordering::Relaxed);
                    eprintln!("    ❌ Build failed: {}", e);
                }
            }
        }

        stats.total.fetch_add(1, Ordering::Relaxed);
    }

    // Report
    let elapsed = start.elapsed();
    eprintln!("\n═══════════════════════════════════════════");
    eprintln!("  DASL Index Processing Complete");
    eprintln!("  Duration: {:.2}s", elapsed.as_secs_f64());
    eprintln!("───────────────────────────────────────────");
    eprintln!("  Total projects processed:  {}", stats.total.load(Ordering::Relaxed));
    if args.generate_lockfiles {
        eprintln!("  Lockfiles generated:       {}", stats.lockfile_generated.load(Ordering::Relaxed));
        eprintln!("  Lockfiles skipped:         {}", stats.lockfile_skipped.load(Ordering::Relaxed));
    }
    if args.generate_cargo_nix {
        eprintln!("  Cargo.nix generated:       {}", stats.cargo_nix_generated.load(Ordering::Relaxed));
    }
    if args.generate_flakes {
        eprintln!("  Flakes generated:          {}", stats.flake_generated.load(Ordering::Relaxed));
    }
    if args.build {
        eprintln!("  Builds succeeded:          {}", stats.built.load(Ordering::Relaxed));
    }
    eprintln!("  Failed:                    {}", stats.failed.load(Ordering::Relaxed));
    eprintln!("═══════════════════════════════════════════");

    Ok(())
}
