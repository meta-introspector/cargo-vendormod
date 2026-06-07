#!/bin/bash
# Run tests on Layer 2 Direct Users while monitoring for 0xD8 0x2A

set -e

echo "=== Layer 2 Direct Users Test Runner ==="
echo "Building and testing projects that directly encode 0xD8 0x2A..."

# Step 1: Build eBPF monitor first
echo "[1/3] Building eBPF monitor..."
cd /mnt/data1/time-2026/06/03/aya && cargo build --release 2>&1 | tail -5

# Step 2: Build each Layer 2 project
echo ""
echo "[2/3] Building Layer 2 projects..."

# Rust: serde_ipld_dagcbor-escaped
echo "Building: serde_ipld_dagcbor-escaped"
cd /home/mdupont/dasl/rust/serde_ipld_dagcbor-escaped
cargo build --release 2>&1 | tail -3 || echo "Build failed (expected for some)"

# Python: dag_cbrrr - find and run tests
echo "Finding: dag_cbrrr Python tests..."
cd /home/mdupont/dasl/lang/dag-cbor
find . -name "test*.py" -type f 2>/dev/null | head -5

# Go: go-ipld-prime
echo "Building: go-ipld-prime"
cd /home/mdupont/dasl/lang/go-ipld-prime
go build ./... 2>&1 | tail -3 || echo "Go build check"

# JS: ipld-dag-cbor
echo "Checking: js-dag-cbor"
cd /home/mdupont/dasl/lang/js-dag-cbor
ls -la package.json 2>/dev/null && cat package.json | head -10

# Java: java-ipld-cbor
echo "Checking: java-ipld-cbor"
cd /home/mdupont/dasl/lang/java-ipld-cbor
ls -la src/main/java/*.java 2>/dev/null | head -5

# C: zcbor
echo "Checking: zcbor"
cd /home/mdupont/dasl/lang/zcbor
ls -la *.c 2>/dev/null | head -5

# Step 3: Run tests with perf recording
echo ""
echo "[3/3] Running tests (manual step)..."
echo "Monitor needs to run separately:"
echo "  sudo /mnt/data1/time-2026/06/03/aya/target/release/d8_2a_user"
echo ""
echo "Then run actual tests to capture hits"