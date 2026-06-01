//! # Per-Crate Flake Generator
//!
//! Generates production-quality `flake.nix` files for individual Rust crates
//!
//! Generates production-quality `flake.nix` files for individual Rust crates
//! processed through the cargo-vendormod pipeline.
//!
//! Supports multiple flake generation styles:
//!
//! - `Simple` (default): Uses `rustPlatform.buildRustPackage` with
//!   `cargoLock.lockFile` (standard Nixpkgs pattern). The source comes
//!   either from a local directory (`src = ./.;`) or from a git mirror
//!   (`src` flake input pointing to a local bare repo).
//!
//! - `Crate2Nix`: Uses the crate2nix pattern with nix-common infrastructure,
//!   generating a flake that references a local git mirror and builds via
//!   `tools.generatedCargoNix`. This matches the existing DASL flake pattern.
//!
//! Output structure per crate:
//! ```text
//! output_dir/
//!   flakes/
//!     <crate_name>/
//!       flake.nix       # multi-system build flake
//!       package.nix     # buildRustPackage derivation (consumed by flake.nix)
//! ```
//!
//! Usage via processing pipeline:
//! ```text
//! processing all --input-file <crate_dirs> --output-dir <out> --generate-flakes
//! processing all --input-file <crate_dirs> --output-dir <out> --generate-flakes \
//!   --style crate2nix --nix-common /home/mdupont/nix-common
//! ```

use anyhow::{bail, Context, Result};

/// .cargo/config.toml content for redirecting to nora registry
const NORA_CARGO_CONFIG: &str = r#"[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
"#;
use serde::Deserialize;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// ── Cargo.toml data structures (minimum needed) ────────────────────────

#[derive(Debug, Deserialize)]
struct CargoToml {
    package: Option<CargoPackage>,
}

#[derive(Debug, Deserialize)]
struct CargoPackage {
    name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_version")]
    version: Option<String>,
}

/// Deserialize version field that can be a string ("1.2.3") or a
/// workspace-inherited table ({ workspace = true }). In the latter case
/// we return None — the actual version comes from the workspace root.
fn deserialize_version<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // Try string first
    if let Ok(s) = <Option<String>>::deserialize(deserializer) {
        return Ok(s);
    }
    // Otherwise it's a table ({ workspace = true }) — skip
    Ok(None)
}

// ── Flake generation style ────────────────────────────────────────────

/// The style of flake.nix to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlakeStyle {
    /// Simple `buildRustPackage` + `cargoLock.lockFile` pattern.
    /// Source can be local dir or git mirror.
    Simple,
    /// Crate2nix pattern using nix-common infrastructure.
    /// Requires nix-common path. Always uses git mirror source.
    Crate2Nix,
}

impl fmt::Display for FlakeStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FlakeStyle::Simple => write!(f, "simple"),
            FlakeStyle::Crate2Nix => write!(f, "crate2nix"),
        }
    }
}

impl FlakeStyle {
    /// Parse from a string, returning `Simple` as default.
    pub fn from_str(s: Option<&str>) -> Self {
        match s {
            Some("crate2nix") => FlakeStyle::Crate2Nix,
            _ => FlakeStyle::Simple,
        }
    }
}

/// Where the crate source comes from.
#[derive(Debug, Clone)]
pub enum FlakeSource {
    /// Source is a local directory (src = ./.;).
    LocalDir,
    /// Source is a git mirror bare repo.
    GitMirror {
        /// Absolute path to the bare git mirror (e.g. /mnt/data1/git/github.com/ipld/rust-ipld-core.git)
        mirror_path: PathBuf,
        /// Display name for the mirror reference (e.g. "github.com/ipld/rust-ipld-core")
        display: String,
    },
}

// ── Flake generation ───────────────────────────────────────────────────

