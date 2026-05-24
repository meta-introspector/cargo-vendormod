# Getting Started with Cargo-Vendormod

Welcome to Cargo-Vendormod! This tutorial will guide you through the most common workflows and help you get productive quickly.

## 📋 Prerequisites

Before you begin, ensure you have:

1. **Rust toolchain** installed (1.75+)
   ```bash
   rustc --version
   cargo --version
   ```

2. **Git** installed
   ```bash
   git --version
   ```

3. **Cargo-Vendormod** built
   ```bash
   cargo build --release
   ```

## 🚀 Your First Workflow

Let's start with a complete end-to-end example. We'll analyze a workspace, generate dependencies, and create Nix flakes.

### Step 1: Navigate to Your Workspace

```bash
cd /path/to/your/rust/workspace
```

Your workspace should have a `Cargo.toml` at the root:

```toml
[workspace]
members = [
    "crates/crate-a",
    "crates/crate-b",
]
```

### Step 2: Analyze Dependencies

Build a complete dependency graph:

```bash
cargo-vendormod global-graph build \
    --workspace-path . \
    --include-dev \
    --expand-features \
    --output-dir ./analysis
```

**What this does:**
- Scans all crates in your workspace
- Resolves all dependencies (direct and transitive)
- Builds a directed graph of dependencies
- Exports to JSON for further analysis

**What you'll see:**
```
📊 Building dependency graph...
📊 Dependency graph built successfully
   - Total nodes: 42
   - Total edges: 87
   - Workspace members: 5
   - External dependencies: 37
```

### Step 3: Analyze the Graph

Get insights into your dependency structure:

```bash
cargo-vendormod global-graph analyze \
    --input-path ./analysis/graph.json \
    --output-dir ./analysis
```

**What this does:**
- Analyzes dependency patterns
- Identifies circular dependencies
- Calculates metrics (diameter, degree, etc.)
- Generates a detailed report

**What you'll see:**
```
📊 Analysis Report
==================
Node Count: 42
Edge Count: 87
Strongly Connected Components: 2
Diameter: 5
Average Degree: 2.07
...
```

### Step 4: Visualize (Optional)

Generate a visual graph (requires Graphviz):

```bash
# Generate DOT file
cargo-vendormod global-graph visualize \
    --input-path ./analysis/graph.json \
    --output-path ./analysis/graph.dot

# Convert to PNG (requires graphviz)
dot -Tpng ./analysis/graph.dot -o ./analysis/graph.png
```

Now you can view your dependency graph as an image!

### Step 5: Process Crates with Layers

Now let's process crates in the correct order:

```bash
cargo-vendormod process-crates \
    --workspace-path . \
    --output-dir ./processed \
    --generate-flakes \
    --compile-standalone \
    --layered-processing
```

**What this does:**

1. **Layer 1** - Processes external dependencies first
   - Crates from crates.io
   - GitHub dependencies
   - Ensures they're available before workspace crates

2. **Layer 2** - Processes workspace members
   - In topological order (dependencies first)
   - Each crate after its dependencies
   - Generates Nix flakes for each
   - Compiles standalone copies

**What you'll see:**
```
🏗️  Processing crates...
📦 Processing external/crate-foo...
   📁 Repository found
   🔖 Checking out branch
   📄 Generating flake.nix
   🛠️  Compiling...
   ✅ Success!

📦 Processing workspace/crate-a...
   📁 Repository found
   🔖 Checking out branch
   📄 Generating flake.nix
   🛠️  Compiling...
   ✅ Success!

✅ All crates processed successfully!
```

### Step 6: Run Complete Workflow (Alternative)

Or, combine all steps with one command:

```bash
cargo-vendormod run-workflow \
    --workspace-path . \
    --output-dir ./output \
    --use-cargo-rail \
    --workflow-type standard
```

This runs:
1. Dependency graph building
2. Layered processing
3. Post-processing scripts
4. Error checking and reporting

**What you'll see:**
```
🚀 Starting cargo-vendormod workflow
📊 Building dependency graph...
📊 Dependency graph built successfully
   - Total nodes: 42
   - Total edges: 87
   - Workspace members: 5
   - External dependencies: 37

🏗️  Processing crates...
✅ All crates processed!

✅ Workflow completed successfully!
```

## 🔄 Git Submodule Workflow

If you want to convert vendored crates to git submodules:

### Step 1: Initialize Vendoring

```bash
cargo-vendormod vendoring
```

This:
- Finds all git dependencies
- Creates bare mirror repositories
- Adds them as submodules

### Step 2: Fetch Latest Changes

```bash
cargo-vendormod fetch-upstream
```

Fetches from all upstream repositories in parallel.

### Step 3: Rebase Submodules

```bash
cargo-vendormod rebase
```

Rebases your submodules onto the latest upstream.

### Step 4: Full Sync (All Steps)

Or do it all at once:

```bash
cargo-vendormod sync
```

This runs: fetch → rebase → patch (all in parallel)

## 🎯 Common Use Cases

