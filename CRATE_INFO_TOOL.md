# Crate Info Tool

A tool for scanning Cargo workspaces, cloning git repositories, and building dependency graphs.

## Overview

The `crate_info` tool scans a directory for Cargo.toml files with repository URLs, clones them as bare mirrors, searches for additional URLs in cloned repos, and builds a dependency graph.

## Usage

```bash
cargo run --bin crate_info -- <input_dir> <output_dir>
```

- `<input_dir>`: Directory to scan for Cargo.toml files
- `<output_dir>`: Directory for output files (scan results, partitions, cache)

## Features

### Scanning
- Recursively scans directories for Cargo.toml files
- Extracts repository URLs from `[package] repository` fields
- Max depth: 20 (prevents infinite recursion)
- Uses canonicalize() to handle symlinks

### Cloning
- Clones repos as bare mirrors to `~/git/<host>/<owner>/<repo>.git`
- Pre-checks repos with curl before cloning (fast 404 detection)
- Caches repo status (Exists, NotFound, Deleted, Error) in `repo_cache.json`
- Detects deleted repos during git clone and marks them

### Search
- Searches cloned repos for more git URLs (8 iterations)
- Parses Cargo.toml files in cloned repos for `[package] repository` fields

### Graph Partitioning
- Builds dependency graph using petgraph
- Partitions using 3 methods:
  - `round_robin`: Distributes nodes evenly across partitions
  - `bfs`: Breadth-first partitioning for locality
  - `random`: Random partitioning for comparison

### Serialization
- Outputs:
  - `scan_report.json` - Full scan results (pretty)
  - `scan_report.min.json` - Compact JSON
  - `scan_report.toml` - TOML format
  - `partition_<method>.json` - Partition data
  - `partition_<method>_part<N>.txt` - Per-partition details

## Error Handling

- Curl pre-check avoids wasting time on 404/deleted repos
- Cache persists across runs - won't retry permanently failed repos
- Connection timeouts are retryable (RepoStatus::Error)
- NotFound (404) and Deleted (410) are not retried

## Performance

- Uses rayon for parallel operations (all CPU cores)
- Sequential cloning due to curl check + cache requirements
- Search operations are parallelized

## Example

```bash
# Scan a workspace and clone repos
cargo run --bin crate_info -- workload/workspaces/rust-native-tls output

# Results in:
# output/scan_report.json
# output/partition_*.json
# output/repo_cache.json
```