{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "fedimint-connectors";
  version = "0.0.0";
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
  # (workspace lock originally at: /home/mdupont/dasl/data/fedimint/Cargo.lock)
}