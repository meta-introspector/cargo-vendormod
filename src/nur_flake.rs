//! # NUR Flake Generator
//!
//! Reads `repos.json` (repo URLs) and `repos.json.lock` (locked revisions)
//! and generates a `flake.nix` that lists every NUR repository as a flake input
//! with locked revisions for reproducible evaluation.
//!
//! Adapted from the Python `generate_nur_flake.py` approach, now natively in Rust
//! as part of cargo-vendormod's all-in-one CLI tooling.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

// ── Data structures matching repos.json / repos.json.lock ──────────────

#[derive(Debug, Deserialize)]
struct ReposFile {
    repos: BTreeMap<String, RepoEntry>,
}

#[derive(Debug, Deserialize)]
struct RepoEntry {
    url: String,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    r#type: Option<String>,
    #[serde(default)]
    submodules: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct LockFile {
    repos: BTreeMap<String, LockEntry>,
}

#[derive(Debug, Deserialize)]
struct LockEntry {
    rev: String,
    sha256: Option<String>,
}

/// Represents a single processed NUR repository for flake generation.
#[derive(Debug, Clone)]
struct NurRepo {
    name: String,
    url: String,
    rev: String,
}

// ── URL helpers ───────────────────────────────────────────────────────

fn is_github_url(url: &str) -> bool {
    url.starts_with("https://github.com/")
        || url.starts_with("git+https://github.com/")
        || url.starts_with("github:")
}

fn sanitize_name(name: &str) -> String {
    name.replace('-', "_")
        .replace('.', "_")
        .replace('/', "_")
}

fn to_flake_input_url(url: &str, rev: &str) -> String {
    let clean = url
        .trim_start_matches("git+")
        .trim_end_matches(".git");

    if is_github_url(url) {
        // github:owner/repo/rev format
        let path = clean
            .trim_start_matches("https://github.com/")
            .trim_start_matches("github:");
        format!("github:{}/{}", path, rev)
    } else if url.contains("gitlab") || url.contains("gitlab.com") {
        format!("git+{}?rev={}", clean, rev)
    } else {
        // Generic git URL
        format!("git+{}?rev={}", clean, rev)
    }
}

// ── Generator ─────────────────────────────────────────────────────────

/// The NUR flake generator. Reads JSON manifests and produces a flake.nix.
pub struct NurFlakeGenerator {
    repos_path: PathBuf,
    lock_path: PathBuf,
    output_path: PathBuf,
}

impl NurFlakeGenerator {
    /// Create a new generator with paths to the input files and output destination.
    pub fn new(repos_path: PathBuf, lock_path: PathBuf, output_path: PathBuf) -> Self {
        Self {
            repos_path,
            lock_path,
            output_path,
        }
    }

    /// Generate the flake.nix, returning the number of repos processed.
    pub fn generate(&self) -> Result<usize> {
        let repos = self.load_and_merge()?;
        let content = Self::render_flake(&repos);
        let repo_count = repos.len();

        // Ensure parent directory exists
        if let Some(parent) = self.output_path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }

        fs::write(&self.output_path, &content)
            .with_context(|| format!("Failed to write {}", self.output_path.display()))?;

        Ok(repo_count)
    }

    /// Check whether the current flake.nix matches what would be generated.
    /// Returns `true` if up-to-date, `false` if out of date.
    pub fn check(&self) -> Result<bool> {
        let repos = self.load_and_merge()?;
        let content = Self::render_flake(&repos);

        if self.output_path.exists() {
            let existing = fs::read_to_string(&self.output_path)
                .with_context(|| format!("Failed to read {}", self.output_path.display()))?;
            Ok(existing == content)
        } else {
            Ok(false)
        }
    }

    /// Load repos.json and repos.json.lock, validate, merge into a sorted list.
    fn load_and_merge(&self) -> Result<Vec<NurRepo>> {
        let repos_raw: String = fs::read_to_string(&self.repos_path)
            .with_context(|| format!("Failed to read {}", self.repos_path.display()))?;
        let lock_raw: String = fs::read_to_string(&self.lock_path)
            .with_context(|| format!("Failed to read {}", self.lock_path.display()))?;

        let repos_file: ReposFile = serde_json::from_str(&repos_raw)
            .context("Failed to parse repos.json")?;
        let lock_file: LockFile = serde_json::from_str(&lock_raw)
            .context("Failed to parse repos.json.lock")?;

        let mut merged: Vec<NurRepo> = Vec::new();

        for (name, entry) in &repos_file.repos {
            let rev = match lock_file.repos.get(name) {
                Some(lock) => lock.rev.clone(),
                None => {
                    eprintln!("Warning: {} has no locked revision in repos.json.lock", name);
                    "unknown".to_string()
                }
            };

            merged.push(NurRepo {
                name: name.clone(),
                url: entry.url.clone(),
                rev,
            });
        }

        // Sort by sanitized name for deterministic output
        merged.sort_by(|a, b| sanitize_name(&a.name).cmp(&sanitize_name(&b.name)));

        Ok(merged)
    }