/// Generate a production-quality `flake.nix` + `package.nix` for a Rust crate.
///
/// * `crate_dir`   — directory containing `Cargo.toml` (and optionally `Cargo.lock`)
/// * `output_dir`  — base output directory; `<output_dir>/flakes/<crate_name>/` is created
/// * `vendor_dir`  — optional path to a vendored deps directory (if already materialized)
/// * `style`       — flake generation style (`Simple` or `Crate2Nix`)
/// * `nix_common`  — path to nix-common directory (required for `Crate2Nix` style)
/// * `source`      — where the crate source is located
///
/// Returns the number of files written.
pub fn generate_flake(
    crate_dir: &Path,
    output_dir: &Path,
    vendor_dir: Option<&Path>,
    style: FlakeStyle,
    nix_common: Option<&Path>,
    source: FlakeSource,
    nora: bool,
) -> Result<usize> {
    let cargo_toml_path = crate_dir.join("Cargo.toml");

    // Read and parse Cargo.toml
    let toml_raw = fs::read_to_string(&cargo_toml_path)
        .with_context(|| format!("Failed to read {}", cargo_toml_path.display()))?;
    let cargo_toml: CargoToml = toml::de::from_str(&toml_raw)
        .context("Failed to parse Cargo.toml")?;

    // Virtual workspace check: skip if no [package] section
    let cargo_pkg = match cargo_toml.package {
        Some(ref pkg) => pkg,
        None => bail!(
            "Virtual workspace manifest (no [package] section): {}. Use a workspace member instead.",
            cargo_toml_path.display()
        ),
    };

    let crate_name = cargo_pkg.name.as_deref().unwrap_or("crate");
    let crate_version = cargo_pkg.version.as_deref().unwrap_or("0.0.0");

    // Create output directory
    let flake_dir = output_dir.join("flakes").join(crate_name);
    fs::create_dir_all(&flake_dir)
        .with_context(|| format!("Failed to create flake dir: {}", flake_dir.display()))?;

    // Detect whether Cargo.lock exists — search upward for workspace roots
    let has_lock = crate_dir.join("Cargo.lock").exists();

    // Check if Cargo.lock exists at a parent (workspace root) — note it in a comment
    let workspace_lock = if !has_lock {
        find_cargo_lock(crate_dir)
    } else {
        None
    };

    match style {
        FlakeStyle::Simple => {
            // Generate package.nix (simple buildRustPackage)
            let package_nix = render_package_nix(
                crate_name, crate_version, has_lock, vendor_dir, workspace_lock.as_deref(), nora,
            );
            fs::write(flake_dir.join("package.nix"), &package_nix)
                .with_context(|| format!("Failed to write package.nix for {}", crate_name))?;

            // Generate flake.nix (simple style)
            let flake_nix = render_flake_nix_simple(crate_name, crate_version, &source);
            fs::write(flake_dir.join("flake.nix"), &flake_nix)
                .with_context(|| format!("Failed to write flake.nix for {}", crate_name))?;

            // When using nora, copy Cargo.lock and write .cargo/config.toml
            if nora {
                let lock_src = if has_lock {
                    crate_dir.join("Cargo.lock")
                } else if let Some(ref wl) = workspace_lock {
                    wl.clone()
                } else {
                    // No lock file — generate one via cargo generate-lockfile
                    let status = std::process::Command::new("cargo")
                        .args(["generate-lockfile"])
                        .current_dir(crate_dir)
                        .env("CARGO_TARGET_DIR", "/tmp/cargo-vendormod-target")
                        .status()?;
                    if !status.success() {
                        bail!("cargo generate-lockfile failed for {}", crate_dir.display());
                    }
                    crate_dir.join("Cargo.lock")
                };
                if lock_src.exists() {
                    fs::copy(&lock_src, flake_dir.join("Cargo.lock"))
                        .with_context(|| format!("Failed to copy Cargo.lock from {}", lock_src.display()))?;
                }

                // Write .cargo/config.toml for non-nix cargo builds (redirects to nora)
                // Place it in the output root, NOT in the flake dir (which becomes the nix source)
                let cargo_dir = output_dir.join(".cargo");
                fs::create_dir_all(&cargo_dir)?;
                fs::write(cargo_dir.join("config.toml"), NORA_CARGO_CONFIG)?;
            }

            Ok(2)
        }
        FlakeStyle::Crate2Nix => {
            let nix_common = nix_common.context("Crate2Nix style requires --nix-common path")?;
            let git_mirror = match &source {
                FlakeSource::GitMirror { mirror_path, display } => {
                    (mirror_path.clone(), display.clone())
                }
                FlakeSource::LocalDir => {
                    bail!("Crate2Nix style requires a git mirror source (use FlakeSource::GitMirror)");
                }
            };

            // Generate flake.nix (crate2nix style) — no package.nix, everything in flake.nix
            let flake_nix = render_flake_nix_crate2nix(
                crate_name, crate_version, &nix_common, &git_mirror.0, &git_mirror.1,
            );
            fs::write(flake_dir.join("flake.nix"), &flake_nix)
                .with_context(|| format!("Failed to write flake.nix for {}", crate_name))?;

            Ok(1) // only flake.nix, no package.nix for crate2nix style
        }
    }
}

