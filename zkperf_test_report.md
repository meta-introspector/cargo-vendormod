# Repository Mathematical Atlas - ZKperf Coverage & Performance Test Report

Generated on: 1779374010

## ZKperf Test Summary

- **Total ZKperf Tests**: 5
- **Passed**: 1
- **Failed**: 4
- **Success Rate**: 20.0%

## Performance Analysis

- **Total Test Time**: 34136ms
- **Average Test Time**: 6827.2ms

## Detailed ZKperf Test Results

### Basic Coverage Recording
- **Status**: ❌ FAIL
- **Duration**: 16623ms
- **Error**: Atlas execution failed: [1m[92m   Compiling[0m openssl-sys v0.9.116
[1m[92m   Compiling[0m regex v1.12.3
[1m[33mwarning[0m: openssl-sys@0.9.116: In file included from /usr/include/openssl/opensslv.h:109,
[1m[33mwarning[0m: openssl-sys@0.9.116:                  from build/expando.c:1:
[1m[33mwarning[0m: openssl-sys@0.9.116: /usr/include/openssl/macros.h:14:10: fatal error: openssl/opensslconf.h: No such file or directory
[1m[33mwarning[0m: openssl-sys@0.9.116:    14 | #include <openssl/opensslconf.h>
[1m[33mwarning[0m: openssl-sys@0.9.116:       |          ^~~~~~~~~~~~~~~~~~~~~~~
[1m[33mwarning[0m: openssl-sys@0.9.116: compilation terminated.
[1m[91merror[0m: failed to run custom build command for `openssl-sys v0.9.116`

Caused by:
  process didn't exit successfully: `/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/target/debug/build/openssl-sys-ab23dd1ac583ac6f/build-script-main` (exit status: 101)
  --- stdout
  cargo:rustc-check-cfg=cfg(osslconf, values("OPENSSL_NO_OCB", "OPENSSL_NO_SM4", "OPENSSL_NO_SEED", "OPENSSL_NO_CHACHA", "OPENSSL_NO_CAST", "OPENSSL_NO_IDEA", "OPENSSL_NO_CAMELLIA", "OPENSSL_NO_RC4", "OPENSSL_NO_BF", "OPENSSL_NO_PSK", "OPENSSL_NO_DEPRECATED_3_0", "OPENSSL_NO_SCRYPT", "OPENSSL_NO_SM3", "OPENSSL_NO_RMD160", "OPENSSL_NO_EC2M", "OPENSSL_NO_OCSP", "OPENSSL_NO_CMS", "OPENSSL_NO_COMP", "OPENSSL_NO_SOCK", "OPENSSL_NO_STDIO", "OPENSSL_NO_EC", "OPENSSL_NO_SSL3_METHOD", "OPENSSL_NO_KRB5", "OPENSSL_NO_TLSEXT", "OPENSSL_NO_SRP", "OPENSSL_NO_SRTP", "OPENSSL_NO_RFC3779", "OPENSSL_NO_SHA", "OPENSSL_NO_NEXTPROTONEG", "OPENSSL_NO_ENGINE", "OPENSSL_NO_BUF_FREELISTS", "OPENSSL_NO_RC2"))
  cargo:rustc-check-cfg=cfg(openssl)
  cargo:rustc-check-cfg=cfg(libressl)
  cargo:rustc-check-cfg=cfg(boringssl)
  cargo:rustc-check-cfg=cfg(awslc)
  cargo:rustc-check-cfg=cfg(awslc_pregenerated)
  cargo:rustc-check-cfg=cfg(libressl250)
  cargo:rustc-check-cfg=cfg(libressl251)
  cargo:rustc-check-cfg=cfg(libressl252)
  cargo:rustc-check-cfg=cfg(libressl261)
  cargo:rustc-check-cfg=cfg(libressl270)
  cargo:rustc-check-cfg=cfg(libressl271)
  cargo:rustc-check-cfg=cfg(libressl273)
  cargo:rustc-check-cfg=cfg(libressl280)
  cargo:rustc-check-cfg=cfg(libressl281)
  cargo:rustc-check-cfg=cfg(libressl291)
  cargo:rustc-check-cfg=cfg(libressl310)
  cargo:rustc-check-cfg=cfg(libressl321)
  cargo:rustc-check-cfg=cfg(libressl332)
  cargo:rustc-check-cfg=cfg(libressl340)
  cargo:rustc-check-cfg=cfg(libressl350)
  cargo:rustc-check-cfg=cfg(libressl360)
  cargo:rustc-check-cfg=cfg(libressl361)
  cargo:rustc-check-cfg=cfg(libressl370)
  cargo:rustc-check-cfg=cfg(libressl380)
  cargo:rustc-check-cfg=cfg(libressl381)
  cargo:rustc-check-cfg=cfg(libressl382)
  cargo:rustc-check-cfg=cfg(libressl390)
  cargo:rustc-check-cfg=cfg(libressl400)
  cargo:rustc-check-cfg=cfg(libressl410)
  cargo:rustc-check-cfg=cfg(libressl420)
  cargo:rustc-check-cfg=cfg(libressl430)
  cargo:rustc-check-cfg=cfg(ossl101)
  cargo:rustc-check-cfg=cfg(ossl102)
  cargo:rustc-check-cfg=cfg(ossl102f)
  cargo:rustc-check-cfg=cfg(ossl102h)
  cargo:rustc-check-cfg=cfg(ossl110)
  cargo:rustc-check-cfg=cfg(ossl110f)
  cargo:rustc-check-cfg=cfg(ossl110g)
  cargo:rustc-check-cfg=cfg(ossl110h)
  cargo:rustc-check-cfg=cfg(ossl111)
  cargo:rustc-check-cfg=cfg(ossl111b)
  cargo:rustc-check-cfg=cfg(ossl111c)
  cargo:rustc-check-cfg=cfg(ossl111d)
  cargo:rustc-check-cfg=cfg(ossl300)
  cargo:rustc-check-cfg=cfg(ossl310)
  cargo:rustc-check-cfg=cfg(ossl320)
  cargo:rustc-check-cfg=cfg(ossl330)
  cargo:rustc-check-cfg=cfg(ossl340)
  cargo:rustc-check-cfg=cfg(ossl400)
  cargo:rerun-if-env-changed=X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR
  X86_64_UNKNOWN_LINUX_GNU_OPENSSL_LIB_DIR unset
  cargo:rerun-if-env-changed=OPENSSL_LIB_DIR
  OPENSSL_LIB_DIR unset
  cargo:rerun-if-env-changed=X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR
  X86_64_UNKNOWN_LINUX_GNU_OPENSSL_INCLUDE_DIR unset
  cargo:rerun-if-env-changed=OPENSSL_INCLUDE_DIR
  OPENSSL_INCLUDE_DIR unset
  cargo:rerun-if-env-changed=X86_64_UNKNOWN_LINUX_GNU_OPENSSL_DIR
  X86_64_UNKNOWN_LINUX_GNU_OPENSSL_DIR unset
  cargo:rerun-if-env-changed=OPENSSL_DIR
  OPENSSL_DIR unset
  cargo:rerun-if-env-changed=OPENSSL_NO_PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG
  cargo:rerun-if-env-changed=OPENSSL_STATIC
  cargo:rerun-if-env-changed=OPENSSL_DYNAMIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_STATIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_DYNAMIC
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-env-changed=SYSROOT
  cargo:rerun-if-env-changed=OPENSSL_STATIC
  cargo:rerun-if-env-changed=OPENSSL_DYNAMIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_STATIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_DYNAMIC
  cargo:rustc-link-lib=ssl
  cargo:rustc-link-lib=crypto
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG
  cargo:rerun-if-env-changed=OPENSSL_STATIC
  cargo:rerun-if-env-changed=OPENSSL_DYNAMIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_STATIC
  cargo:rerun-if-env-changed=PKG_CONFIG_ALL_DYNAMIC
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-changed=build/expando.c
  cargo:rerun-if-env-changed=CC_x86_64-unknown-linux-gnu
  CC_x86_64-unknown-linux-gnu = None
  cargo:rerun-if-env-changed=CC_x86_64_unknown_linux_gnu
  CC_x86_64_unknown_linux_gnu = None
  cargo:rerun-if-env-changed=HOST_CC
  HOST_CC = None
  cargo:rerun-if-env-changed=CC
  CC = None
  cargo:rerun-if-env-changed=CC_ENABLE_DEBUG_OUTPUT
  cargo:rerun-if-env-changed=CRATE_CC_NO_DEFAULTS
  CRATE_CC_NO_DEFAULTS = None
  cargo:rerun-if-env-changed=CFLAGS
  CFLAGS = Some(-O2 -g)
  cargo:rerun-if-env-changed=CC_SHELL_ESCAPED_FLAGS
  CC_SHELL_ESCAPED_FLAGS = None
  cargo:rerun-if-env-changed=HOST_CFLAGS
  HOST_CFLAGS = None
  cargo:rerun-if-env-changed=CFLAGS_x86_64_unknown_linux_gnu
  CFLAGS_x86_64_unknown_linux_gnu = None
  cargo:rerun-if-env-changed=CFLAGS_x86_64-unknown-linux-gnu
  CFLAGS_x86_64-unknown-linux-gnu = None
  cargo:warning=In file included from /usr/include/openssl/opensslv.h:109,
  cargo:warning=                 from build/expando.c:1:
  cargo:warning=/usr/include/openssl/macros.h:14:10: fatal error: openssl/opensslconf.h: No such file or directory
  cargo:warning=   14 | #include <openssl/opensslconf.h>
  cargo:warning=      |          ^~~~~~~~~~~~~~~~~~~~~~~
  cargo:warning=compilation terminated.

  --- stderr

  thread 'main' (3659052) panicked at /home/mdupont/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/openssl-sys-0.9.116/build/main.rs:320:13:

  Header expansion error:
  Error { kind: ToolExecError, message: "command did not execute successfully (status code exit status: 1): LC_ALL=\"C\" \"cc\" \"-O0\" \"-ffunction-sections\" \"-fdata-sections\" \"-fPIC\" \"-g\" \"-gdwarf-4\" \"-fno-omit-frame-pointer\" \"-m64\" \"-I\" \"/usr/include\" \"-O2\" \"-g\" \"-E\" \"build/expando.c\"" }

  Failed to find OpenSSL development headers.

  You can try fixing this setting the `OPENSSL_DIR` environment variable
  pointing to your OpenSSL installation or installing OpenSSL headers package
  specific to your distribution:

      # On Ubuntu
      sudo apt-get install pkg-config libssl-dev
      # On Arch Linux
      sudo pacman -S pkgconf openssl
      # On Fedora
      sudo dnf install pkgconf perl-FindBin perl-IPC-Cmd openssl-devel
      # On Alpine Linux
      apk add pkgconf openssl-dev

  See rust-openssl documentation for more information:

      https://docs.rs/openssl

  note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
[1m[33mwarning[0m: build failed, waiting for other jobs to finish...


### Performance Recording
- **Status**: ❌ FAIL
- **Duration**: 6625ms
- **Error**: No performance data files found

### Coverage Analysis
- **Status**: ❌ FAIL
- **Duration**: 144ms
- **Error**: Analysis output missing expected data

### ZKperf Integration
- **Status**: ✅ PASS
- **Duration**: 4243ms
- **Coverage Data**: 2 scenarios successful

### Performance Benchmarking
- **Status**: ❌ FAIL
- **Duration**: 6501ms
- **Error**: Only 0 successful benchmark iterations

## ZKperf Integration Recommendations

- ⚠️ **Good Coverage Integration**: Basic coverage recording works
- ❌ **Poor Performance Integration**: Performance recording needs improvement
