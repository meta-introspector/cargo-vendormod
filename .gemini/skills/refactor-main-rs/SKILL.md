---
name: refactor-main-rs
description: Provides utilities for incremental refactoring of large Rust files by moving functions and structs into separate modules and resolving compilation errors. Use when a `pi` agent needs to perform iterative code splitting and restructuring in a Rust project.
---

# Rust Incremental Refactoring Skill

## Overview

This skill enables a `pi` agent to perform incremental refactoring of Rust source files. It is designed to assist in breaking down large, monolithic files by moving functions, structs, and constants into appropriate new modules, while ensuring the project remains compilable throughout the process. The agent will follow a provided task definition to guide its refactoring efforts.

## How to Use This Skill

This skill is typically invoked via a task-specific `runme.sh` script located in a `n0x-pi` task directory. This `runme.sh` script will:

1.  **Set up the environment:** Using `nix develop`, providing the necessary Rust toolchain (`cargo`, `rustc`, `rust-analyzer`, `rustfmt`, `clippy`) and the `pi` agent itself.
2.  **Provide Context:** The refactoring is typically intended to occur within a dedicated git worktree.
3.  **Formulate a Prompt:** The `runme.sh` script will construct a detailed prompt for the `pi` agent, usually by reading a `GEMINI.md` file located within the task directory. This `GEMINI.md` file will contain the specific refactoring instructions, current state, and objectives for the agent.

## Resources

This skill bundles the following resources:

### scripts/
*   `runme.sh`: A shell script template that can be used within a `n0x-pi` task directory to invoke the `pi` agent with a specific prompt and environment configuration, leveraging this skill.