// ── Rendering ──────────────────────────────────────────────────────────

/// Render `package.nix` — the `buildRustPackage` derivation consumed by the simple flake.
fn render_package_nix(
    crate_name: &str,
    crate_version: &str,
    has_lock: bool,
    vendor_dir: Option<&Path>,
    workspace_lock: Option<&Path>,
    nora: bool,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    lines.push("{ pkgs, lib, ... }:".to_string());
    lines.push(String::new());
    lines.push(format!(
        "pkgs.rustPlatform.buildRustPackage {{"
    ));
    lines.push(format!("  pname = \"{}\";", crate_name));
    lines.push(format!("  version = \"{}\";", crate_version));
    lines.push("  src = ./.;".to_string());

    if nora {
        // Nora integration: for nix builds, cargoLock.lockFile is used (nix fetches deps
        // as fixed-output derivations). For non-nix cargo builds, the .cargo/config.toml
        // redirecting to nora is written separately (outside the source dir).
        if has_lock {
            lines.push("  cargoLock.lockFile = ./Cargo.lock;".to_string());
        } else if let Some(wl) = workspace_lock {
            lines.push("  cargoLock.lockFile = ./Cargo.lock;".to_string());
            lines.push(format!("  # (workspace lock originally at: {})", wl.display()));
        }
        // Skip tests in nix sandbox (many crates have test failures in sandbox)
        lines.push("  doCheck = false;".to_string());
    } else if let Some(vendor) = vendor_dir {
        // Vendored deps mode — vendored dir already materialized
        lines.push(format!("  cargoVendorDir = ./{};", vendor.display()));
        lines.push("  # vendorHash = \"\";  # set after first build".to_string());
    } else if has_lock {
        // Standard nixpkgs cargoLock.lockFile pattern
        lines.push("  cargoLock.lockFile = ./Cargo.lock;".to_string());
    } else if let Some(wl) = workspace_lock {
        // Cargo.lock found at workspace root (not in this crate dir).
        // The lock file will be placed alongside the flake when deployed.
        lines.push("  cargoLock.lockFile = ./Cargo.lock;".to_string());
        lines.push(format!("  # (workspace lock originally at: {})", wl.display()));
    } else {
        // No lock file found anywhere
        lines.push("  # cargoLock.lockFile = ./Cargo.lock;  # add once Cargo.lock exists".to_string());
    }

    lines.push("}".to_string());

    lines.join("\n")
}

/// Render a simple `flake.nix` — multi-system wrapper calling `package.nix`.
///
/// Supports both local directory and git mirror sources.
fn render_flake_nix_simple(crate_name: &str, crate_version: &str, source: &FlakeSource) -> String {
    let (src_input, src_attr, _extra_inputs) = match source {
        FlakeSource::LocalDir => {
            // No extra inputs needed; package.nix points to src = ./.
            (String::new(), String::new(), String::new())
        }
        FlakeSource::GitMirror { mirror_path, display: _ } => {
            // Add a `src` input pointing to the git mirror.
            // The package.nix references `inputs.src` instead of `./.`.
            let input = format!(
                r#"    src = {{
      url = "git+file://{mirror}";
      flake = false;
    }};
"#,
                mirror = mirror_path.display()
            );
            (input, "src".to_string(), String::new())
        }
    };

    let params = if src_attr.is_empty() {
        "{ self, nixpkgs, flake-utils }".to_string()
    } else {
        format!("{{ self, nixpkgs, flake-utils, {src_attr} }}")
    };

    let src_reference = if src_attr.is_empty() {
        "./.".to_string()
    } else {
        src_attr.clone()
    };

    format!(
        r#"# THIS FILE IS AUTO-GENERATED by cargo-vendormod
# DO NOT EDIT MANUALLY — regenerate with:
#   cargo-vendormod process crates --generate-flakes
#
# Crate: {name}
# Version: {version}

{{
  description = "{name} - {version}";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
{src_input}  }};

  outputs =
    {params}:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {{ inherit system; }};
        real_src = {src_reference};
      in
      {{
        packages = {{
          default = pkgs.callPackage ./package.nix {{ src = real_src; }};
          {name} = self.packages.${{system}}.default;
        }};

        devShells.default = pkgs.mkShell {{
          buildInputs = with pkgs; [
            rustc
            cargo
            clippy
            rustfmt
          ];
        }};

        # Verification: nix flake check
        checks.default = self.packages.${{system}}.default;
      }}
    );
}}
"#,
        name = crate_name,
        version = crate_version,
        src_input = src_input,
        params = params,
        src_reference = src_reference,
    )
}

