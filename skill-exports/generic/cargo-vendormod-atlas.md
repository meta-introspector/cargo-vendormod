# cargo-vendormod-atlas

**Description**: Classify repositories using finite simple group theory and generate mathematical atlases. Use with atlas binaries and ZKperf coverage runners.

## Source

- ./skills/cargo-vendormod-atlas/SKILL.md

---

# Cargo-Vendormod Atlas

## Generate Atlas

```bash
cargo run --bin simple_group_atlas -- --path /path/to/repo --output ./atlas.json
cargo run --bin group_atlas -- --path /path/to/repo --output ./group_atlas.json
cargo run --bin repository_mathematical_atlas -- --path /path/to/repo --output ./repo_atlas.json
cargo run --bin enhanced_repository_mathematical_atlas -- --path /path/to/repo --output ./enhanced_atlas.json
cargo run --bin enhanced_repository_mathematical_atlas_with_zkperf -- --path /path/to/repo --output ./zkperf_atlas.json
cargo run --bin final_repository_mathematical_atlas -- --path /path/to/workspace
```

## Render Atlas Tiles

```bash
cargo run --bin simple_cli_tile_renderer -- --input ./atlas.json --output ./tiles
cargo run --bin cli_tile_renderer -- --input ./repo_atlas.json
cargo run --bin final_cli_tile_renderer -- --input ./final_atlas.json
cargo run --bin atlas_composer -- --inputs ./tiles --output ./composed_atlas.html
```

## Coverage / Benchmark

```bash
./zkperf_coverage_test_runner
./zkperf_integration_test_runner
./zkperf_coverage_performance_test_runner
./final_benchmark_runner
./integration_test_runner
./final_standalone_test_runner
```

## Key Binaries

`simple_group_atlas`, `group_atlas`, `repository_mathematical_atlas`, `enhanced_repository_mathematical_atlas`, `enhanced_repository_mathematical_atlas_with_zkperf`, `final_repository_mathematical_atlas`, plus `zkperf_*` runners.

