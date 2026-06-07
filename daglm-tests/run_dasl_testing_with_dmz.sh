#!/bin/bash
# Run dasl-testing harnesses with DMZ 0xD8 0x2A capture
# Uses the existing fuzzing harness infrastructure in dasl-testing/

set -e

DASL_TESTING=/mnt/data1/time-2026/02-february/22/dasl/dasl-testing

echo "=== DASL Testing with DMZ Capture ==="
echo ""

# Step 1: Build eBPF monitor
echo "[1/4] Building eBPF monitor for 0xD8 0x2A detection..."
cd /mnt/data1/time-2026/06/03/aya
cargo build --release 2>&1 | tail -3

# Step 2: Build all Rust harnesses
echo ""
echo "[2/4] Building Rust harnesses..."
for h in n0_dasl serde_ipld_dagcbor libipld; do
    echo "  Building $h..."
    cd $DASL_TESTING/harnesses/$h 2>/dev/null || continue
    cargo build --release --bin round_robin 2>&1 | tail -2 || true
done

# Step 3: Run quick fuzz with round_robin (this exercises the libraries)
echo ""
echo "[3/4] Running quick fuzzing (1K iterations)..."
cd $DASL_TESTING

# Run Rust round_robin with test inputs
if [ -f harnesses/n0_dasl/fixtures/test_inputs.txt ]; then
    echo "Testing n0_dasl round_robin..."
    cargo run --release --bin round_robin -p test_n0_dasl < harnesses/n0_dasl/fixtures/test_inputs.txt 2>&1 | head -20 || true
elif [ -d fixtures/cbor ]; then
    echo "Testing n0_dasl with fixtures..."
    ls fixtures/cbor/*.hex 2>/dev/null | head -5 | while read f; do
        cat "$f" | cargo run --release --bin round_robin -p test_n0_dasl 2>&1 | head -5 || true
    done
fi

# Step 4: Show instructions for full run with monitoring
echo ""
echo "[4/4] For full monitoring run:"
echo ""
echo "  1. Start eBPF monitor in another terminal:"
echo "     sudo /mnt/data1/time-2026/06/03/aya/target/release/d8_2a_user"
echo ""
echo "  2. Run full fuzzing campaign:"
echo "     cd $DASL_TESTING && make fuzz-quick"
echo ""
echo "  3. Hits (0xD8 0x2A) will be printed by the monitor as tests run"
echo ""
echo "  4. Test inputs containing CBOR tag 42 will trigger hits like:"
echo "     hit: pid=1234 tid=1235 ip=0x555555555000 reg=rax val=0xd82a00..."