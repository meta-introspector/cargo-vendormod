# CLI Quick Reference Guide

## 📋 Command Index

### Global Dependency Graph Commands
```bash
cargo-vendormod global-graph build    # Build dependency graph
cargo-vendormod global-graph analyze  # Analyze dependency graph
cargo-vendormod global-graph visualize # Generate graph visualization
cargo-vendormod global-graph toml-structure # TOML structure analysis
cargo-vendormod global-graph partition # Partition graph for parallel processing
```

### Processing Commands
```bash
cargo-vendormod process-crates         # Layered crate processing
cargo-vendormod process-all-crates     # Process crates from list
cargo-vendormod generate-report        # Generate processing report
```

### Workflow Commands
```bash
cargo-vendormod run-workflow           # Run complete workflow
cargo-vendormod workspace              # Analyze workspace
cargo-vendormod workload               # Process workload
```

### Git/Vendoring Commands
```bash
cargo-vendormod vendoring              # Convert to submodules
cargo-vendormod fetch-upstream         # Fetch upstream changes
cargo-vendormod rebase                 # Rebase submodules
cargo-vendormod releases               # Fetch releases
cargo-vendormod patch                  # Generate patches
cargo-vendormod status                 # Show status
cargo-vendormod sync                   # Full sync
```

### Edit Commands
```bash
cargo-vendormod edit                   # Edit workspace Cargo.toml
```

### Fix Commands
```bash
cargo-vendormod fix-cargo-toml         # Fix Rust toolchain workspaces
cargo-vendormod fix-edition            # Fix edition configuration
```

### Nix Build Pipeline
```bash
cargo-vendormod nix-build-pipeline     # Run parallel Nix builds
cargo-vendormod generate-error-report  # Generate LLM error report
```

---

## 🔍 Global Dependency Graph

### Build - Create dependency graph
```bash
cargo-vendormod global-graph build \
  --workspace-path <PATH> \
  --include-dev \
  --include-build \
  --expand-features \
  --output-dir <DIR>
```

**Options:**
- `--workspace-path` - Path to workspace root (required)
- `--include-dev` - Include dev dependencies
- `--include-build` - Include build dependencies
- `--expand-features` - Expand feature dependencies
- `--output-dir` - Output directory (default: ./analysis/global_graph)

---

### Analyze - Analyze existing graph
```bash
cargo-vendormod global-graph analyze \
  --input-path <FILE> \
  --output-dir <DIR>
```

**Options:**
- `--input-path` - Path to graph.json (required)
- `--output-dir` - Output directory (default: ./analysis/global_graph)

**Output:**
- `analysis_report.md` - Analysis report
- Console statistics

---

### Visualize - Generate graph visualization
```bash
cargo-vendormod global-graph visualize \
  --input-path <FILE> \
  --output-path <FILE>
```

**Options:**
- `--input-path` - Path to graph.json (required)
- `--output-path` - Output .dot file (default: ./analysis/global_graph/graph.dot)

**Render with Graphviz:**
```bash
dot -Tpng graph.dot -o graph.png
dot -Tsvg graph.dot -o graph.svg
```

---

### TOML Structure - Analyze TOML patterns
```bash
cargo-vendormod global-graph toml-structure \
  --input-path <FILE> \
  --output-dir <DIR>
```

**Options:**
- `--input-path` - Path to graph.json (required)
- `--output-dir` - Output directory (default: ./analysis/toml_structure)

**Output:**
- `toml_structure_analysis.md` - Analysis report
- `toml_structure_graph.dot` - Graph visualization

---

### Partition - Partition graph for parallel processing
```bash
cargo-vendormod global-graph partition \
  --input-path <FILE> \
  --partition-count <N> \
  --balance-factor <FACTOR> \
  --algorithm <ALGO> \
  --output-dir <DIR>
```

**Options:**
- `--input-path` - Path to graph.json (required)
- `--partition-count` - Number of partitions (default: 8)
- `--balance-factor` - Balance factor 0.0-1.0 (default: 1.1)
- `--algorithm` - Algorithm: KaMinPar, Metis, Louvain, KernighanLin, Spectral, Greedy (default: KaMinPar)
- `--output-dir` - Output directory (default: ./analysis/partitions)

**Algorithms:**
- `KaMinPar` - Fast, good quality (recommended for most cases)
- `Metis` - High quality, slower
- `Louvain` - Community detection
- `Greedy` - Fastest, lower quality
- `KernighanLin` - Small graphs
- `Spectral` - Theoretical, small graphs

---

## 🏗️ Processing Commands

### Process Crates - Layered crate processing
```bash
cargo-vendormod process-crates \
  --workspace-path <PATH> \
  --output-dir <DIR> \
  --generate-flakes \
  --compile-standalone \
  --layered-processing \
  --layer <LAYER>
```

