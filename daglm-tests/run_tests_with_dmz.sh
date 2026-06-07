#!/bin/bash
# Run tests on clean_project Layer 2 projects with DMZ monitoring

echo "=== Building Layer 2 projects with DMZ capture ==="

# Check if eBPF monitor binary exists, build if not
if [ ! -f /mnt/data1/time-2026/06/03/aya/target/release/d8_2a_user ]; then
    echo "Building eBPF monitor..."
    cd /mnt/data1/time-2026/06/03/aya
    cargo build --release 2>&1 | tail -3
fi

echo ""
echo "=== Running Rust tests (serde_ipld_dagcbor) ==="
cd /home/mdupont/dasl/IMPL/clean_project/lang/rust/serde_ipld_dagcbor
if [ -f Cargo.toml ]; then
    echo "Building and testing serde_ipld_dagcbor..."
    cargo build --release 2>&1 | tail -5
    cargo test --release 2>&1 | tail -10
else
    echo "No Cargo.toml found in serde_ipld_dagcbor"
fi

echo ""
echo "=== Running Rust tests (libipld) ==="
cd /home/mdupont/dasl/IMPL/clean_project/lang/rust/libipld
if [ -f Cargo.toml ]; then
    echo "Building and testing libipld..."
    cargo build --release 2>&1 | tail -5
    cargo test --release 2>&1 | tail -10
else
    echo "No Cargo.toml found in libipld"
fi

echo ""
echo "=== Running Go tests (go-ipld-prime) ==="
cd /home/mdupont/dasl/IMPL/clean_project/lang/go/go-ipld-prime
if [ -f go.mod ]; then
    echo "Building and testing go-ipld-prime..."
    go build ./... 2>&1 | tail -3 || true
    go test ./... -v 2>&1 | tail -20 || true
else
    echo "No go.mod found"
fi

echo ""
echo "=== Running Python tests (dag-cbrrr) ==="
cd /home/mdupont/dasl/IMPL/clean_project/lang/py/dag-cbrrr
if [ -f pyproject.toml ] || [ -f setup.py ]; then
    echo "Building and testing dag-cbrrr..."
    pip install -e . 2>&1 | tail -3 || true
    python -m pytest tests/ -v 2>&1 | tail -10 || true
else
    echo "Checking for Python tests..."
    find . -name "test*.py" -type f | head -5
fi

echo ""
echo "=== Running JavaScript tests (js-dag-cbor) ==="
cd /home/mdupont/dasl/IMPL/clean_project/lang/js/js-dag-cbor
if [ -f package.json ]; then
    echo "Building and testing js-dag-cbor..."
    npm install 2>&1 | tail -3 || true
    npm test 2>&1 | tail -10 || true
else
    echo "No package.json found"
fi

echo ""
echo "=== Test run complete ==="
echo "If running eBPF monitor, hits would have been captured."