---
name: refactor-main-rs
description: Continues refactoring the `cargo-vendormod/src/main.rs` file by moving helper functions and structs to separate modules (`src/args.rs`, `src/utils.rs`) and resolving compilation errors. Designed for incremental, iterative code splitting. Use when a `pi` agent needs to continue refactoring `cargo-vendormod`.
---

# Refactor `cargo-vendormod/src/main.rs`

## Overview

This skill enables a `pi` agent to continue the incremental refactoring of the `cargo-vendormod` project, specifically focusing on splitting the large `src/main.rs` file. The agent will move functions, structs, and constants into appropriate new modules (`src/args.rs`, `src/utils.rs`) and ensure the project remains compilable throughout the process.

## How to Use This Skill

1.  **Environment Setup:** Ensure you are in a `nix develop` environment that provides the necessary Rust toolchain (`cargo`, `rustc`, `rust-analyzer`, `rustfmt`, `clippy`) and the `pi` agent itself. A suitable `flake.nix` is provided within the task's directory for this purpose.
2.  **Context:** The refactoring is intended to occur within a dedicated git worktree (e.g., `refactor-main`).
3.  **Invocation:** Execute the `runme.sh` script located in the skill's `scripts/` directory. This script will invoke the `pi` agent with a pre-configured prompt and load the necessary skills and context.

    ```bash
    ./scripts/runme.sh
    ```

## Task Instructions

The detailed instructions and current state of the refactoring task are documented in the `references/task_instructions.md` file. The `pi` agent should refer to this file for the step-by-step plan, including which functions and structs to move next, and the incremental verification steps.

## Resources

This skill bundles the following resources:

### scripts/
*   `runme.sh`: A shell script designed to invoke the `pi` agent with the specific prompt and environment configuration required to continue the refactoring task.

### references/
*   `task_instructions.md`: Contains the detailed, step-by-step plan for refactoring `cargo-vendormod/src/main.rs`, outlining what has been completed, what remains, and the order of operations. This serves as the primary guide for the `pi` agent.