**Options:**
- `--workspace-path` - Path to workspace root (required)
- `--output-dir` - Output directory (default: ./processed)
- `--generate-flakes` - Generate Nix flakes
- `--compile-standalone` - Compile crates standalone
- `--layered-processing` - Use layered processing (default: true)
- `--layer` - Process only layer: 1 (external) or 2 (workspace)

**Layers:**
- **Layer 1** - External dependencies (crates.io, GitHub)
- **Layer 2** - Workspace members

---

### Process All Crates - Batch processing
```bash
cargo-vendormod process-all-crates \
  --input-file <FILE> \
  --output-dir <DIR> \
  --max-parallel <N>
```

**Options:**
- `--input-file` - File with list of Cargo.toml paths (required)
- `--output-dir` - Output directory (default: ./processed_all)
- `--max-parallel` - Max parallel processes (default: 4)

**Input File Format:**
```
/path/to/crate1/Cargo.toml
/path/to/crate2/Cargo.toml
/path/to/crate3/Cargo.toml
```

---

### Generate Report - Processing report
```bash
cargo-vendormod generate-report \
  --input-file <FILE> \
  --output-dir <DIR>
```

**Options:**
- `--input-file` - File with list of Cargo.toml paths (required)
- `--output-dir` - Directory to analyze (default: ./processed_all)

---

## 🚀 Workflow Commands

### Run Workflow - Complete processing pipeline
```bash
cargo-vendormod run-workflow \
  --workspace-path <PATH> \
  --output-dir <DIR> \
  --use-cargo-rail \
  --workflow-type <TYPE>
```

**Options:**
- `--workspace-path` - Path to workspace (required)
- `--output-dir` - Output directory (default: ./processed)
- `--use-cargo-rail` - Use cargo-rail for git ops (default: true)
- `--workflow-type` - Type: standard, minimal, or ci (default: standard)

**Workflow Types:**
- `standard` - Full processing with scripts
- `minimal` - Analysis only, no compilation
- `ci` - Read-only, no compilation

---

### Workspace - Analyze workspace
```bash
cargo-vendormod workspace \
  --workspace-path <PATH> \
  --recursive
```

**Options:**
- `--workspace-path` - Path to workspace root (required)
- `--recursive` - Process all workspace members

---

### Workload - Process workload
```bash
cargo-vendormod workload \
  --workspace-path <PATH> \
  --fork-dir <DIR> \
  --zkperf
```

**Options:**
- `--workspace-path` - Path to workspace root (required)
- `--fork-dir` - Directory for local forks (default: ./forks)
- `--zkperf` - Apply zkperf annotations

---

## 🔄 Git/Vendoring Commands

### Vendoring - Convert to submodules
```bash
cargo-vendormod vendoring
```
Converts vendored crates to git submodules.

---

### Fetch Upstream - Fetch latest changes
```bash
cargo-vendormod fetch-upstream
```
Fetches upstream changes into all bare mirrors (parallel, 24 CPUs).

---

### Rebase - Rebase submodules
```bash
cargo-vendormod rebase
```
Rebases all submodule changes onto latest upstream.

---

### Releases - Fetch releases
```bash
cargo-vendormod releases
```
Fetches all release tags and creates version branches in bare mirrors.

---

### Patch - Generate patches
```bash
cargo-vendormod patch
```
Generates/updates `.cargo/config.toml` patches for vendored crates.

---

### Status - Show status
```bash
cargo-vendormod status
```
Shows status of all submodules vs upstream/bare.

---

### Sync - Full sync
```bash
cargo-vendormod sync
```
Full sync: fetch upstream → rebase → update patches (all parallel).

---

### Edit - Edit workspace Cargo.toml
```bash
cargo-vendormod edit \
  --sort \
  --add-missing \
  --remove-unused \
  --update-versions
```

**Options:**
- `--sort` - Sort dependencies alphabetically
- `--add-missing` - Add dependencies from submodules
- `--remove-unused` - Remove dependencies with missing paths
- `--update-versions` - Update version fields from Cargo.toml

---

### Workspace - Process workspace
```bash
cargo-vendormod workspace \
  --workspace-path <PATH> \
  --recursive
```

**Options:**
- `--workspace-path` - Path to workspace root (required)
- `--recursive` - Process all workspace members

---

## 🔧 Fix Commands

### Fix Cargo.toml - Fix Rust toolchain workspaces
```bash
cargo-vendormod fix-cargo-toml
```
Fixes Cargo.toml files for Rust toolchain components.

---

### Fix Edition - Fix edition configuration
```bash
cargo-vendormod fix-edition \
  --base-dir <PATH> \
  --edition <EDITION>
```

**Options:**
- `--base-dir` - Base directory containing workspaces (required)
- `--edition` - Rust edition to set (default: 2021)

---

## ☁️ Nix Build Pipeline

### Nix Build Pipeline - Run parallel Nix builds
```bash
cargo-vendormod nix-build-pipeline \
  --flake-dir <DIR> \
  --max-parallel <N> \
  --max-retries <N> \
  --timeout-seconds <N> \
  --output-dir <DIR> \
  --log-dir <DIR> \
  --workspace-path <PATH>
```

