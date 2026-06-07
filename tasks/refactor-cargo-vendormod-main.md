# Task: Refactor `cargo-vendormod/src/main.rs`

## Current Status

We are in a git worktree named `refactor-main`, located at `/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/worktrees/refactor-main`.

The goal is to refactor the monolithic `src/main.rs` file by moving argument parsing structs and helper functions into separate modules.

So far, the following steps have been completed:

1.  **Created `refactor-main` worktree.**
2.  **Moved all `...Args` structs and the `Commands` enum from `main.rs` to `src/args.rs`.**
3.  **Updated `lib.rs` to declare `pub mod args;` and `pub use args::MainArgs as Args;`.**
4.  **Removed `mod args;` from `main.rs` (it should only be declared in `lib.rs`).**
5.  **Updated `main.rs` to use `cargo_vendormod::Args` and `cargo_vendormod::args::{Commands, VendoringCmd, ...}` for argument structs.**
6.  **Updated `handle_...` function signatures in `main.rs` to use `&Args` instead of `&MainArgs`.**
7.  **Created `src/utils.rs` and moved `chrono_now` function to it.**
8.  **Updated `lib.rs` to declare `pub mod utils;`.**
9.  **Created/Updated `flake.nix` in the worktree root to follow `n0x-pi` conventions.** It now includes `n0x-pi` and `ipld-car-ipc-shmem-linux` as inputs, and their packages in the `devShellBuildInputs`.

## Remaining Steps

The current compilation is failing with numerous errors, primarily related to:
*   Undefined types (structs and enums) that were moved to `src/args.rs` but are not yet imported or correctly referenced in `main.rs` and other files.
*   Undefined functions (e.g., `get_defined_workloads`, `compute_cid`, `shmem_put`, etc.) that were defined in `main.rs` and need to be moved to `src/utils.rs` (or other appropriate modules) and made public.
*   Structs like `FileFingerprint`, `Snapshot`, `Delta`, `ScannedFile`, `ScannedGitmodule`, `IngestedSubmodule` are also currently undefined in `main.rs` and need to be moved to `src/utils.rs` and made public.
*   Constants like `SHMEM_MAX_SIZE`, `IPLD_MEMORY_BIN`, `SAMPLE_HEAD_LINES`, etc. also need to be moved to `src/utils.rs`.

The detailed plan to address these is:

1.  **Continue moving helper functions and structs from `main.rs` to `src/utils.rs` one by one.** For each item:
    *   Move the definition to `src/utils.rs`.
    *   Make it `pub`.
    *   Add `use cargo_vendormod::utils::[item_name];` to `main.rs` (or `cargo_vendormod::utils::*` once all are moved).
    *   Run `cargo check` after each logical group of moves to identify and fix errors incrementally.
2.  **Address any remaining compilation errors** (e.g., `type annotations needed`, `no field '...'`).
3.  **Run `cargo check` until compilation succeeds.**

## Key Challenges / Learnings

*   The `replace` tool is very sensitive to exact string matching, making large, multi-line replacements difficult. Programmatic content modification (read-modify-write) might be more robust for larger refactors.
*   Rust's module system requires careful attention when separating a library crate (`lib.rs`) from a binary crate (`main.rs`) that shares code. A single file cannot be part of two `mod` declarations in different compilation units.
*   The `MainArgs` vs `Args` type alias and module path (`crate::args::Args` vs `crate::Args`) caused confusion in module resolution. This was resolved by using a public alias in `lib.rs` and referencing it through the crate name in `main.rs`.
*   Nix flake conventions for `n0x-pi` tasks involve defining `devShells` that include `n0x-pi` specific packages and potentially `apps.default` for task execution.

## Next Action for Agent

Continue with the refactoring plan outlined above, focusing on moving the helper functions and structs from `main.rs` to `src/utils.rs` incrementally. Start with `compute_cid`.
