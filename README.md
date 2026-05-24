# Cargo-Vendormod

[![Rust](https://img.shields.io/badge/Rust-1.75+-orange?logo=rust)](https://www.rust-lang.org/)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT_OR_Apache--2.0-blue)](./LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0-green)](./Cargo.toml)

A comprehensive Rust crate processing pipeline with dependency analysis, topological sorting, git repository integration, and Nix flake generation. Built for managing complex monorepos with ease.

## Overview

**Cargo-Vendormod** helps you:

- **Analyze** complex dependency graphs across workspaces (1000s of repos)
- **Process** crates in correct topological order (dependencies first)
- **Integrate** git repositories for advanced operations
- **Generate** Nix flakes for reproducible builds
- **Convert** vendored crates to git submodules with local mirrors
- **Recurse** into all submodules, crates, deps, and git modules across monorepos

### Key Capabilities

- **Global Dependency Graph**: Build, analyze, visualize, and partition dependency graphs
- **Topological Sorting**: Process dependencies before dependents with cycle detection
- **Nix Integration**: Flake generation for reproducible builds
- **Git Integration**: Clone, manage mirrors, fetch upstream, rebase, sync
- **Parallel Processing**: Rayon-based concurrent operations

## Installation

```bash
# Clone and build (requires Rust 1.75+)
git clone <repository-url>
cd cargo-vendormod
cargo build --release

# Install to path
cp target/release/cargo-vendormod ~/.cargo/bin/
```

Verify: `cargo-vendormod --help`

## Quick Start

### 1. Generate Configuration

```bash
# Generate sample config file
cargo-vendormod init-config
```

This outputs a sample config you can copy to `./vendormod.toml` or `~/.config/cargo-vendormod/config.toml`.

### 2. Use Standalone Binaries

Cargo-vendormod provides separate binaries for different operations:

```bash
# Vendoring - Git submodule operations
cargo run --bin vendoring -- init --source-repo .
cargo run --bin vendoring -- fetch-upstream
cargo run --bin vendoring -- rebase
cargo run --bin vendoring -- status
cargo run --bin vendoring -- sync
cargo run --bin vendoring -- patch

# Graph - Dependency graph analysis
cargo run --bin graph -- build --workspace-path .
cargo run --bin graph -- analyze --input-path ./analysis/graph.json
cargo run --bin graph -- visualize --input-path ./graph.json --output-path graph.dot
cargo run --bin graph -- partition --input-path ./graph.json --partition-count 8

# Processing - Crate processing
cargo run --bin processing -- crates --workspace-path .
cargo run --bin processing -- all --input-file crates.txt
cargo run --bin processing -- workflow --workspace-path .
```

### 3. Configuration

Create `vendormod.toml` in your project root:

```toml
# Git executable path
git-path = "git"

# Directories (relative or absolute)
vendor-dir = "vendor"
submodules-dir = "submodules"
mirrors-dir = "~/git"

# Branch settings
target-branch = "main"
version-branch-format = "v{}"
create-version-branches = false

# Parallel threads
default-threads = 8
```

Or use environment variables:
- `VENDORMOD_GIT_PATH`
- `VENDORMOD_VENDOR_DIR`
- `VENDORMOD_SUBMODULES_DIR`
- `VENDORMOD_MIRRORS_DIR`
- `VENDORMOD_TARGET_BRANCH`
- `VENDORMOD_CREATE_VERSION_BRANCHES`
- `VENDORMOD_THREADS`

## Binaries

| Binary | Purpose |
|--------|---------|
| `cargo-vendormod` | Main CLI dispatcher |
| `vendoring` | Git submodule operations |
| `graph` | Dependency graph analysis |
| `processing` | Crate processing and workflows |

## Command Reference

### Vendoring Commands

```
vendoring init              Initialize vendoring from source repo
vendoring fetch-upstream   Fetch from all upstream remotes
vendoring rebase           Rebase submodules onto upstream
vendoring releases         Fetch tags and create version branches
vendoring status           Show submodule status
vendoring sync             Full sync (fetch + rebase + patch)
vendoring patch            Generate .cargo/config.toml patches
```

### Graph Commands

```
graph build                 Build dependency graph from workspace
graph analyze               Analyze graph patterns and metrics
graph visualize             Generate DOT/Graphviz output
graph toml-structure        Analyze TOML schema patterns
graph partition             Partition graph for parallel processing
```

### Processing Commands

```
processing crates           Process crates in topological order
processing all              Process all crates from file list
processing workflow        Run complete workflow
processing report          Generate processing report
```

## Project Structure

```
cargo-vendormod/
├── src/
│   ├── main.rs              # CLI dispatcher
│   ├── lib.rs              # Library exports
│   ├── config.rs           # Configuration management
│   ├── args.rs             # CLI argument parsing
│   ├── bin/
│   │   ├── vendoring.rs    # Vendoring binary
│   │   ├── graph.rs        # Graph binary
│   │   └── processing.rs   # Processing binary
│   └── ...                 # Core modules
├── Cargo.toml
└── README.md
```

## Configuration

### Config File

Create `./vendormod.toml` or `~/.config/cargo-vendormod/config.toml`:

```toml
git-path = "git"
vendor-dir = "vendor"
submodules-dir = "submodules"
mirrors-dir = "~/git"
target-branch = "main"
version-branch-format = "v{}"
create-version-branches = false
default-threads = 8
```

### Environment Variables

```bash
export VENDORMOD_GIT_PATH=/usr/bin/git
export VENDORMOD_MIRRORS_DIR=~/git
export VENDORMOD_SUBMODULES_DIR=submodules
export VENDORMOD_TARGET_BRANCH=main
export VENDORMOD_THREADS=8
```

## Testing

```bash
# Run all tests
cargo test

# Run specific binary
cargo build --bin vendoring

# Build release
cargo build --release
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Git not found | Set `git-path` in config or `VENDORMOD_GIT_PATH` env var |
| Config not found | Create `./vendormod.toml` or use `--config` flag |
| Permission denied | Check `mirrors-dir` path permissions |

## License

MIT OR Apache-2.0 at your option.