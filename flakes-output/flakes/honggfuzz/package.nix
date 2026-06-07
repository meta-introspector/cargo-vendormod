{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "honggfuzz";
  version = "0.5.60";
  src = ./.;
  preBuild = ''
    mkdir -p .cargo
    cat > .cargo/config.toml << 'EOF'
[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
EOF
  '';
  cargoLock.lockFile = ./Cargo.lock;
}