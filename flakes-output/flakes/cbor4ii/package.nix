{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "cbor4ii";
  version = "1.2.2";
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