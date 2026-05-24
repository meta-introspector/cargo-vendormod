#!/bin/bash

# Improved comprehensive crate processing script for cargo-vendormod
# Handles 9,485 Cargo.toml files with proper error handling and parallel processing

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CARGO_VENDORMOD="$SCRIPT_DIR/target/release/cargo-vendormod"
INPUT_FILE="$SCRIPT_DIR/test_subset_10.txt"
OUTPUT_BASE="$SCRIPT_DIR/workload/intermediates"
WORKSPACE_BASE="$SCRIPT_DIR/workload/workspaces"
LOG_DIR="$SCRIPT_DIR/logs"

# Create directories
mkdir -p "$OUTPUT_BASE"
mkdir -p "$WORKSPACE_BASE"
mkdir -p "$LOG_DIR"

# Configuration
MAX_PARALLEL=2  # Number of parallel processes
BATCH_SIZE=5  # Number of crates to process in each batch

echo "🚀 Starting comprehensive crate processing"
echo "📊 Total Cargo.toml files: $(wc -l < "$INPUT_FILE")"
echo "🔧 Configuration: $MAX_PARALLEL parallel processes, $BATCH_SIZE batch size"

# Function to process a single crate
process_crate() {
    local cargo_toml_path="$1"
    local crate_id="$2"
    local log_file="$LOG_DIR/crate_${crate_id}.log"
    
    # Extract crate name from path
    crate_dir="$(dirname "$cargo_toml_path")"
    crate_name="$(basename "$crate_dir")"
    
    echo "📦 Processing crate $crate_id: $crate_name" | tee "$log_file"
    echo "📁 Path: $crate_dir" | tee -a "$log_file"
    
    # Create workspace directory if it doesn't exist
    workspace_dir="$WORKSPACE_BASE/$crate_name"
    if [ ! -d "$workspace_dir" ]; then
        echo "🏗️  Creating workspace directory" | tee -a "$log_file"
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
    
    # Process Layer 1: External Dependencies
    echo "🔹 Processing Layer 1: External Dependencies" | tee -a "$log_file"
    if "$CARGO_VENDORMOD" process-crates "$workspace_dir" --output-dir "$output_dir" --layer 1 >> "$log_file" 2>&1; then
        echo "✅ Successfully processed $crate_name (Layer 1)" | tee -a "$log_file"
        
        # Process Layer 2: Workspace Members
        echo "🔹 Processing Layer 2: Workspace Members" | tee -a "$log_file"
        if "$CARGO_VENDORMOD" process-crates "$workspace_dir" --output-dir "$output_dir" --layer 2 >> "$log_file" 2>&1; then
            echo "✅ Successfully processed $crate_name (Layer 2)" | tee -a "$log_file"
            echo "SUCCESS" > "$log_file.success"
            return 0
        else
            echo "⚠️  Layer 2 failed for $crate_name" | tee -a "$log_file"
            echo "PARTIAL" > "$log_file.partial"
            return 1
        fi
    else
        echo "❌ Failed to process $crate_name (Layer 1)" | tee -a "$log_file"
        echo "FAILED" > "$log_file.failed"
        return 1
    fi
}

# Function to process a batch of crates
process_batch() {
    local batch_file="$1"
    local batch_id="$2"
    local batch_log="$LOG_DIR/batch_${batch_id}.log"
    
    echo "📦 Processing batch $batch_id" > "$batch_log"
    
    local success_count=0
    local failed_count=0
    local partial_count=0
    
    while IFS= read -r line; do
        local cargo_toml_path="$(echo "$line" | cut -d' ' -f1)"
        local crate_id="$(echo "$line" | cut -d' ' -f2)"
        
        if process_crate "$cargo_toml_path" "$crate_id" >> "$batch_log" 2>&1; then
            success_count=$((success_count + 1))
        else
            # Check if it was a partial success
            if [ -f "$LOG_DIR/crate_${crate_id}.partial" ]; then
                partial_count=$((partial_count + 1))
            else
                failed_count=$((failed_count + 1))
            fi
        fi
    done < "$batch_file"
    
    echo "📊 Batch $batch_id complete: $success_count success, $partial_count partial, $failed_count failed" >> "$batch_log"
    echo "BATCH_COMPLETE $success_count $partial_count $failed_count" > "$batch_log.complete"
}

# Main processing loop
echo "🏗️  Preparing batches..."

# Create numbered file with crate IDs
cat -n "$INPUT_FILE" > numbered_crates.txt
TOTAL_CRATES=$(wc -l < numbered_crates.txt)

# Split into batches
BATCH_COUNT=$(( (TOTAL_CRATES + BATCH_SIZE - 1) / BATCH_SIZE ))
echo "📦 Total batches: $BATCH_COUNT"

# Process batches in parallel
for ((batch_id=1; batch_id<=$BATCH_COUNT; batch_id++)); do
    echo "📦 Preparing batch $batch_id/$BATCH_COUNT"
    
    # Calculate start and end lines for this batch
    start_line=$(( (batch_id - 1) * BATCH_SIZE + 1 ))
    end_line=$(( batch_id * BATCH_SIZE ))
    
    # Create batch file
    batch_file="$LOG_DIR/batch_${batch_id}.txt"
    sed -n "${start_line},${end_line}p" numbered_crates.txt > "$batch_file"
    
    # Process batch (with parallel processing)
    process_batch "$batch_file" "$batch_id" &
    
    # Limit parallel processes
    if [ $((batch_id % MAX_PARALLEL)) -eq 0 ]; then
        echo "🔄 Waiting for parallel processes to complete..."
        wait
    fi
done

# Wait for all remaining processes
wait

echo "🏁 All batches processed!"

# Generate summary report
echo "📊 Generating summary report..."
SUCCESS=0
PARTIAL=0
FAILED=0

for ((batch_id=1; batch_id<=$BATCH_COUNT; batch_id++)); do
    batch_complete="$LOG_DIR/batch_${batch_id}.log.complete"
    if [ -f "$batch_complete" ]; then
        read status s p f < "$batch_complete"
        SUCCESS=$((SUCCESS + s))
        PARTIAL=$((PARTIAL + p))
        FAILED=$((FAILED + f))
    fi
done

TOTAL_PROCESSED=$((SUCCESS + PARTIAL + FAILED))

echo ""
echo "🏁 Processing complete!"
echo "📊 Total crates processed: $TOTAL_PROCESSED/$TOTAL_CRATES"
echo "✅ Successfully processed: $SUCCESS"
echo "⚠️  Partially processed: $PARTIAL"
echo "❌ Failed to process: $FAILED"
echo "📈 Overall success rate: $((SUCCESS * 100 / TOTAL_PROCESSED))%"

if [ $FAILED -gt 0 ]; then
    echo ""
    echo "⚠️  Some crates failed to process. Check logs in $LOG_DIR for details."
    echo "📁 Failed crates: $(find "$LOG_DIR" -name "*.failed" | wc -l)"
    exit 1
else
    echo ""
    echo "🎉 All crates processed successfully!"
    exit 0
fi