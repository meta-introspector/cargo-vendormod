{
  description = "n0x-pi task for vendormod-module-tile — interactive module tree visualization";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    n0x-pi.url = "git+file:///mnt/data1/git/github.com/sub0xdai/n0x-pi.git?ref=master";
    cargo-vendormod = {
      url = "git+file:///home/mdupont/git/solana.solfunmeme.com/cargo-vendormod";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = inputs@{ self, nixpkgs, n0x-pi, cargo-vendormod, ... }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages.${system};
    in {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          python3        # for http.server (testing the tile locally)
          n0x-pi.packages.${system}.pi
          # Include vendormod source tree so agent can scan module structure
          (cargo-vendormod.packages.${system}.default.overrideAttrs (_oldAttrs: {
            # Don't actually build — just provide the source tree
            buildPhase = "true";
            installPhase = "true";
          }))
        ];

        shellHook = ''
          echo "Entering n0x-pi task: vendormod-module-tile"
          echo "  Source tree: $(pwd)/../.."
          echo "  Docs: ~/DOCS/VENDORMOD_REFACTOR_MAIN_RS.md"
          echo "  Plan: ~/dasl/plan.org (search 'update jun 7')"
          echo "  Serve tile: python3 -m http.server 8765"
        '';
      };
    };
}
