#!/bin/bash

# Comprehensive crate processing script for cargo-vendormod
# Processes all 9,485 Cargo.toml files with proper layer separation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_VENDORMOD="$SCRIPT_DIR/target/release/cargo-vendormod"
INPUT_FILE="$SCRIPT_DIR/test_subset.txt"
OUTPUT_BASE="$SCRIPT_DIR/workload/intermediates"
WORKSPACE_BASE="$SCRIPT_DIR/workload/workspaces"

# Create output directories
mkdir -p "$OUTPUT_BASE"
mkdir -p "$WORKSPACE_BASE"

echo "🚀 Starting comprehensive crate processing"
echo "📊 Total Cargo.toml files: $(wc -l < "$INPUT_FILE")"

# Process each Cargo.toml file
COUNTER=0
SUCCESS=0
FAILED=0

while IFS= read -r cargo_toml_path; do
    COUNTER=$((COUNTER + 1))
    
    # Extract crate name from path
    crate_dir="$(dirname "$cargo_toml_path")"
    crate_name="$(basename "$crate_dir")"
    
    echo ""
    echo "📦 Processing crate $COUNTER/$TOTAL: $crate_name"
    echo "📁 Path: $crate_dir"
    
    # Create workspace directory if it doesn't exist
    workspace_dir="$WORKSPACE_BASE/$crate_name"
    if [ ! -d "$workspace_dir" ]; then
        echo "🏗️  Creating workspace directory"
        mkdir -p "$workspace_dir"
        cp "$cargo_toml_path" "$workspace_dir/Cargo.toml"
        
        # Create minimal src directory if it doesn't exist
        if [ ! -d "$workspace_dir/src" ]; then
            mkdir -p "$workspace_dir/src"
            echo "// Minimal implementation for $crate_name" > "$workspace_dir/src/lib.rs"
        fi
    fi
    
    # Process with cargo-vendormod
    output_dir="$OUTPUT_BASE/${crate_name}_output"
    
    if "$CARGO_VENDORMOD" process-crates "$workspace_dir" --output-dir "$output_dir" --layer 1; then
        echo "✅ Successfully processed $crate_name (Layer 1)"
        
        # Process Layer 2 if Layer 1 succeeded
        if "$CARGO_VENDORMOD" process-crates "$workspace_dir" --output-dir "$output_dir" --layer 2; then
            echo "✅ Successfully processed $crate_name (Layer 2)"
            SUCCESS=$((SUCCESS + 1))
        else
            echo "⚠️  Layer 2 failed for $crate_name"
            FAILED=$((FAILED + 1))
        fi
    else
        echo "❌ Failed to process $crate_name (Layer 1)"
        FAILED=$((FAILED + 1))
    fi
    
    # Progress reporting
    if [ $((COUNTER % 100)) -eq 0 ]; then
        echo ""
        echo "📊 Progress: $COUNTER/$TOTAL crates processed"
        echo "✅ Success: $SUCCESS"
        echo "❌ Failed: $FAILED"
        echo "📈 Success rate: $((SUCCESS * 100 / COUNTER))%"
    fi
done < "$INPUT_FILE"

echo ""
echo "🏁 Processing complete!"
echo "📊 Total crates processed: $COUNTER"
echo "✅ Successfully processed: $SUCCESS"
echo "❌ Failed to process: $FAILED"
echo "📈 Overall success rate: $((SUCCESS * 100 / COUNTER))%"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "⚠️  Some crates failed to process. Check logs for details."
    exit 1
else
    echo ""
    echo "🎉 All crates processed successfully!"
    exit 0
fi