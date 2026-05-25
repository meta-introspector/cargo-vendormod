{
  description = "Cargo Vendormod — vendor git dependencies as submodules with local mirrors";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        nativeBuildInputs = with pkgs; [ pkg-config ];

        # Build all binaries from this workspace at once
        cargo-vendormod = pkgs.rustPlatform.buildRustPackage {
          pname = "cargo-vendormod";
          version = "0.2.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs;
          doCheck = false;
        };

        # All binary names known to this workspace
        allBinNames = [
          "cargo-vendormod" "vendoring" "graph" "processing"
          "scanner" "scan-mirrors" "full-scan" "extract-git-refs"
          "group_atlas" "simple_group_atlas" "workload_processor"
          "workload_report" "goal_tracker" "dasl_metadata_processor"
          "atlas_composer" "benchmark_runner" "cli_tile_renderer"
          "crate_info" "enhanced_repository_mathematical_atlas"
          "enhanced_repository_mathematical_atlas_with_zkperf"
          "final_benchmark_runner" "final_cli_tile_renderer"
          "final_repository_mathematical_atlas" "final_standalone_test_runner"
          "integration_test" "integration_test_runner"
          "project_performance_coverage_tile" "project_self_coverage_tile"
          "repository_mathematical_atlas" "simple_cli_tile_renderer"
          "simple_repository_mathematical_atlas" "standalone_test_runner"
          "test_runner" "zkperf_coverage_performance_test_runner"
          "zkperf_coverage_test_runner" "zkperf_integration_test_runner"
        ];

        # Create a package wrapping a single binary (symlink to the all-in-one build)
        mkPkg = name: pkgs.runCommand "cargo-vendormod-${name}" {
          buildInputs = [ cargo-vendormod ];
        } ''
          mkdir -p $out/bin
          ln -s ${cargo-vendormod}/bin/${name} $out/bin/${name}
        '';

        # Create a flake app entry for a single binary
        mkApp = name: {
          type = "app";
          program = "${cargo-vendormod}/bin/${name}";
        };

        # Runner that chains existing binaries: graph build -> analyze -> visualize -> partition
        # Result is a directory suitable for nix-store --add or fetchTree
        graph-analysis-runner = pkgs.writeShellApplication {
          name = "graph-analysis-runner";
          runtimeInputs = [ cargo-vendormod pkgs.jq ];
          text = ''
            set -euo pipefail
            usage() { echo "Usage: graph-analysis-runner <repo-path> <output-dir>"; exit 1; }
            REPO="''${1:-}"; OUT="''${2:-}"; [ -d "$REPO" ] || usage; [ -n "$OUT" ] || usage
            mkdir -p "$OUT"
            echo "=== Analyzing $REPO ==="
            MANIFEST="$REPO/Cargo.toml"
            if [ ! -f "$MANIFEST" ]; then
              echo "SKIP (no Cargo.toml): $REPO" > "$OUT/status.txt"
              exit 0
            fi
            graph build -w "$REPO" -o "$OUT/graph" --include-dev --include-build 2>&1 | tee "$OUT/build.log" || true
            if [ -f "$OUT/graph/graph.json" ]; then
              graph analyze -i "$OUT/graph/graph.json" -o "$OUT" 2>&1 | tee -a "$OUT/analyze.log" || true
              graph visualize -i "$OUT/graph/graph.json" -O "$OUT/graph.dot" 2>&1 || true
              graph partition -i "$OUT/graph/graph.json" -o "$OUT/partitions" 2>&1 || true
              jq -r '"nodes=" + (.total_nodes|tostring) + " edges=" + (.total_edges|tostring) + " scc=" + (.scc_count|tostring)' \
                "$OUT/analysis.json" > "$OUT/summary.txt" 2>/dev/null || true
              echo "DONE: $REPO" > "$OUT/status.txt"
            else
              echo "FAIL (no graph.json): $REPO" > "$OUT/status.txt"
            fi
          '';
        };
      in
      {
        packages = {
          default = cargo-vendormod;
          inherit graph-analysis-runner;
        }
        # Auto-generate package aliases for every binary
        // builtins.listToAttrs (map (n: { name = n; value = mkPkg n; }) allBinNames);

        apps = {
          analyze-repo = {
            type = "app";
            program = "${graph-analysis-runner}/bin/graph-analysis-runner";
          };
        }
        # Auto-generate app entries for every binary
        // builtins.listToAttrs (map (n: { name = n; value = mkApp n; }) allBinNames);

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc cargo rustfmt clippy
            openssl.dev libgit2 libssh2 curl zlib nghttp2
            pkg-config gcc glibc.dev graphviz jq
          ];
        };
      }
    );
}
