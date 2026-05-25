---
name: crate2nix-build
description: Build Rust crates using crate2nix-generated Nix derivations, where each dependency crate becomes its own separate Nix store path. Use when the user asks to build projects with crate2nix, generate Cargo-generated.nix, create Nix packages from Cargo projects, or verify Nix-store linking. Triggers on: "crate2nix", "nix build", "build crate", "Cargo-generated.nix", "buildRustCrate".
---

# Crate2nix Build

Build Rust projects using crate2nix, where each dependency crate links from the Nix store.

## Architecture

```
crate2nix generate → Cargo-generated.nix → buildRustCrate for each dep → Nix store paths
                                            ↓
                                    rootCrate (final binary) → linked via --extern
```

Each dependency IS its own Nix store path — no `~/.cargo/registry/` usage (~4.7GB avoided).

## Prerequisites

- `crate2nix` available: `nix run nixpkgs#crate2nix`
- Project with `Cargo.toml` + `Cargo.lock`

## Generating Derivations

```bash
# Generate Cargo-generated.nix for a project
nix run nixpkgs#crate2nix -- generate \
  -f /path/to/project/Cargo.toml \
  -o crate2nix_output/<project>/Cargo-generated.nix
```

## Building

```bash
# Build via the project flake
nix build --impure ./crate2nix_output#<project-name>

# Output: result/bin/<binary> (ELF dynamically linked via Nix glibc)
```

## Output Structure

```
crate2nix_output/
├── flake.nix                    # Wraps all projects
├── flake.lock                   # Pinned inputs
└── <project-name>/
    └── Cargo-generated.nix      # Full Nix build expression
```

The flake uses:
- `nixpkgs` for system packages
- `flake-parts` for multi-system config
- `rust-overlay` for nightly Rust toolchain
- `buildRustCrateForPkgs` override

## Flake Attribute Handling

The flake handles all three output patterns found in `Cargo-generated.nix`:

```nix
# Pattern 1 — projects with rootCrate (most common):
packages.<project> = callPackage ./<project>/Cargo-generated.nix {
  inherit pkgs buildRustCrateForPkgs;
}.rootCrate.build;

# Pattern 2 — workspace-only projects (no rootCrate):
packages.<project> = callPackage ./<project>/Cargo-generated.nix {
  inherit pkgs buildRustCrateForPkgs;
}.allWorkspaceMembers;
```

## Verification

```bash
# Check that deps are Nix store paths
nix-store -qR result | grep "rust_"      # Shows individual crate derivations

# Check binary linking
file result/bin/*                         # Should show ELF + dynamically linked

# Confirms no cargo registry pollution
du -sh ~/.cargo/registry/                 # Should show 0 or unchanged
```

## Examples

```bash
# Generate + build nginx-generator
nix run nixpkgs#crate2nix -- generate -f ~/projects/nginx-generator/Cargo.toml \
  -o crate2nix_output/nginx-generator/Cargo-generated.nix
nix build --impure ./crate2nix_output#nginx-generator

# Batch build all projects
make crate2nix-build-all

# Build a specific project
make crate2nix-build-forgecode
```

## Troubleshooting

- **"attribute 'rootCrate' missing"**: Project has no rootCrate; the flake falls back to `allWorkspaceMembers`
- **Build fails on type error**: The issue is in the project's source code, not crate2nix
- **"cannot find -lgcc_s"**: Missing build dependencies; add to `nativeBuildInputs` in the flake
