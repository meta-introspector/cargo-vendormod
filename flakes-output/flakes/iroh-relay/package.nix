{ pkgs, lib, ... }:

pkgs.rustPlatform.buildRustPackage {
  pname = "iroh-relay";
  version = "0.96.1";
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
  # (workspace lock originally at: /home/mdupont/dasl/lang/iroh/Cargo.lock)
}