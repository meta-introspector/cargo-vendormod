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

        # Build all binaries from this workspace
        mkVendormodBin = name: pkgs.rustPlatform.buildRustPackage {
          pname = "cargo-vendormod-${name}";
          version = "0.2.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs;
          buildAndTestSubdir = null;
          cargoBuildFlags = [ "--bin" name ];
          postInstall = ''
            mv $out/bin/${name} $out/bin/${name}
          '';
          doCheck = false;
        };

        # Build all binaries at once (more efficient)
        cargo-vendormod = pkgs.rustPlatform.buildRustPackage {
          pname = "cargo-vendormod";
          version = "0.2.0";
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs;
          doCheck = false;
        };

        graph-analysis-runner = pkgs.writeShellApplication {
          name = "graph-analysis-runner";
          runtimeInputs = [ cargo-vendormod pkgs.jq ];
          text = ''
            set -euo pipefail
            usage() { echo "Usage: graph-analysis-runner <repo-path> <output-dir>"; exit 1; }
            REPO="''${1:-}"; OUT="''${2:-}"; [ -d "$REPO" ] || usage; [ -n "$OUT" ] || usage
            mkdir -p "$OUT"
            echo "=== Analyzing $REPO ==="
            # Check for Cargo.toml
            MANIFEST="$REPO/Cargo.toml"
            if [ ! -f "$MANIFEST" ]; then
              echo "SKIP (no Cargo.toml): $REPO" > "$OUT/status.txt"
              exit 0
            fi
            # Run graph build
            graph build -w "$REPO" -o "$OUT/graph" --include-dev --include-build 2>&1 | \
              tee "$OUT/build.log" || true
            # If graph.json exists, run analysis too
            if [ -f "$OUT/graph/graph.json" ]; then
              graph analyze -i "$OUT/graph/graph.json" -o "$OUT" 2>&1 | tee -a "$OUT/analyze.log" || true
              graph visualize -i "$OUT/graph/graph.json" -O "$OUT/graph.dot" 2>&1 || true
              graph partition -i "$OUT/graph/graph.json" -o "$OUT/partitions" 2>&1 || true
              # Extract summary
              jq -r '"nodes=\(.total_nodes) edges=\(.total_edges) scc=\(.scc_count)"' \
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

          # Graph binary specifically
          graph = mkVendormodBin "graph";

          # Individual binaries for targeted use
          scanner = pkgs.runCommand "cargo-vendormod-scanner" {
            buildInputs = [ cargo-vendormod ];
          } ''
            mkdir -p $out/bin
            ln -s ${cargo-vendormod}/bin/scanner $out/bin/scanner
          '';

          extract-git-refs = pkgs.runCommand "cargo-vendormod-extract-git-refs" {
            buildInputs = [ cargo-vendormod ];
          } ''
            mkdir -p $out/bin
            ln -s ${cargo-vendormod}/bin/extract-git-refs $out/bin/extract-git-refs
          '';

          vendoring = pkgs.runCommand "cargo-vendormod-vendoring" {
            buildInputs = [ cargo-vendormod ];
          } ''
            mkdir -p $out/bin
            ln -s ${cargo-vendormod}/bin/vendoring $out/bin/vendoring
          '';
        };

        # Apps for convenient nix run usage
        apps = {
          graph = {
            type = "app";
            program = "${cargo-vendormod}/bin/graph";
          };
          analyze-repo = {
            type = "app";
            program = "${graph-analysis-runner}/bin/graph-analysis-runner";
          };
        };

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
