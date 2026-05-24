#!/bin/bash
# Recursive git repository scanner and cloner
# Scans all Cargo.toml files, extracts git URLs, clones to mirrors, recurses

set -e

SCAN_DIR="${1:-.}"
MIRRORS_DIR="${2:-$HOME/git/host}"
MAX_DEPTH="${3:-5}"
ITERATIONS="${4:-8}"

echo "=== Recursive Git Repository Scanner ==="
echo "Scan directory: $SCAN_DIR"
echo "Mirrors directory: $MIRRORS_DIR"
echo "Max depth: $MAX_DEPTH"
echo "Iterations: $ITERATIONS"

mkdir -p "$MIRRORS_DIR"

# Function to extract repo URL from Cargo.toml
extract_cargo_repos() {
    local dir="$1"
    local -a urls
    
    # Find all Cargo.toml files
    while IFS= read -r -d '' toml; do
        # Skip test fixtures
        if [[ "$toml" == *"tests"* ]] || [[ "$toml" == *"/invalid_"* ]]; then
            continue
        fi
        
        # Try to extract repository URL
        if [[ -f "$toml" ]]; then
            # Try [package] repository field
            repo=$(grep -m1 "^repository" "$toml" 2>/dev/null | sed 's/.*=\s*"\(.*\)"/\1/' | tr -d '"' || true)
            if [[ -n "$repo" ]] && [[ "$repo" == http* ]]; then
                echo "$repo"
            fi
            
            # Try [workspace.metadata.vendormod] or similar
            if grep -q "url\s*=" "$toml" 2>/dev/null; then
                grep "url\s*=" "$toml" | sed 's/.*=\s*"\(.*\)"/\1/' | tr -d '"' | grep -E "^https?://" || true
            fi
        fi
    done < <(find "$dir" -maxdepth 3 -name "Cargo.toml" -print0 2>/dev/null)
}

# Function to clone a repo to mirrors
clone_to_mirrors() {
    local url="$1"
    local owner_repo=$(echo "$url" | sed -E 's|https?://([^/]+)/([^/]+)(\.git)?|\2|')
    local host=$(echo "$url" | sed -E 's|https?://([^/]+)/.*|\1|')
    local owner=$(echo "$url" | sed -E 's|https?://[^/]+/([^/]+)/.*|\1|')
    
    local target_dir="$MIRRORS_DIR/$host/$owner"
    local bare_path="$target_dir/${owner_repo}.git"
    
    mkdir -p "$target_dir"
    
    if [[ -d "$bare_path" ]]; then
        echo "  Already exists: $bare_path"
        return 0
    fi
    
    echo "  Cloning: $url -> $bare_path"
    git clone --bare "$url" "$bare_path" 2>/dev/null || {
        echo "  Failed to clone: $url"
        return 1
    }
}

# Function to scan a directory for git repos
scan_directory() {
    local dir="$1"
    local depth="$2"
    
    if [[ $depth -ge $MAX_DEPTH ]]; then
        return
    fi
    
    echo "Scanning (depth $depth): $dir"
    
    # Extract repos from Cargo.toml files
    while IFS= read -r url; do
        if [[ -n "$url" ]] && [[ "$url" == http* ]]; then
            # Skip already processed
            if [[ ! " ${PROCESSED_URLS[*]} " =~ " ${url} " ]]; then
                PROCESSED_URLS+=("$url")
                clone_to_mirrors "$url"
            fi
        fi
    done < <(extract_cargo_repos "$dir")
}

# Main iteration loop
declare -a PROCESSED_URLS
declare -a SCAN_DIRS=("$SCAN_DIR")

for iter in $(seq 1 $ITERATIONS); do
    echo ""
    echo "=== Iteration $iter of $ITERATIONS ==="
    
    new_dirs=()
    
    for dir in "${SCAN_DIRS[@]}"; do
        scan_directory "$dir" 0
        
        # After cloning, add the cloned repos to scan dirs for next iteration
        if [[ -d "$MIRRORS_DIR" ]]; then
            while IFS= read -r -d '' bare_repo; do
                repo_name=$(basename "$bare_repo" .git)
                work_dir="$bare_repo"
                
                # Check if it has Cargo.toml files
                if find "$work_dir" -maxdepth 2 -name "Cargo.toml" 2>/dev/null | head -1 | grep -q .; then
                    new_dirs+=("$work_dir")
                fi
            done < <(find "$MIRRORS_DIR" -type d -name "*.git" -print0 2>/dev/null)
        fi
    done
    
    # Update scan dirs for next iteration (avoid duplicates)
    if [[ ${#new_dirs[@]} -gt 0 ]]; then
        SCAN_DIRS=("${new_dirs[@]}")
    fi
done

echo ""
echo "=== Summary ==="
echo "Total unique URLs processed: ${#PROCESSED_URLS[@]}"
echo "Mirrors directory: $MIRRORS_DIR"
find "$MIRRORS_DIR" -type d -name "*.git" | wc -l
echo "repositories in mirrors"

echo ""
echo "All done!"