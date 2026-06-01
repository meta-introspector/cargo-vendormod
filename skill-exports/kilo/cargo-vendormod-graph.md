---
name: cargo-vendormod-graph
description: Build, analyze, partition, and visualize Rust dependency graphs with cargo-vendormod. Use for global-graph commands.
---

# Cargo-Vendormod Graph Operations

## Build Graph

```bash
cargo run --bin cargo-vendormod -- BuildGraph \
  --workspace-path . \
  --output-dir ./graph_output \
  --include-dev \
  --include-build \
  --expand-features
```

Returns `graph.json` with nodes, edges, features, TOML structures, SCCs, and metrics.

## Analyze Graph

```bash
cargo run --bin cargo-vendormod -- AnalyzeGraph \
  --input-path ./graph_output/graph.json \
  --output-dir ./analysis
```

Reports: node/edge counts, workspace members, direct/transitive/feature-gated deps, SCCs, diameter, average degree.

## Visualize

```bash
cargo run --bin cargo-vendormod -- VisualizeGraph \
  --input-path ./graph_output/graph.json \
  --output-path ./graph.dot
dot -Tsvg graph.dot -o graph.svg
```

## Partition Graph

```bash
cargo run --bin cargo-vendormod -- PartitionGraph \
  --input-path ./graph_output/graph.json \
  --partition-count 8 \
  --algorithm greedy \
  --output-dir ./partitions
```

Supported algorithms: `greedy`, `kaminpar`, `metis`, `louvain`, `kernighanlin`, `spectral`.

## Key Library Types

- `GlobalDependencyGraphBuilder` — construct the graph
- `GlobalDependencyGraph` — result container (nodes, edges, metrics, topological orders)
- `DependencyNode` / `DependencyEdge` — graph elements
- `PartitioningConfig` / `GraphPartition` — partition inputs/outputs

## Library Entry Points

```rust
use cargo_vendormod::global_dep_graph::{GlobalDependencyGraphBuilder, PartitioningConfig, PartitioningAlgorithm};

let mut builder = GlobalDependencyGraphBuilder::new(workspace_path);
builder.set_options(include_dev, include_build, expand_features);
let graph = builder.build_global_graph()?;
```
