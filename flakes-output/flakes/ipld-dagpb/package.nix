{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "ipld-dagpb";
  version = "0.2.2";
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