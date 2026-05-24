{
  description = "C++ Protobuf development environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, utils }:
    utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          # Packages needed for building and running
          buildInputs = with pkgs; [
            protobuf
            pkg-config
          ];

          # Packages needed only at build-time (like the compiler)
          nativeBuildInputs = with pkgs; [
            gcc
            cmake
            protobuf # Includes the 'protoc' binary
          ];

          # Automatically set CFLAGS/CXXFLAGS using pkg-config
          shellHook = ''
            export CXXFLAGS="$(pkg-config --cflags protobuf)"
            export LDFLAGS="$(pkg-config --libs protobuf)"
            echo "Protobuf environment loaded. protoc version: $(protoc --version)"
          '';
        };
      }
    );
}