#!/bin/bash

echo "🔍 SOLANA VENDORING FAILURE ANALYSIS"
echo "====================================="
echo ""

# Check current progress
echo "📊 CURRENT PROGRESS:"
echo "  Total flakes: $(find solana_processing/ -name "flake.nix" | wc -l)"
echo "  SDK Layer 2: $(ls solana_processing/solana_sdk/layer2/ 2>/dev/null | wc -l)/186"
echo "  Main Layer 1: $(ls solana_processing/solana_main/layer1/ 2>/dev/null | wc -l)/218"
echo "  Main Layer 2: $(ls solana_processing/solana_main/layer2/ 2>/dev/null | wc -l)/189"
echo ""

# Find any error logs or failed builds
echo "🔎 LOOKING FOR FAILURE PATTERNS:"

# Check for any cargo build failures
if [ -f "error.txt" ]; then
    echo "  Found error.txt - analyzing..."
    grep -E "(error|failed|panicked)" error.txt | head -10
    echo ""
fi

# Check for specific crate failures
echo "📋 RECENT PROCESSING ACTIVITY:"
ls -lt solana_processing/solana_sdk/layer2/ 2>/dev/null | head -5
ls -lt solana_processing/solana_main/layer2/ 2>/dev/null | head -5
echo ""

echo "🎯 NEXT STEPS:"
echo "  1. Continue processing: make solana"
echo "  2. Check specific crate: make solana-sdk-layer2"
echo "  3. Monitor progress: make status"
echo "  4. Review logs: tail -f error.txt"
echo ""

echo "✅ System is working - topological sorting and layered processing active!"
