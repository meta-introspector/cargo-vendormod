# Refactor `cargo-vendormod/src/main.rs` Task

This task is designed for a `pi` agent to continue the refactoring of the `cargo-vendormod` project. The primary goal is to break down the monolithic `src/main.rs` file into smaller, more manageable modules, specifically by moving helper functions, structs, and constants into `src/utils.rs`.

## Context

The refactoring is being performed in a dedicated git worktree named `refactor-main`, located at:
`/mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/worktrees/refactor-main`

A `flake.nix` file has been set up within this worktree (`<worktree-root>/flake.nix`) and within this task's directory (`./flake.nix`) to provide a consistent development environment for the `pi` agent.

## Current State of Refactoring

As of the creation of this task, the following steps have been completed:

*   **Created `refactor-main` worktree.**
*   **Moved Argument Parsing:** All `...Args` structs and the main `Commands` enum have been moved from `main.rs` to `src/args.rs`.
*   **Updated Module Declarations:** `lib.rs` now declares `pub mod args;` and `pub mod utils;`. It also uses `pub use args::MainArgs as Args;` to alias the main argument struct.
*   **Updated Main Binary `use` Statements:** `main.rs` has been updated to correctly import the argument structures from the library (`cargo_vendormod::Args` and `cargo_vendormod::args::{...}`).
*   **Updated Function Signature:** All `handle_...` functions in `main.rs` have had their `args` parameter types updated from `&MainArgs` to `&Args`.
*   **Moved `chrono_now`:** The `chrono_now` helper function has been successfully moved from `main.rs` to `src/utils.rs` and made public.
*   **Updated `flake.nix` for `n0x-pi`:** The `flake.nix` in the worktree root has been updated to follow `n0x-pi` conventions, including `n0x-pi` and `ipld-car-ipc-shmem-linux` as inputs, and their packages in the `devShellBuildInputs`.

## Objective for the `pi` Agent

The `pi` agent should continue the refactoring process by:

1.  **Moving helper functions and associated structs/constants from `src/main.rs` to `src/utils.rs` incrementally.**
    *   **Start with the `compute_cid` function.**
    *   For each item moved:
        *   Ensure the item (function, struct, constant) is `pub` in `src/utils.rs`.
        *   Update `src/main.rs` to import the moved item from `cargo_vendormod::utils::*` (or specific items if preferred).
        *   Address any new compilation errors that arise (e.g., missing imports, undefined types, etc.).
    *   Continue this process for:
        *   `shmem_put`
        *   `FileSample` (and its associated constants and functions like `sample_file`, `compute_entropy`, `compute_hecke_score`, `count_cids`)
        *   `IngestedSubmodule`
        *   `handle_memecache_upgrade`, `count_dependencies`, `handle_memecache_gc`, `parse_crate_version`, `dir_size`
        *   `ScannedFile`, `ScannedGitmodule`, `handle_scan_index`, `read_file_list`, `read_gitmodules_file`, `plocate_gitmodules`, and `read_parquet_index`.

2.  **Regularly run `cargo check`** (or `cargo build`) to monitor progress and identify compilation errors after each logical group of changes.

3.  **Ensure that `cargo check` passes cleanly** for the `cargo-vendormod` project within the `refactor-main` worktree after all the planned refactoring is complete.

## Environment Setup and Execution

The `pi` agent should use `nix develop` in the task's directory to set up the development environment, as defined by `flake.nix`. This environment includes all necessary Rust toolchains and `n0x-pi` utilities.

To start this task, the user (or another agent) should:

1.  Navigate to the task directory:
    ```bash
    cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod/worktrees/refactor-main/tasks/refactor-main-rs
    ```
2.  Enter the development shell (which will also load the `pi` agent environment):
    ```bash
    nix develop
    ```
3.  Execute the `runme.sh` script to invoke the `pi` agent with the specific task prompt:
    ```bash
    ./runme.sh
    ```
