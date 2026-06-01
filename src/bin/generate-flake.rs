//! # generate-flake — Universal flake generator (multi-language)
//!
//! Auto-detects project language and generates flake.nix for:
//! - Rust (Cargo.toml) → uses `crate_flake` generator
//! - Go (go.mod)       → Go flake with buildGoModule
//! - Python (pyproject.toml, setup.py) → Python flake with buildPythonPackage
//! - C/C++ (Makefile, CMakeLists.txt)  → C flake with stdenv.mkDerivation
//! - JavaScript (package.json) → JS flake with buildNpmPackage
//! - Java (pom.xml, build.gradle) → Java flake with stdenv.mkDerivation + maven/gradle
//!
//! ## Usage
//!
//! ```text
//! # Auto-detect language and generate flake
//! generate-flake /path/to/project --output /tmp/flakes
//!
//! # Force a specific language
//! generate-flake /path/to/project --output /tmp/flakes --lang go
//!
//! # Batch: discover all projects under a directory tree
//! generate-flake /path/to/workspace --output /tmp/flakes --discover --max-depth 8
//!
//! # Git mirror source (for style crate2nix or source tracking)
//! generate-flake /path/to/project --git-mirror /mnt/data1/git/github.com/owner/repo.git
//! ```

use anyhow::{Context, Result};
use cargo_vendormod::crate_flake::{self, FlakeSource, FlakeStyle};
use cargo_vendormod::lang_detect::{self, Language};
use cargo_vendormod::multi_lang_flake;
use clap::Parser;
use std::path::PathBuf;
use std::fs;

#[derive(Parser, Debug)]
#[command(name = "generate-flake")]
#[command(about = "Universal multi-language flake generator", long_about = None)]
struct Args {
    /// Project directory (or workspace root with --discover)
    path: PathBuf,

    /// Output directory for generated flakes
    #[arg(long, short, default_value = "./flakes-output")]
    output: PathBuf,

    /// Flake generation style: "simple" (default) or "crate2nix" (Rust only)
    #[arg(long, default_value = "simple")]
    style: String,

    /// Force project language (auto-detect if not set)
    /// Values: rust, go, python, c, js, java
    #[arg(long)]
    lang: Option<String>,

    /// Path to nix-common (required for --style crate2nix)
    #[arg(long)]
    nix_common: Option<PathBuf>,

    /// Git mirror path (e.g. /mnt/data1/git/github.com/owner/repo.git)
    #[arg(long)]
    git_mirror: Option<PathBuf>,

    /// Display name for the git mirror
    #[arg(long)]
    mirror_name: Option<String>,

    /// Enable NORA registry integration (adds .cargo/config.toml redirecting to nora)
    #[arg(long)]
    nora: bool,

    /// Vendor directory path (Rust only)
    #[arg(long)]
    vendor: Option<PathBuf>,

    /// Discover all projects under the given path (recursive)
    #[arg(long)]
    discover: bool,

    /// Max depth for discovery (default: 8)
    #[arg(long, default_value = "8")]
    max_depth: usize,

    /// Verbose output
    #[arg(long, short)]
    verbose: bool,
}

fn detect_project_language(path: &std::path::Path, force_lang: Option<&str>) -> Result<Language> {
    if let Some(lang_str) = force_lang {
        match lang_str.to_lowercase().as_str() {
            "rust" => Ok(Language::Rust),
            "go" => Ok(Language::Go),
            "python" | "py" => Ok(Language::Python),
            "c" | "cpp" | "cxx" | "c++" => Ok(Language::C),
            "js" | "javascript" | "node" => Ok(Language::JavaScript),
            "java" => Ok(Language::Java),
            other => anyhow::bail!("Unknown language '{}'. Valid: rust, go, python, c, js, java", other),
        }
    } else {
        Ok(lang_detect::detect_language(path))
    }
}

