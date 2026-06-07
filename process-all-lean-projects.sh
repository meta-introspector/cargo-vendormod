#!/usr/bin/env nix-shell
#!nix-shell -p nix findutils coreutils -i bash

# Process All Lean Projects - Generate Nix Flakes per Declaration
# Usage: ./process-all-lean-projects.sh [output_dir]

OUTPUT_DIR="${1:-./generated-flakes}"
SCAN_DIRS=( 
  "$HOME/projects/lean-split-tool/dasl-verification"
  "$HOME/projects/arist"
  "$HOME/projects/deep_scanner_data_20260528_2200"
  "$HOME/projects/leanstall"
  "$HOME/projects/lean4"
)

echo "=== Scanning Lean Projects ==="
echo "Output directory: $OUTPUT_DIR"
echo ""

mkdir -p "$OUTPUT_DIR"

# Counter for statistics
total_files=0
total_decls=0

# Process each project
tfor scanDir in "${SCAN_DIRS[@]}"; do
  if [ -d "$scanDir" ]; then
    echo "Scanning: $scanDir"
    
    # Find all .lean files
    find "$scanDir" -name "*.lean" -type f | while read -r leanFile; do
      # Skip lakefiles and toolchain files
      if [[ "$leanFile" == *"lakefile"* ]] || [[ "$leanFile" == *"lean-toolchain"* ]]; then
        continue
      fi
      
      # Get relative path
      if [[ "$leanFile" == /mnt/data1/time-2026/02-february/22/dasl/* ]]; then
        relPath="${leanFile#/mnt/data1/time-2026/02-february/22/dasl/}"
      elif [[ "$leanFile" == "$HOME/projects/lean-split-tool/dasl-verification/"* ]]; then
        relPath="${leanFile#$HOME/projects/lean-split-tool/dasl-verification/}"
      elif [[ "$leanFile" == /mnt/data1/git/github.com/lenianiva/lean4-nix.git/* ]]; then
        relPath="${leanFile#/mnt/data1/git/github.com/lenianiva/lean4-nix.git/}"
      else
        relPath="$leanFile"
      fi
      
      # Extract module name
      if [[ "$relPath" == RequestProject/* ]]; then
        moduleName="${relPath%.lean}"
        # Convert path separators to dots
        moduleName="${moduleName//\//.}"
        outputDir="$OUTPUT_DIR/dasl/$moduleName"
      elif [[ "$relPath" == Mathlib/* ]]; then
        moduleName="${relPath%.lean}"
        outputDir="$OUTPUT_DIR/mathlib/$moduleName"
      else
        moduleName="${relPath%.lean}"
        outputDir="$OUTPUT_DIR/other/$moduleName"
      fi
      
      mkdir -p "$outputDir"
      
      # Copy the lean file
      cp "$leanFile" "$outputDir/"
      
      # Generate flake.nix based on lean4-nix template
      cat > "$outputDir/flake.nix" << FLAKE
# Auto-generated flake.nix for ${moduleName}
{
  description = "${moduleName}";

  inputs = {
    nixpkgs.follows = "lean4-nix/nixpkgs";
  };

  outputs = { self, nixpkgs }:
    let
      #[system] = "x86_64-linux";
      lake2nix = pkgs.callPackage lean4-nix.lake {};
    in {
      packages."";
    };
}
FLAKE
      
      # Generate lakefile.toml
      cat > "$outputDir/lakefile.toml" << LAKEFILE
# Auto-generated lakefile for ${moduleName}
name = "${moduleName}"

packages = [
  { name = "src", src = "." }
]

[[lean_toolchain]]
  lean_version = "4.28.0"
  lean4_nix_repo = "github:lenianiva/lean4-nix"

[[flake]]]
  nixpkgs_repo = "github:NixOS/nixpkgs"
  lean4_nix_repo = "github:lenianiva/lean4-nix"
LAKEFILE
      
      # Generate lean-toolchain
      if [ -f "$scanDir/lean-toolchain" ]; then
        cp "$scanDir/lean-toolchain" "$outputDir/lean-toolchain"
      else
        cat > "$outputDir/lean-toolchain" << TOOLCHAIN
{
  "leanprover/lean4:v4.28.0"
}
TOOLCHAIN
      fi
      
      ((total_files++))
      ((total_decls++))
      
      echo "  → $moduleName"
    done
    
    echo ""
  else
    echo "Skipping (not found): $scanDir"
  fi
done

echo "=== Summary ==="
echo "Total files processed: $total_files"
echo "Total declarations found: $total_decls"
echo ""
echo "Generated flakes in: $OUTPUT_DIR"
echo ""
echo "=== Next Steps ==="
echo "1. cd $OUTPUT_DIR"
echo "2. Run: nix develop"
echo "3. Run: lake build"
echo "4. Push to git: git push origin main"