### Use Case 1: New Team Member Onboarding

```bash
# 1. Understand dependencies
cargo-vendormod global-graph build --workspace-path .
cargo-vendormod global-graph analyze --input-path graph.json

# 2. Explore dependency structure
cat analysis/analysis_report.md

# 3. Visualize (optional)
cargo-vendormod global-graph visualize --input-path graph.json
dot -Tpng graph.dot -o graph.png
```

### Use Case 2: Before Major Refactoring

```bash
# 1. Capture current state
cargo-vendormod global-graph build --workspace-path . --output-dir ./before

# 2. Make your changes
# ... edit code ...

# 3. Capture new state
cargo-vendormod global-graph build --workspace-path . --output-dir ./after

# 4. Compare
# Review the two analysis reports
```

### Use Case 3: Continuous Integration

```bash
# In your CI pipeline
cargo-vendormod run-workflow --workspace-path . --workflow-type ci
```

The `ci` workflow type:
- Runs in read-only mode
- Validates dependency structure
- Checks for cycles
- No compilation or modifications

### Use Case 4: Monorepo with Many Crates

```bash
# 1. Partition for parallel processing
cargo-vendormod global-graph partition \
    --input-path graph.json \
    --partition-count 8 \
    --algorithm KaMinPar

# 2. Process each partition independently
# (Use your build system to parallelize)

# 3. Or process all at once
cargo-vendormod process-crates \
    --workspace-path . \
    --output-dir ./processed \
    --generate-flakes \
    --compile-standalone
```

## 🔧 Configuration

### Set Default Options

Add to your workspace's `Cargo.toml`:

```toml
[package.metadata.cargo-vendormod]
vendor-dir = "vendor"
submodules-dir = "submodules"

[package.metadata.repo-manager]
git_path = "/usr/bin/git"
```

### Or Use CLI Flags

```bash
cargo-vendormod --submodules-path ./my-submodules global-graph build --workspace-path .
```

## 🎓 Next Steps

Now that you're familiar with the basics:

1. **Read the [USER_GUIDE.md](USER_GUIDE.md)** for detailed feature documentation
2. **Check [PROJECT_PROGRESS_TRACKER.md](PROJECT_PROGRESS_TRACKER.md)** for current status
3. **Explore examples** in the `examples/` directory
4. **Customize workflows** for your team's needs

## 🐛 Troubleshooting

### Issue: "Git tree is dirty" warning

This is normal! It just means you have uncommitted changes.

```bash
# To suppress, commit your changes first
git add .
git commit -m "Save changes before vendoring"

# Or ignore the warning (it's harmless)
```

### Issue: Permission denied

```bash
# Fix output directory permissions
chmod -R u+rwx ./processed ./analysis

# Or run with appropriate permissions
# (not recommended, prefer fixing permissions)
```

### Issue: Out of memory

```bash
# Reduce parallelism
cargo-vendormod process-crates \
    --workspace-path . \
    --max-parallel 2
```

### Issue: Command not found

```bash
# Add cargo bin to your PATH
export PATH="$HOME/.cargo/bin:$PATH"

# Or use the binary directly
./target/release/cargo-vendormod --help
```

## 💡 Pro Tips

1. **Always start with `global-graph build`** - Understand your dependencies first
2. **Use `--dry-run`** - Preview changes before making them
3. **Check `--verbose`** - Get detailed output for debugging
4. **Save analysis results** - Keep `graph.json` for historical comparison
5. **Use `--layer` flag** - Process only what you need
6. **Try different partition algorithms** - Find what works best for your graph
7. **Combine with CI** - Automate dependency checks
8. **Document your workflow** - Share with your team

## 📚 Where to Go Next

- **[CLI Reference](CLI_REFERENCE.md)** - All commands in one place
- **[User Guide](USER_GUIDE.md)** - Detailed feature documentation
- **[Quick Start](QUICK_START_GUIDE.md)** - What to do right now
- **[Examples](examples/)** - Real-world usage examples

## 🎉 You're Ready!

You now know how to:
- ✅ Analyze dependency graphs
- ✅ Process crates in correct order
- ✅ Generate Nix flakes
- ✅ Manage git submodules
- ✅ Run complete workflows

Happy crate managing! 🚀

## 🤝 Need Help?

- Check [USER_GUIDE.md](USER_GUIDE.md) for detailed documentation
- Review [PROJECT_PROGRESS_TRACKER.md](PROJECT_PROGRESS_TRACKER.md) for status
- Open an issue on GitHub for bugs or feature requests
- Join the community discussions

## 🚀 Quick Command Reference

```bash
# Analyze dependencies
cargo-vendormod global-graph build --workspace-path .

# Process crates
cargo-vendormod process-crates --workspace-path . --generate-flakes

# Full workflow
cargo-vendormod run-workflow --workspace-path .

# Git submodules
cargo-vendormod vendoring
cargo-vendormod sync

# Get help
cargo-vendormod --help
cargo-vendormod global-graph --help
```

Welcome to the Cargo-Vendormod community! 🌟
