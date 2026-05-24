#!/bin/bash

echo "🔍 Finding missing crates..."

# Extract crate names from input file
input_crates=()
while IFS= read -r line; do
    if [[ -n "$line" && "$line" != "="* ]]; then
        crate_dir="$(dirname "$line")"
        crate_name="$(basename "$crate_dir")"
        input_crates+=("$crate_name")
    fi
done < all_cargo_tomls.txt

echo "📊 Found ${#input_crates[@]} crates in input file"

# Find processed crates
processed_crates=()
for workspace in final_processing_output/workspaces/*/; do
    if [ -d "$workspace" ]; then
        workspace_name="$(basename "$workspace")"
        
        # Check if this workspace has a Cargo.toml that matches our input
        if [ -f "$workspace/Cargo.toml" ]; then
            # Extract the actual crate name from Cargo.toml
            crate_name="$(grep -m 1 '^name =' "$workspace/Cargo.toml" | cut -d'"' -f2)"
            if [ -n "$crate_name" ]; then
                processed_crates+=("$crate_name")
            else
                # Fallback to workspace name
                processed_crates+=("$workspace_name")
            fi
        fi
    fi
done

echo "📊 Found ${#processed_crates[@]} processed crates"

# Find missing crates
echo "🔍 Comparing..."
missing_count=0
for input_crate in "${input_crates[@]}"; do
    found=false
    for processed_crate in "${processed_crates[@]}"; do
        if [[ "$input_crate" == "$processed_crate" ]]; then
            found=true
            break
        fi
    done
    
    if [ "$found" = false ]; then
        missing_count=$((missing_count + 1))
    fi
done

echo "📊 Missing crates: $missing_count"
echo "📊 Processed crates: ${#processed_crates[@]}"
echo "📊 Success rate: $(((processed_crates_count * 100) / ${#input_crates[@]}))%"

echo "✅ Report complete!"