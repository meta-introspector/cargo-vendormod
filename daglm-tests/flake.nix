{
  description = "eBPF D8 2A monitor for CBOR tag 42 detection";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    cargo2nix = {
      url = "github:cargo2nix/cargo2nix/release-0.12";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, cargo2nix }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        cargo2nixOverlay = cargo2nix.overlays.default;
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ cargo2nixOverlay ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [ bpf-linker llvm_19 clang_19 lld_19 pkg-config ];
          shellHook = ''
            echo "eBPF dev environment ready (bpf-linker available)"
          '';
        };
      });
}