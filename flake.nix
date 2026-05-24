{
  description = "Cargo Vendormod - Solana crate processing with Nix dependencies";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = [
            # Rust toolchain
            pkgs.rustc
            pkgs.cargo
            pkgs.rustfmt
            pkgs.clippy

            # System libraries needed for OpenSSL, libgit2, and other build dependencies
            pkgs.openssl.dev
            pkgs.libgit2
            pkgs.curl
            pkgs.libssh2
            pkgs.zlib
            pkgs.nghttp2
            pkgs.pkg-config
            pkgs.gcc
            pkgs.glibc.dev
            
            # C++ development headers for protobuf
            pkgs.stdenv  # Current standard environment (includes C++ headers)
            pkgs.gcc-unwrapped
            
            # Protobuf dependencies
            pkgs.protobuf
            pkgs.protobufc
            pkgs.automake
            pkgs.autoconf
            pkgs.libtool
            pkgs.m4
            pkgs.systemd.dev
            pkgs.libusb1
            pkgs.hidapi
            pkgs.ncurses
            pkgs.util-linux
            pkgs.snappy
            pkgs.lz4
            pkgs.zstd
            pkgs.bzip2
            pkgs.liburing
            pkgs.llvmPackages_21.llvm
            pkgs.llvmPackages_21.libclang
          ];

          shellHook = ''
            export CFG_RELEASE="1.70.0";
            export CFLAGS="-O2 -g";
            export CPATH="${pkgs.glibc.dev}/include:${pkgs.gcc}/include${CPATH:+:}$CPATH";
            export CXXFLAGS="-O2 -g -isystem ${pkgs.glibc.dev}/include";
            export LIBCLANG_FLAGS="--sysroot=${pkgs.glibc.dev}";
            export LIBCLANG_PATH="${pkgs.llvmPackages_21.libclang.lib}/lib";
            export LLVM_CONFIG="${pkgs.llvmPackages_21.llvm.dev}/bin/llvm-config";
            export LLVM_CONFIG_PATH="${pkgs.llvmPackages_21.llvm}/lib";
            export PATH="${pkgs.rustc}/bin:${pkgs.cargo}/bin${PATH:+:}$PATH";
            export PKG_CONFIG_PATH="${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.systemd.dev}/lib/pkgconfig:${pkgs.libusb1}/lib/pkgconfig";
            export REAL_LIBRARY_PATH="$LD_LIBRARY_PATH";
            export REAL_LIBRARY_PATH_VAR="LD_LIBRARY_PATH";
            export RUSTC_BOOTSTRAP=1;
            export NIX_GLIBC_DEV="${pkgs.glibc.dev}";
            export NIX_GCC_PATH="${pkgs.gcc}";
            export NIX_GCC_REAL_PATH="${pkgs.gcc.cc}";

            export BINDGEN_EXTRA_CLANG_ARGS="$(
              cat ${pkgs.stdenv.cc}/nix-support/libc-crt1-cflags \
                  ${pkgs.stdenv.cc}/nix-support/libc-cflags \
                  ${pkgs.stdenv.cc}/nix-support/cc-cflags
            ) ${pkgs.lib.optionalString pkgs.stdenv.cc.isClang "-idirafter ${pkgs.stdenv.cc.cc.lib}/lib/clang/${pkgs.lib.getVersion pkgs.stdenv.cc.cc}/include"}";

            echo "Nix development shell with Rust and all dependencies ready.";
          '';
        };
      }
    );
}