/// Render a crate2nix-style `flake.nix` — uses nix-common + crate2nix infrastructure.
///
/// Matches the existing DASL flake pattern:
/// - References a local git mirror as `src` input
/// - Uses nix-common for crate2nix tools
/// - Builds via `tools.generatedCargoNix`
fn render_flake_nix_crate2nix(
    crate_name: &str,
    crate_version: &str,
    nix_common: &Path,
    mirror_path: &Path,
    mirror_display: &str,
) -> String {
    // Sanitize: strip trailing .git for display
    let display_name = mirror_display.trim_end_matches(".git");

    format!(
        r#"# THIS FILE IS AUTO-GENERATED by cargo-vendormod
# DO NOT EDIT MANUALLY — regenerate with:
#   cargo-vendormod process crates --generate-flakes --style crate2nix
#
# Crate: {name}
# Version: {version}
# Source: {mirror}

{{
  description = "{name} — crate2nix build from local git mirror";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {{
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    nix-common = {{
      url = "path:{nix_common_path}";
      inputs.nixpkgs.follows = "nixpkgs";
    }};
    src = {{
      url = "git+file://{mirror_path}";
      flake = false;
    }};
  }};

  outputs = {{ self, nixpkgs, flake-utils, rust-overlay, nix-common, src }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {{
          inherit system;
          overlays = [ (import rust-overlay) ];
        }};

        crate2nix = nix-common.crate2nix;
        tools = import "${{crate2nix}}/tools.nix" {{ inherit pkgs; }};
        rustToolchain = pkgs.rust-bin.nightly.latest.default;

        cargoNix = import (tools.generatedCargoNix {{
          name = "{name}";
          inherit src;
          cargo = rustToolchain;
        }}) {{
          inherit pkgs;
          buildRustCrateForPkgs = p:
            p.buildRustCrate.override {{
              rustc = rustToolchain;
              defaultCrateOverrides = p.defaultCrateOverrides // {{}};
            }};
        }};

      in {{
        packages = {{
          default = cargoNix.rootCrate.build;
          {name} = cargoNix.rootCrate.build;
        }};

        devShells.default = pkgs.mkShell {{
          packages = [ rustToolchain ];
        }};
      }});
}}
"#,
        name = crate_name,
        version = crate_version,
        mirror = display_name,
        nix_common_path = nix_common.display(),
        mirror_path = mirror_path.display(),
    )
}

// ── Convenience function for processing pipeline ───────────────────────

/// Generate flakes for many crates from a list of crate directories.
/// This is the entry point used by `cmd_all` in `processing.rs`.
pub fn generate_flakes_for_crates(
    crate_paths: &[PathBuf],
    output_dir: &Path,
    vendor_dir: Option<&Path>,
    style: FlakeStyle,
    nix_common: Option<&Path>,
    source: &FlakeSource,
    verbose: bool,
    nora: bool,
) -> Result<usize> {
    let mut total = 0;

    for crate_path in crate_paths {
        if verbose {
            let name = crate_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            eprintln!("  Generating flake for: {}", name);
        }

        match generate_flake(crate_path, output_dir, vendor_dir, style, nix_common, source.clone(), nora) {
            Ok(count) => total += count,
            Err(e) => eprintln!("  Warning: failed to generate flake for {}: {:#}", crate_path.display(), e),
        }
    }

    Ok(total)
}

// ── Scan helpers ───────────────────────────────────────────────────────

/// Search upward from `crate_dir` to find the nearest `Cargo.lock`.
/// Workspace sub-crates keep their lock at the workspace root.
fn find_cargo_lock(crate_dir: &Path) -> Option<PathBuf> {
    let mut dir = Some(crate_dir);
    while let Some(d) = dir {
        let lock = d.join("Cargo.lock");
        if lock.exists() {
            return Some(lock);
        }
        dir = d.parent();
    }
    None
}