fn generate_single_flake(
    project_dir: &std::path::Path,
    output_dir: &std::path::Path,
    lang: Language,
    args: &Args,
) -> Result<usize> {
    let style = FlakeStyle::from_str(Some(&args.style));

    if style == FlakeStyle::Crate2Nix && lang != Language::Rust {
        anyhow::bail!("--style crate2nix is only supported for Rust projects");
    }

    let source = match (&args.git_mirror, &args.mirror_name) {
        (Some(mirror_path), Some(display)) => FlakeSource::GitMirror {
            mirror_path: mirror_path.clone(),
            display: display.clone(),
        },
        (Some(mirror_path), None) => {
            let display = mirror_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("mirror")
                .trim_end_matches(".git")
                .to_string();
            FlakeSource::GitMirror {
                mirror_path: mirror_path.clone(),
                display,
            }
        }
        _ => FlakeSource::LocalDir,
    };

    match lang {
        Language::Rust => {
            if !project_dir.join("Cargo.toml").exists() {
                anyhow::bail!("No Cargo.toml in {} (language detected as Rust)", project_dir.display());
            }
            if args.nora {
                // Run cargo vendor to create vendored deps for offline nix builds
                // This uses nora as the registry (via global .cargo/config.toml)
                let vendor_dir = output_dir.join("vendor");
                if !vendor_dir.exists() {
                    eprintln!("[nora] Running cargo vendor for {}...", project_dir.display());
                    let status = std::process::Command::new("cargo")
                        .args(["vendor", vendor_dir.to_str().unwrap_or("vendor")])
                        .current_dir(project_dir)
                        .env("CARGO_TARGET_DIR", "/tmp/cargo-vendormod-target")
                        .status()
                        .context("Failed to run cargo vendor")?;
                    if !status.success() {
                        anyhow::bail!("cargo vendor failed for {}", project_dir.display());
                    }
                    eprintln!("[nora] Vendored {} deps to {}", vendor_dir.display(), vendor_dir.display());
                }
            }
            if let Some(mirror) = &args.git_mirror {
                crate_flake::generate_flake(
                    project_dir, output_dir, args.vendor.as_deref(),
                    style, args.nix_common.as_deref(),
                    FlakeSource::GitMirror {
                        mirror_path: mirror.clone(),
                        display: args.mirror_name.clone().unwrap_or_else(|| {
                            mirror.file_stem().and_then(|s| s.to_str()).unwrap_or("mirror").to_string()
                        }),
                    },
                    args.nora,
                )
            } else {
                crate_flake::generate_flake(
                    project_dir, output_dir, args.vendor.as_deref(),
                    style, args.nix_common.as_deref(),
                    FlakeSource::LocalDir,
                    args.nora,
                )
            }
        }
        Language::Go => {
            multi_lang_flake::generate_go_flake(project_dir, output_dir, style, &source)
        }
        Language::Python => {
            multi_lang_flake::generate_python_flake(project_dir, output_dir, style, &source)
        }
        Language::C => {
            multi_lang_flake::generate_c_flake(project_dir, output_dir, style, &source)
        }
        Language::JavaScript => {
            multi_lang_flake::generate_js_flake(project_dir, output_dir, style, &source)
        }
        Language::Java => {
            multi_lang_flake::generate_java_flake(project_dir, output_dir, style, &source)
        }
        Language::Lean4 => {
            anyhow::bail!("Lean4 flake generation not yet implemented for {}. Use --lang rust as fallback.", project_dir.display())
        }
        Language::Unknown => {
            anyhow::bail!("Could not detect language in {}. Use --lang to specify.", project_dir.display())
        }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.output)
        .context("Failed to create output directory")?;

    if args.discover {
        // ── Discovery mode ──
        let projects = lang_detect::discover_projects(&args.path, args.max_depth);

        if projects.is_empty() {
            eprintln!("No supported projects found under {}", args.path.display());
            return Ok(());
        }

        println!("Found {} projects under {}", projects.len(), args.path.display());

        let mut total_files = 0usize;
        let mut success = 0usize;
        let mut failures = 0usize;

        for (proj_path, proj_lang) in &projects {
            let proj_name: &str = proj_path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            if args.verbose {
                println!("  {:?} {} ({:?})", proj_path, proj_name, proj_lang);
            }

            match generate_single_flake(proj_path, &args.output, *proj_lang, &args) {
                Ok(count) => {
                    total_files += count;
                    success += 1;
                }
                Err(e) => {
                    eprintln!("  FAILED {}: {}", proj_name, e);
                    failures += 1;
                }
            }
        }

        println!("\nGenerated {} file(s) for {}/{} projects in {}",
            total_files, success, success + failures, args.output.display());
        if failures > 0 {
            eprintln!("{} project(s) failed", failures);
        }
    } else {
        // ── Single project mode ──
        let lang = detect_project_language(&args.path, args.lang.as_deref())?;
        let proj_name: &str = args.path.file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("project");
        println!("Detected language: {:?} for {}", lang, proj_name);

        let count = generate_single_flake(&args.path, &args.output, lang, &args)?;
        println!("Generated {} file(s) in {}", count, args.output.display());
    }

    Ok(())
}
