#!/bin/bash
# Benchmark script for cargo-vendormod workload processing

set -e

cd "$(dirname "$0")/.."

echo "========================================"
echo "  Cargo-Vendormod Workload Benchmark"
echo "========================================"

# Run benchmark
echo ""
echo "📊 Running workload analysis with timing..."
time nix develop -f ../../flake.nix -c cargo run --bin workload_processor -- . 2>&1

echo ""
echo "========================================"
echo "  Benchmark Complete"
echo "========================================"