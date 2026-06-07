//! # Lean 4 Language Driver
//!
//! Splits mathlib-style Lean 4 projects into per-declaration flakes.
//!
//! Strategy:
//! 1. Detect Lean 4 projects via `lean-toolchain`, `lakefile.lean`, or `.lean` sources.
//! 2. Invoke the `lean-split-tool` to process declarations per-declaration
//! 3. Generate per-declaration flakes using the lean4-nix dependency template
//! 4. Return the split result so the caller can run vendoring / push.
//!
//! The driver is registered in `language_driver.rs` and discovered through
//! the global `DriverRegistry`.

use crate::language_driver::{LanguageDriver, SplitConfig, SplitResult};
use crate::lang_detect::Language;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::io::{Write};
use std::fs::File;

const LAKE4NIX_DEP_FLAKE: &str = r#"
{
  description = "MODULE_NAME";

  inputs = {
    nixpkgs.follows = "lean4-nix/nixpkgs";
    flake-parts.url = "github:hercules-ci/flake-parts";
    lean4-nix.url = "github:lenianiva/lean4-nix";
  };

  outputs = inputs @ { nixpkgs, flake-parts, lean4-nix, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "aarch64-linux"
        "x86_64-darwin"
        "x86_64-linux"
      ];

      perSystem = { system, pkgs, ... }: let
        lake2nix = pkgs.callPackage lean4-nix.lake {};
      in {
        _module.args.pkgs = import nixpkgs {
          inherit system;
          overlays = [(lean4-nix.readToolchainFile ./lean-toolchain)];
        };

        packages.default = lake2nix.mkPackage {
          name = "mathlib-module-MODULE_NAME";
          src = ./.;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs.lean; [lean-all];
        };
      };
    };
}
"#;

pub struct Lean4Driver;

impl Lean4Driver {
    pub const fn new() -> Self {
        Self
    }

    fn write_decl_flake(&self, decl_name: &str, decl_dir: &Path) -> Result<()> {
        let flake_name = decl_name.replace('.', "_");
        let flake = LAKE4NIX_DEP_FLAKE
            .replace("MODULE_NAME", decl_name)
            .replace("mathlib-module-", &format!("mathlib-module-{}", flake_name));

        std::fs::write(decl_dir.join("flake.nix"), flake)
            .with_context(|| format!("Failed to write flake.nix for {}", decl_name))?;

        Ok(())
    }
}

impl LanguageDriver for Lean4Driver {
    fn name(&self) -> &'static str {
        "lean4"
    }

    fn language(&self) -> Language {
        Language::Lean4
    }

    fn detect(&self, dir: &Path) -> bool {
        if dir.join("lean-toolchain").exists() {
            return true;
        }
        if dir.join("lakefile.lean").exists() || dir.join("lakefile.toml").exists() {
            return true;
        }
        if dir.join("Mathlib").exists() {
            return true;
        }
        if walkdir::WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .any(|e| e.ok().map(|e| e.path().extension() == Some("lean".as_ref())) == Some(true))
        {
            return true;
        }
        false
    }

    fn split_decls(&self, source_dir: &Path, config: &SplitConfig) -> Result<SplitResult> {
        let output_root = config.output_dir.join("flakes");
        std::fs::create_dir_all(&output_root)?;

        // Collect all .lean source files up to depth 4 (covers Mathlib/*/*/*/*.lean).
        let mut decls = Vec::new();
        for entry in walkdir::WalkDir::new(source_dir)
            .max_depth(8)
            .into_iter()
            .flatten()
        {
            let path = entry.path();
            if path.extension() == Some("lean".as_ref()) && path.file_name() != Some("All.lean".as_ref()) {
                decls.push(path.to_path_buf());
            }
        }

        let mut result = SplitResult::default();

        for lean_path in &decls {
            let rel = lean_path
                .strip_prefix(source_dir)
                .with_context(|| format!("{} not under {}", lean_path.display(), source_dir.display()))?;
            let stem = rel
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown");
            let module_name = rel
                .with_extension("")
                .to_string_lossy()
                .replace('/', ".");

            let decl_dir = output_root.join(module_name.replace('.', "_"));
            std::fs::create_dir_all(&decl_dir)?;

            // Copy the Lean source next to the flake so lake2nix sees a real package.
            let src_name = decl_dir.join(format!("{}.lean", stem));
            std::fs::copy(lean_path, src_name)?;

            // Mirror lean-toolchain if present.
            if let Some(root_toolchain) = find_ancestor(source_dir, "lean-toolchain") {
                let dest = decl_dir.join("lean-toolchain");
                if !dest.exists() {
                    std::fs::copy(root_toolchain, dest)?;
                }
            }

            self.write_decl_flake(&module_name, &decl_dir)?;
            result.decls.push(decl_dir);
        }

        Ok(result)
    }

    fn generate_decl_flake(
        &self,
        decl_name: &str,
        decl_dir: &Path,
        config: &SplitConfig,
    ) -> Result<()> {
        std::fs::create_dir_all(decl_dir)?;
        self.write_decl_flake(decl_name, decl_dir)
    }
}

fn find_ancestor(start: &Path, target: &str) -> Option<PathBuf> {
    let mut dir = Some(start);
    while let Some(d) = dir {
        let candidate = d.join(target);
        if candidate.exists() {
            return Some(candidate);
        }
        dir = d.parent();
    }
    None
}
