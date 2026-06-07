{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "libipld-core";
  version = "0.4.3";
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