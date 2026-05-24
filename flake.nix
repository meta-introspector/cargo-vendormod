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

          # Build only the specified binary
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
      in
      {
        packages = {
          default = cargo-vendormod;

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

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustc cargo rustfmt clippy
            openssl.dev libgit2 libssh2 curl zlib nghttp2
            pkg-config gcc glibc.dev
          ];
        };
      }
    );
}
