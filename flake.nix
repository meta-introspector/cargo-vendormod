{
  description = "Cargo Vendormod — vendor git dependencies as submodules with local mirrors";

  inputs = {
    nixpkgs.url = "git+file:///mnt/data1/git/github.com/NixOS/nixpkgs.git?ref=master";
    flake-utils.url = "git+file:///mnt/data1/git/github.com/numtide/flake-utils.git?ref=main";
            # Shared inputs overlay — all git repos declared centrally
    common-inputs = {
      url = "path:/home/mdupont/nix-common";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    #crate2nix = {
    #  url = "path:/tmp/flake-local/crate2nix";
    #  inputs.nixpkgs.follows = "nixpkgs";
    #};
    cargo-vendormod-src = {
      url = "git+file:///home/mdupont/git/solana.solfunmeme.com/cargo-vendormod";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "flake-utils";
    };
  };

  outputs = { self, nixpkgs, flake-utils, crate2nix, cargo-vendormod-src }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        nativeBuildInputs = with pkgs; [ pkg-config perl ];
        buildInputs = with pkgs; [ openssl.dev libgit2 libssh2 curl zlib nghttp2 ];

        # Build all binaries from this workspace at once
        cargo-vendormod = pkgs.rustPlatform.buildRustPackage {
          pname = "cargo-vendormod";
          version = "0.2.0";
          src = cargo-vendormod-src;
          cargoLock.lockFile = ./Cargo.lock;
          inherit nativeBuildInputs buildInputs;
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
          "nur-flake" "process_dasl_index"
        ];

        # All ~/projects/ directories that contain Cargo.toml
        #
        # Each entry produces `nix build .#graph-<name>` which outputs
        # graph.json + summary.txt in the Nix store.
        projects = {
          agave-solana-validator = /home/mdupont/projects/agave-solana-validator;
          cargo = /home/mdupont/projects/cargo;
          cargo2 = /home/mdupont/projects/cargo2;
          cargo2nix = /home/mdupont/projects/cargo2nix;
          crate2nix-zos = /home/mdupont/projects/crate2nix-zos;
          fandom-multiverse-tinystories = /home/mdupont/projects/fandom-multiverse-tinystories;
          ferron-zos = /home/mdupont/projects/ferron-zos;
          forgecode = /home/mdupont/projects/forgecode;
          fractran-vm = /home/mdupont/projects/fractran-vm;
          gibb-eri-sh = /home/mdupont/projects/gibb.eri.sh;
          lm-rs = /home/mdupont/projects/lm.rs;
          mmgroup-rust = /home/mdupont/projects/mmgroup-rust;
          monster-hash = /home/mdupont/projects/monster-hash;
          monster-shadows-math = /home/mdupont/projects/monster-shadows-math;
          nginx-generator = /home/mdupont/projects/nginx-generator;
          osm-parquet-tiles = /home/mdupont/projects/osm-parquet-tiles;
          osm-planet-torrent = /home/mdupont/projects/osm-planet-torrent;
          pastebin = /home/mdupont/projects/pastebin;
          pi-agent-rust = /home/mdupont/projects/pi_agent_rust;
          pozzoli-ettore-solfeggi = /home/mdupont/projects/pozzoli-ettore-solfeggi;
          rust = /home/mdupont/projects/rust;
          solfunmeme-dioxus = /home/mdupont/projects/solfunmeme-dioxus;
          solfunmeme-dioxus2 = /home/mdupont/projects/solfunmeme-dioxus2;
          solfunmeme-dioxus-apk = /home/mdupont/projects/solfunmeme-dioxus-apk;
          voa-borcherds-archive = /home/mdupont/projects/voa-borcherds-archive;
          zkperf = /home/mdupont/projects/zkperf;
          zos-plugins = /home/mdupont/projects/zos-plugins;
        };

        # Produce a Nix derivation for one project's dependency graph.
        # Usage:  nix build --impure .#graph-pastebin
        mkProjectGraph = name: srcPath:
          let
            # Copy project source to Nix store during eval (requires --impure).
            # Only Cargo.toml / Cargo.lock needed for metadata-based graph build.
            projectSrc = builtins.path {
              path = srcPath;
              name = "${name}-src";
              filter = path: type:
                let bn = baseNameOf path;
                in if type == "directory"
                   then !(builtins.elem bn [ ".git" "target" "vendor" "node_modules" "build" "__pycache__" ])
                   else bn == "Cargo.toml" || bn == "Cargo.lock";
            };
          in pkgs.runCommand "graph-${name}" {
            nativeBuildInputs = [ cargo-vendormod pkgs.jq ];
            # The graph binary uses pure-Rust Cargo.lock parsing (no cargo metadata shell-out).
            # Only Cargo.toml + Cargo.lock are needed — no git, no network.
            src = projectSrc;
          } ''
            cd "$src"
            graph build -w . -o "$out" --include-dev --include-build 2>&1
            jq '{ nodes: (.nodes | length), edges: (.edges | length) }' \
              "$out/graph.json" > "$out/summary.json"
          '';

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

          # Wrapper for process_dasl_index binary with all tool paths configured
          process-dasl-index = pkgs.writeShellApplication {
            name = "process-dasl-index";
            runtimeInputs = [
              cargo-vendormod
              crate2nix.packages.${system}.default
              pkgs.cargo
              pkgs.nix
            ];
            text = ''
              exec process_dasl_index \
                --crate2nix-path "${crate2nix.packages.${system}.default}/bin/crate2nix" \
                --cargo-path     "${pkgs.cargo}/bin/cargo" \
                --nix-path       "${pkgs.nix}/bin/nix" \
                "$@"
            '';
          };
          # Global graph — merged dependency graph of all ~/projects/ Rust repos
          # Contains: global_graph.json, build_order.json, upgrade_plan.json,
          #           mirrors.json, projects.json, bare_repos/, flakes/
          global-graph = pkgs.stdenv.mkDerivation {
            name = "global-graph";
            src = ./global_graph;
            phases = [ "installPhase" ];
            installPhase = ''
              mkdir -p $out
              cp -r $src/* $out/
            '';
          };
        }
        # Auto-generate package aliases for every binary
        // builtins.listToAttrs (map (n: { name = n; value = mkPkg n; }) allBinNames)
        # Auto-generate per-project graph derivations (nix build .#graph-pastebin)
        // builtins.listToAttrs (map (name:
            { name = "graph-${name}"; value = mkProjectGraph name projects.${name}; }
          ) (builtins.attrNames projects));

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
