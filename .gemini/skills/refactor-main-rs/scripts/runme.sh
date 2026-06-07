#!/usr/bin/env bash
# Run the pi agent to continue refactoring cargo-vendormod/src/main.rs
set -euo pipefail
cd "$(dirname "$0")"

# Source API key if available
[ -f ~/.deepseek/env.sh ] && source ~/.deepseek/env.sh

TASK_NAME="refactor-cargo-vendormod-main"
TASK_SKILLS="rust-async-patterns,nix-flakes,nix,gitnexus-refactoring"

echo "=== Bootstrap: ${TASK_NAME} ==="
echo "Skills: ${TASK_SKILLS}"
echo ""

# Path to the task definition markdown
TASK_FILE="../../tasks/refactor-cargo-vendormod-main.md"

# Prompt for the pi agent
TASK_INPUT="Task: ${TASK_NAME}
Skills to load: ${TASK_SKILLS}

Context: The goal is to refactor the monolithic 'src/main.rs' file by moving argument parsing structs and helper functions into separate modules. This is being done in a dedicated git worktree named 'refactor-main'. A 'flake.nix' has been set up in the worktree root to provide a consistent development environment.

Action: Continue the refactoring process as detailed in the task description in '${TASK_FILE}'. Your immediate goal is to incrementally move helper functions and structs from 'src/main.rs' to 'src/utils.rs' and resolve any compilation errors. Start by moving the 'compute_cid' function from 'src/main.rs' to 'src/utils.rs'. Ensure moved items are public and imports are updated. Run 'cargo check' after each logical change.
"

# Invoke the pi agent
nix develop . -c pi --offline --tools bash "${TASK_INPUT}"