    /// Render the Nix flake.nix content from a list of repos.
    fn render_flake(repos: &[NurRepo]) -> String {
        let count = repos.len();

        // ── Input lines ───────────────────────────────────────────────
        let input_lines: Vec<String> = repos
            .iter()
            .map(|r| {
                let sname = sanitize_name(&r.name);
                let input_url = to_flake_input_url(&r.url, &r.rev);
                format!("    {sname}.url = \"{input_url}\";")
            })
            .collect();
        let inputs = input_lines.join("\n");

        // ── Output lambda parameter list ──────────────────────────────
        let param_names: Vec<String> = repos
            .iter()
            .map(|r| sanitize_name(&r.name))
            .collect();
        let params = param_names.join(", ");

        // ── Repo attribute lines ──────────────────────────────────────
        let attr_lines: Vec<String> = repos
            .iter()
            .map(|r| {
                let sname = sanitize_name(&r.name);
                format!(
                    "        # {orig}\n        {sname} = evalRepo \"{orig}\" {sname};",
                    orig = r.name
                )
            })
            .collect();
        let attrs = attr_lines.join("\n");

        // ── Build the complete flake.nix string ───────────────────────
        format!(
            r#"# THIS FILE IS AUTO-GENERATED by cargo-vendormod nur-flake
# DO NOT EDIT MANUALLY — regenerate with:
#   cargo-vendormod nur-flake --repos-json repos.json --lock-json repos.json.lock --output flake.nix
#
# Generated from cargo-vendormod flake-generation pattern.
# Contains {count} NUR repository inputs.

{{
  description = "NUR Combined — every nix-community NUR repository as a flake";

  inputs = {{
    # Pinned nixpkgs for reproducible evaluation
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    # ── {count} NUR repository inputs ──
{inputs}
  }};

  outputs =
    {{ self, nixpkgs
     , {params}
     }}:
    let
      # Systems to build for
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);

      # Helper: derive a system-specific pkgs from nixpkgs
      pkgsFor = system: import nixpkgs {{ inherit system; }};

      # ── Per-repo evaluation ──────────────────────────────────────────
      # Each NUR repo gets evaluated with the standard NUR pattern:
      #   import default.nix {{ pkgs = ...; }}
      #
      # Some repos provide flake.nix instead — we detect and handle both.
      evalRepo = name: input:
        let
          # Check if the repo has a flake.nix (flake-enabled NUR repo)
          hasFlake = builtins.pathExists (input + "/flake.nix");
          # If flake, use its packages; otherwise use the default.nix pattern
          result = if hasFlake
            then input.packages
            else (import (input + "/default.nix") {{ }});
        in result;

      # ── Per-repo package sets ────────────────────────────────────────
      repoPackages = {{
{attrs}
      }};

      # ── Combined package set (union of all repos) ───────────────────
      # Collects every package from every NUR repo into a flat attribute set.
      # Conflicts are prefixed with "<repo-name>-".
      allPackages = nixpkgs.lib.foldl
        (acc: repoName:
          let
            pkgs = repoPackages.${{repoName}};
            prefixed = nixpkgs.lib.mapAttrs'
              (name: value: {{
                name = "${{repoName}}-${{name}}";
                inherit value;
              }})
              (builtins.removeAttrs pkgs [ "overlays" "modules" "nixosModules" "checks" "devShells" "formatter"]);
          in acc // prefixed
        )
        {{ }}
        (builtins.attrNames repoPackages);
    in
    {{
      # Per-repo packages — use as: nix build .#<repo-name>.<package>
      inherit repoPackages;

      # Flat combined packages (repo-name-prefixed) — use as: nix build .#mic92-hello-nur
      packages = forAllSystems (system: allPackages);

      # Legacy NUR-compatible top-level (for nix-env -f default.nix)
      legacyPackages = forAllSystems (system: repoPackages);

      # Apps: list all repos
      apps = forAllSystems (system: {{
        list-repos = {{
          type = "app";
          program = "${{nixpkgs.legacyPackages.${{system}}.coreutils}}/bin/echo";
        }};
      }});
    }};
}}
"#,
            count = count,
            inputs = inputs.trim_end(),
            params = params,
            attrs = attrs.trim_end(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        assert_eq!(sanitize_name("hello-world"), "hello_world");
        assert_eq!(sanitize_name("foo.bar"), "foo_bar");
        assert_eq!(sanitize_name("simple"), "simple");
        assert_eq!(sanitize_name("a/b"), "a_b");
    }

    #[test]
    fn test_to_flake_input_url_github() {
        let url = "https://github.com/owner/repo";
        let rev = "abc123";
        let result = to_flake_input_url(url, rev);
        assert_eq!(result, "github:owner/repo/abc123");
    }

    #[test]
    fn test_to_flake_input_url_github_with_git() {
        let url = "https://github.com/owner/repo.git";
        let rev = "abc123";
        let result = to_flake_input_url(url, rev);
        assert_eq!(result, "github:owner/repo/abc123");
    }

    #[test]
    fn test_to_flake_input_url_gitlab() {
        let url = "https://gitlab.com/group/subgroup/repo";
        let rev = "def456";
        let result = to_flake_input_url(url, rev);
        assert_eq!(result, "git+https://gitlab.com/group/subgroup/repo?rev=def456");
    }

    #[test]
    fn test_to_flake_input_url_generic() {
        let url = "https://git.sr.ht/~user/repo";
        let rev = "ghi789";
        let result = to_flake_input_url(url, rev);
        assert_eq!(result, "git+https://git.sr.ht/~user/repo?rev=ghi789");
    }

    #[test]
    fn test_render_flake_basic() {
        let repos = vec![
            NurRepo {
                name: "mic92".to_string(),
                url: "https://github.com/Mic92/nur-packages".to_string(),
                rev: "abc123".to_string(),
            },
            NurRepo {
                name: "nix-community".to_string(),
                url: "https://github.com/nix-community/nur".to_string(),
                rev: "def456".to_string(),
            },
        ];

        let result = NurFlakeGenerator::render_flake(&repos);
        assert!(result.contains("mic92"));
        assert!(result.contains("nix_community"));
        assert!(result.contains("abc123"));
        assert!(result.contains("def456"));
        assert!(result.contains("github:Mic92/nur-packages/abc123"));
        assert!(result.contains("github:nix-community/nur/def456"));
    }
}
