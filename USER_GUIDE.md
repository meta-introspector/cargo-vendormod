# Cargo-Vendormod User Guide

## Introduction

Cargo-Vendormod is a powerful Rust crate manager with advanced features for dependency graph analysis, workspace management, Git integration, and Nix flake generation.

## Installation

### Prerequisites
- Rust toolchain (cargo, rustc)
- Git
- Nix (optional, for development)

### Build from Source
```bash
# Clone repo
 git clone https://github.com/your-repo/cargo-vendormod.git
 cd cargo-vendormod

# Build release
 cargo build --release

# Or with Nix
 nix develop --command cargo build --release
```

### Install
```bash
cp target/release/cargo-vendormod ~/.cargo/bin/
cargo-vendormod --help
```

## Basic Usage

### Help
```bash
cargo-vendormod --help
cargo-vendormod global-graph --help
cargo-vendormod global-graph build --help
```

## Command Reference

### Global Dependency Graph
```bash
cargo-vendormod global-graph build \
  --workspace-path /path/to/workspace \
  --include-dev \
  --include-build \
  --expand-features \
  --output-dir ./analysis

cargo-vendormod global-graph analyze \
  --input-path ./analysis/graph.json \
  --output-dir ./analysis

cargo-vendormod global-graph visualize \
  --input-path ./analysis/graph.json \
  --output-path ./analysis/graph.dot

cargo-vendormod global-graph toml-structure \
  --input-path ./analysis/graph.json \
  --output-dir ./toml-analysis

cargo-vendormod global-graph partition \
  --input-path ./analysis/graph.json \
  --partition-count 4 \
  --algorithm KaMinPar \
  --output-dir ./partitions
```

### Processing
```bash
cargo-vendormod process-crates \
  --workspace-path /path/to/workspace \
  --output-dir ./processed \
  --generate-flakes \
  --compile-standalone \
  --layered-processing

cargo-vendormod process-all-crates \
  --input-file crates.txt \
  --output-dir ./processed_all

cargo-vendormod generate-report \
  --input-file crates.txt \
  --output-dir ./reports
```

### Workflow
```bash
cargo-vendormod run-workflow \
  --workspace-path /path/to/workspace \
  --output-dir ./output \
  --use-cargo-rail \
  --workflow-type standard

cargo-vendormod workspace \
  --workspace-path /path/to/workspace \
  --recursive

cargo-vendormod workload \
  --workspace-path /path/to/workload \
  --fork-dir ./forks \
  --zkperf
```

### Git Operations
```bash
cargo-vendormod vendoring
cargo-vendormod fetch-upstream
cargo-vendormod rebase
cargo-vendormod releases
cargo-vendormod patch
cargo-vendormod status
cargo-vendormod sync
```

### Editing and Fixing
```bash
cargo-vendormod edit \
  --sort \
  --add-missing \
  --remove-unused \
  --update-versions

cargo-vendormod fix-cargo-toml
cargo-vendormod fix-edition --base-dir ./workspace --edition 2021
```

## Common Workflows

### Analyze New Workspace
```bash
cargo-vendormod global-graph build --workspace-path . --include-dev
cargo-vendormod global-graph analyze --input-path graph.json
cargo-vendormod global-graph visualize --input-path graph.json
```

### Process Workspace with Nix
```bash
cargo-vendormod process-crates \
  --workspace-path . \
  --output-dir ./processed \
  --generate-flakes \
  --compile-standalone \
  --layered-processing
```

### Git Submodule Workflow
```bash
cargo-vendormod vendoring
cargo-vendormod fetch-upstream
cargo-vendormod rebase
cargo-vendormod sync
```

### Run Complete Workflow
```bash
cargo-vendormod run-workflow --workspace-path . --workflow-type standard
```

### Parallel Build with Nix
```bash
cargo-vendormod nix-build-pipeline --flake-dir ./processed --max-parallel 8
```

### Edit Workspace Dependencies
```bash
cargo-vendormod edit --sort --add-missing --remove-unused --update-versions
```

## Output Formats

| Command | Output Files |
|---------|-------------|
| global-graph build | graph.json |
| global-graph analyze | analysis_report.md |
| global-graph visualize | graph.dot, graph.svg, graph.png |
| global-graph toml-structure | toml_structure_analysis.md, toml_structure_graph.dot |
| global-graph partition | partitioned_graph.json, partition DOT/SVG files |
| process-crates | flake.nix, compiled binaries, reports |
| run-workflow | Above outputs + script logs |

## Troubleshooting

### Common Issues

- Git tree is dirty warning: harmless if you have uncommitted changes.
- Missing cargo-tree command: `cargo install cargo-tree`
- Graphviz not installed: install by your system package manager.
- Permission denied: adjust output directory permissions.
- Out of memory: reduce parallelism with `--max-parallel`.
- Command not found: ensure cargo bin is in PATH.

## Configuration

Set defaults in workspace Cargo.toml:

```toml
[package.metadata.cargo-vendormod]
vendor-dir = "vendor"
submodules-dir = "submodules"

[package.metadata.repo-manager]
git_path = "/usr/bin/git"
```

Or use CLI flags:
```bash
cargo-vendormod --submodules-path ./my-submodules global-graph build --workspace-path .
```

## Performance Tips

- Use KaMinPar partitioning algorithm.
- Adjust parallelism for memory constraints.
- Use layers to process only needed crates.
- Dry run for preview before changes.

## Support

For issues or questions, check GitHub issues, documentation, or community forums.

## License

MIT License.
