# cargo-vendormod-processing

Process Rust crates in layered topological order and generate Nix flakes. Use with the `processing` binary and `workflow` runner.

## Instructions

# Cargo-Vendormod Processing

## Process All Crates

```bash
cargo run --bin processing -- crates \
  --workspace-path /path/to/workspace \
  --output-dir ./processed \
  --generate-flakes \
  --compile-standalone \
  --layered-processing \
  --max-parallel 8
```

## Process from File List

```bash
cargo run --bin processing -- all \
  --input-file crates.txt \
  --output-dir ./output \
  --max-parallel 8
```

## Run Workflow

```bash
cargo run --bin processing -- workflow \
  --workspace-path /path/to/workspace \
  --output-dir ./output \
  --workflow-type standard|minimal|ci
```

## Generate Report

```bash
cargo run --bin processing -- report \
  --workspace-path /path/to/workspace \
  --output-dir ./crate_report
```

## Key Library Types

- `LayerProcessor`
- `ProcessResult`
- `Layer`

## Key Options

- `--generate-flakes`
- `--compile-standalone`
- `--layered-processing`
- `--max-parallel`
- `--workflow-type` standard|minimal|ci

## Examples

- Run the skill workflow as documented in the source.
