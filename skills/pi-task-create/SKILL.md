---
name: pi-task-create
description: Automates the creation of n0x-pi compatible task directories. Generates `flake.nix`, `runme.sh`, and `GEMINI.md` files based on user-provided details. Use when needing to quickly set up a new n0x-pi task for agent delegation.
---

# Pi Task Create Skill

## Overview

This skill automates the process of creating a new `n0x-pi` compatible task directory. A `n0x-pi` task typically consists of a directory containing a `flake.nix` for environment setup, a `runme.sh` script to launch the `pi` agent, and a `GEMINI.md` file that defines the task's instructions and context. This skill streamlines the setup of such tasks, ensuring consistency and adherence to `n0x-pi` conventions.

## How to Use This Skill

To create a new `n0x-pi` task, the agent will prompt you for the necessary details:

1.  **Task Name:** A short, descriptive name for the new task (e.g., `refactor-my-module`).
2.  **Task Objective (GEMINI.md Content):** The detailed instructions and context for the `pi` agent. This will form the content of the `GEMINI.md` file within the new task directory.
3.  **Target Directory:** The parent directory where the new task directory will be created.
4.  **Skills to Load:** A comma-separated list of `gemini` skills that the `pi` agent should load when executing this task.

Upon receiving these details, the skill will generate:

*   A new task directory (e.g., `<target-directory>/<task-name>`).
*   `<task-name>/flake.nix`: Configured with basic Rust development tools and `n0x-pi` integration.
*   `<task-name>/runme.sh`: An executable script to launch the `pi` agent with the provided task objective and skills.
*   `<task-name>/GEMINI.md`: Containing the detailed task objective.

## Resources

This skill bundles the following resources:

### scripts/
*   `create_task.cjs`: A Node.js script that handles the creation of the task directory and generation of `flake.nix`, `runme.sh`, and `GEMINI.md` based on user input.

### assets/
*   `flake.nix.template`: A template for the `flake.nix` file to be generated.
*   `runme.sh.template`: A template for the `runme.sh` script to be generated.
*   `GEMINI.md.template`: A template for the `GEMINI.md` file to be generated.
