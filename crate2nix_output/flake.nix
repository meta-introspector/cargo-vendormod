{
  description = "crate2nix-generated builds for all 27 projects";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay, ... }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" ];
      forAllSystems = f: nixpkgs.lib.genAttrs systems (system: f system);
      pkgsFor = system: import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };
    in {
      packages = forAllSystems (system:
        let
          pkgs = pkgsFor system;
          rustToolchain = pkgs.rust-bin.nightly.latest.default;
          buildRustCrateForPkgs = p: p.buildRustCrate.override {
            rustc = rustToolchain;
          };
        in {
          agave-solana-validator = (pkgs.callPackage ./agave-solana-validator/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          boa = (pkgs.callPackage ./boa/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          cargo2 = (pkgs.callPackage ./cargo2/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          cargo2nix = (pkgs.callPackage ./cargo2nix/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          crate2nix-zos = (pkgs.callPackage ./crate2nix-zos/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          fandom-multiverse-tinystories = (pkgs.callPackage ./fandom-multiverse-tinystories/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          ferron-zos = (pkgs.callPackage ./ferron-zos/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          forgecode = (pkgs.callPackage ./forgecode/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          fractran-vm = (pkgs.callPackage ./fractran-vm/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          gibb-eri-sh = (pkgs.callPackage ./gibb-eri-sh/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          mmgroup-rust = (pkgs.callPackage ./mmgroup-rust/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          monster-shadows-math = (pkgs.callPackage ./monster-shadows-math/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          nginx-generator = (pkgs.callPackage ./nginx-generator/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          osm-parquet-tiles = (pkgs.callPackage ./osm-parquet-tiles/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          osm-planet-torrent = (pkgs.callPackage ./osm-planet-torrent/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          pozzoli-ettore-solfeggi = (pkgs.callPackage ./pozzoli-ettore-solfeggi/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          rust = (pkgs.callPackage ./rust/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          solfunmeme-dioxus = (pkgs.callPackage ./solfunmeme-dioxus/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          solfunmeme-dioxus-apk = (pkgs.callPackage ./solfunmeme-dioxus-apk/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          solfunmeme-dioxus2 = (pkgs.callPackage ./solfunmeme-dioxus2/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
          voa-borcherds-archive = (pkgs.callPackage ./voa-borcherds-archive/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).rootCrate.build;
          zos-plugins = (pkgs.callPackage ./zos-plugins/Cargo-generated.nix { inherit pkgs buildRustCrateForPkgs; }).allWorkspaceMembers;
        }
      );
    };
}