**Options:**
- `--flake-dir` - Directory with Nix flakes (required)
- `--max-parallel` - Max parallel builds (default: 8)
- `--max-retries` - Max retries for failed builds (default: 2)
- `--timeout-seconds` - Build timeout in seconds (default: 3600)
- `--output-dir` - Build artifacts directory (default: ./nix_builds)
- `--log-dir` - Build logs directory (default: ./build_logs)
- `--workspace-path` - Workspace for dependency analysis (optional)

---

### Generate Error Report - LLM error analysis
```bash
cargo-vendormod generate-error-report \
  --results-file <FILE> \
  --output-path <FILE>
```

**Options:**
- `--results-file` - Path to build_results.json (required)
- `--output-path` - Output path for report (default: ./llm_error_report.md)

---

## 🌍 Global Options

Available on all commands:

```bash
--root-dir PATH              Root directory (default: .)
--manifest-path PATH         Path to Cargo.toml
--submodules-path PATH       Submodules directory
--mirrors-path PATH          Git mirrors directory
--vendor-dir PATH            Vendor directory
--target-branch BRANCH       Target branch for deps
--create-version-branches    Create version branches in mirrors
--version-branch-format FMT  Version branch format (default: v{})
--dry-run                    Perform dry run (no changes)
--output-file FILE           Write JSON plan to file
--verbose                    Enable verbose output
--help                       Show help
```

---

## 📊 Common Workflows

### 1. Analyze New Workspace
```bash
# Build dependency graph
cargo-vendormod global-graph build --workspace-path . --include-dev

# Analyze dependencies
cargo-vendormod global-graph analyze --input-path graph.json

# Visualize
cargo-vendormod global-graph visualize --input-path graph.json
```

### 2. Process Workspace with Nix
```bash
# Process crates with flakes and compilation
cargo-vendormod process-crates \
    --workspace-path . \
    --output-dir ./processed \
    --generate-flakes \
    --compile-standalone \
    --layered-processing
```

### 3. Full Git Submodule Workflow
```bash
# Convert to submodules
cargo-vendormod vendoring

# Sync everything
cargo-vendormod sync
```

### 4. Run Complete Workflow
```bash
# Full analysis + processing pipeline
cargo-vendormod run-workflow --workspace-path . --workflow-type standard
```

### 5. Parallel Build with Nix
```bash
# Process crates
g cargo-vendormod process-crates --workspace-path . --generate-flakes

# Run parallel builds
cargo-vendormod nix-build-pipeline --flake-dir ./processed --max-parallel 8
```

### 6. Edit Workspace Dependencies
```bash
cargo-vendormod edit \
    --sort \
    --add-missing \
    --remove-unused \
    --update-versions
```

---

## 🔍 Filtering Options

### By Layer
```bash
# Process only external dependencies
cargo-vendormod process-crates --workspace-path . --layer 1

# Process only workspace members
cargo-vendormod process-crates --workspace-path . --layer 2
```

### By Dependency Type
```bash
# Include all dependency types
cargo-vendormod global-graph build \
    --workspace-path . \
    --include-dev \
    --include-build \
    --expand-features
```

### By Output
```bash
# Generate multiple outputs
cargo-vendormod process-crates \
    --workspace-path . \
    --generate-flakes \
    --compile-standalone
```

---

## 📝 Output Formats

| Command | Outputs |
|---------|---------|
| `global-graph build` | `graph.json` |
| `global-graph analyze` | `analysis_report.md` |
| `global-graph visualize` | `graph.dot`, `graph.svg`, `graph.png` |
| `global-graph toml-structure` | `toml_structure_analysis.md`, `toml_structure_graph.dot` |
| `global-graph partition` | `partitioned_graph.json`, partition DOT/SVG files |
| `process-crates` | `flake.nix`, compiled binaries, reports |
| `run-workflow` | All above + script outputs |

---

## ⚡ Performance Tips

1. **Use KaMinPar** - Fast partitioning for most graphs
2. **Limit parallelism** - Reduce `--max-parallel` on memory-constrained systems
3. **Use layers** - Process only needed layers with `--layer`
4. **Skip compilation** - Use `--workflow-type minimal` for analysis only
5. **Dry run first** - Use `--dry-run` to preview changes

---

## 🐛 Troubleshooting

### Command not found
```bash
# Ensure cargo bin is in PATH
export PATH="$HOME/.cargo/bin:$PATH"
```

### Permission denied
```bash
# Fix permissions on output directories
chmod -R u+rwx ./processed ./analysis
```

### Git errors
```bash
# Verify git is installed
git --version

# Check git configuration
git config --list
```

### Out of memory
```bash
# Reduce parallelism
cargo-vendormod process-crates --workspace-path . --max-parallel 2
```

For more help, see [USER_GUIDE.md](USER_GUIDE.md)
