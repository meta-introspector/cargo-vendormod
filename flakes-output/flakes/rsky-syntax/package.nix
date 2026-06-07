{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "rsky-syntax";
  version = "0.1.0";
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
  # (workspace lock originally at: /home/mdupont/dasl/data/rsky/Cargo.lock)
}