---
name: cargo-vendormod-tests
description: Run and extend the cargo-vendormod Rust test suite. Use when the user asks to run tests, add unit tests, check coverage, or debug test failures for cargo-vendormod.
license: MIT
compatibility: cross-agent
metadata:
  imported: true
  source: ./skills/cargo-vendormod-tests/SKILL.md
---

# Cargo-Vendormod Test Workflow

## Run Existing Tests

```bash
# Run all tests
cargo test

# Run specific test target
cargo test --test basic_tests
cargo test --test global_graph_tests
cargo test --test topological_tests
cargo test --test integration_tests
cargo test --test layer_processor_tests

# Run lib module tests
cargo test --lib
```

## Key Test Coverage

| File | Covers |
|------|--------|
| tests/basic_tests.rs | Config defaults, version branch formatting, Args construction, sample config |
| tests/global_graph_tests.rs | DependencyNode, DependencyEdge, DependencyEdgeType, PartitioningAlgorithm, FeatureSet, GraphMetrics |
| tests/topological_tests.rs | Dependency ordering, workspace vs external filtering, cycle detection |
| tests/integration_tests.rs | Filesystem ops, temp dirs, Cargo.toml read/write, error handling |
| tests/layer_processor_tests.rs | Graph node/edge lookup, node map consistency, external vs workspace |

## Add a New Unit Test

1. Identify the module under `src/` that lacks coverage
2. Open the file and add a `#[cfg(test)] mod tests` block at the bottom
3. Use `tempfile::tempdir()` for filesystem tests
4. Run `cargo test` and fix compiler errors before committing
5. Commit with a clear message

## Adding a New Integration Test

1. Create a new file under `tests/` (e.g. `tests/my_feature_tests.rs`)
2. Import `cargo_vendormod::` types
3. Use `tempfile` and real filesystem operations
4. Add to repository and run `cargo test --test my_feature_tests`
