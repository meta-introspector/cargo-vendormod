{
  description = "eBPF D8 2A monitor for CBOR tag 42 detection";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          targets = [ "bpfel-unknown-none" ];
        };

        bpfLinker = pkgs.rust-bin.stable.latest.defaultRustLibOverlays.bpf-linker;

        nativeBuildInputs = with pkgs; [
          llvm_19
          clang_19
          lld_19
          pkg-config
          rustToolchain
        ];

        buildInputs = with pkgs; [ ];

      in
      {
        devShells.default = pkgs.mkShell {
          inherit nativeBuildInputs buildInputs;
          packages = [ bpfLinker ];
          shellHook = ''
            export PATH="${bpfLinker}/bin:$PATH"
            echo "eBPF dev environment ready"
          '';
        };

        packages.default = pkgs.stdenv.mkDerivation {
          pname = "d8-2a-ebpf";
          version = "0.1.0";
          src = ./.;
          nativeBuildInputs = nativeBuildInputs ++ [ bpfLinker ];
          buildPhase = ''
            export PATH="${bpfLinker}/bin:$PATH"
            RUSTFLAGS='--cfg bpf_target_arch="x86_64"' cargo build --target bpfel-unknown-none --release
          '';
          installPhase = ''
            mkdir -p $out/lib/bpf
            cp target/bpfel-unknown-none/release/*.so $out/lib/bpf/ 2>/dev/null || true
          '';
        };
      });
}