/// Find all Cargo.toml files under a directory, returning their parent dirs.
///
/// `max_depth` controls how deep to scan (default: 8 levels is enough for most NUR repo structures).
pub fn find_crate_dirs(root: &Path) -> Result<Vec<PathBuf>> {
    let mut crates = Vec::new();

    for entry in walkdir::WalkDir::new(root).max_depth(8) {
        let entry = entry?;
        if entry.file_name() == "Cargo.toml" {
            if let Some(parent) = entry.path().parent() {
                crates.push(parent.to_path_buf());
            }
        }
    }

    // Deduplicate by canonical path
    crates.sort();
    crates.dedup();

    Ok(crates)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_render_package_nix_with_lock() {
        let result = render_package_nix("my-crate", "1.2.3", true, None, None, false);
        assert!(result.contains("pname = \"my-crate\";"), "expected pname");
        assert!(result.contains("version = \"1.2.3\";"), "expected version");
        assert!(result.contains("cargoLock.lockFile"), "expected lock file reference");
    }

    #[test]
    fn test_render_package_nix_with_vendor() {
        let result = render_package_nix("my-crate", "0.1.0", false, Some(Path::new("vendor")), None, false);
        assert!(result.contains("cargoVendorDir = ./vendor;"), "expected vendored dir");
        assert!(!result.contains("cargoLock.lockFile"), "lock file should not appear");
    }

    #[test]
    fn test_render_flake_nix() {
        let result = render_flake_nix_simple("hello", "0.1.0", &FlakeSource::LocalDir);
        assert!(result.contains("description = \"hello - 0.1.0\";"));
        assert!(result.contains("flake-utils"));
        assert!(result.contains("package.nix"));
        assert!(result.contains("devShells.default"));
        assert!(result.contains("checks.default"));
    }

    #[test]
    fn test_generate_flake_creates_files() {
        let tmp = tempfile::tempdir().unwrap();
        let crate_dir = tmp.path().join("my-crate");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "test-crate"
version = "0.5.0"
"#,
        )
        .unwrap();
        fs::write(crate_dir.join("Cargo.lock"), "# dummy lock").unwrap();

        let out_dir = tmp.path().join("output");
        let count = generate_flake(
            &crate_dir, &out_dir, None,
            FlakeStyle::Simple, None, FlakeSource::LocalDir, false,
        ).unwrap();
        assert_eq!(count, 2);

        let flake_path = out_dir.join("flakes").join("test-crate").join("flake.nix");
        let pkg_path = out_dir.join("flakes").join("test-crate").join("package.nix");
        assert!(flake_path.exists(), "flake.nix should exist");
        assert!(pkg_path.exists(), "package.nix should exist");

        let flake = fs::read_to_string(&flake_path).unwrap();
        assert!(flake.contains("test-crate - 0.5.0"));
    }

    #[test]
    fn test_generate_flake_with_vendor_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let crate_dir = tmp.path().join("vendored-crate");
        fs::create_dir_all(&crate_dir).unwrap();
        fs::write(
            crate_dir.join("Cargo.toml"),
            r#"[package]
name = "vendored"
version = "0.2.0"
"#,
        )
        .unwrap();

        let vendor_dir = tmp.path().join("vendor");
        fs::create_dir_all(&vendor_dir).unwrap();
        fs::write(vendor_dir.join(".vendor-stamp"), "").unwrap();

        let out_dir = tmp.path().join("output");
        let count = generate_flake(
            &crate_dir, &out_dir, Some(vendor_dir.as_path()),
            FlakeStyle::Simple, None, FlakeSource::LocalDir, false,
        ).unwrap();
        assert_eq!(count, 2);

        let pkg = fs::read_to_string(out_dir.join("flakes").join("vendored").join("package.nix")).unwrap();
        assert!(pkg.contains("cargoVendorDir = ./"), "should reference vendor dir");
    }

    #[test]
    fn test_find_crate_dirs() {
        let tmp = tempfile::tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("crate_a")).unwrap();
        fs::write(tmp.path().join("crate_a").join("Cargo.toml"), "[package]\nname = \"a\"\n").unwrap();
        fs::create_dir_all(tmp.path().join("sub/crate_b")).unwrap();
        fs::write(tmp.path().join("sub/crate_b").join("Cargo.toml"), "[package]\nname = \"b\"\n").unwrap();

        let crates = find_crate_dirs(tmp.path()).unwrap();
        assert!(crates.iter().any(|p| p.ends_with("crate_a")));
        assert!(crates.iter().any(|p| p.ends_with("crate_b")));
    }
}
