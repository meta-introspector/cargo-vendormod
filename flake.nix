{
  description = "Development shell for refactoring cargo-vendormod";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    n0x-pi.url = "git+file:///mnt/data1/git/github.com/sub0xdai/n0x-pi.git?ref=master";
    ipld-car-ipc-shmem-linux = {
      url = "git+file:///mnt/data1/time-2026/02-february/22/dasl/ipld-car-ipc-shmem-linux";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    self-repo = {
      url = "git+file:///mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod";
      flake = false;
    };
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, n0x-pi, ipld-car-ipc-shmem-linux, self-repo, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];
      perSystem = { config, pkgs, system, ... }:
        let
          devShellBuildInputs = with pkgs; [
            cargo
            rustc
            rust-analyzer
            rustfmt
            clippy
            pkg-config
            openssl
            n0x-pi.packages.${system}.pi
            ipld-car-ipc-shmem-linux.packages.${system}.letta-ipld-memory
            # Add any other specific tools needed for refactoring
          ];
        in {
          devShells.default = pkgs.mkShell {
            buildInputs = devShellBuildInputs;
            # Environment variables specific to this project, if any
            # For example, if `self-repo` points to the `cargo-vendormod` itself
            # then we might want to set CARGO_MANIFEST_DIR to self-repo.
            # However, for a devShell, it's usually sufficient to just have the tools.
          };
        };
    };
}
