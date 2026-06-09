#!/usr/bin/env bash
# Run the pi agent to build the vendormod module tree tile
set -euo pipefail
cd "$(dirname "$0")"

# Source API key if available
[ -f ~/.deepseek/env.sh ] && source ~/.deepseek/env.sh # shellcheck disable=SC1090

TASK_NAME="vendormod-module-tile"
TASK_SKILLS="rust-async-patterns,nix-flakes,nix,gitnexus-refactoring"

echo "=== Bootstrap: ${TASK_NAME} ==="
echo "Skills: ${TASK_SKILLS}"
echo ""

TASK_GEMINI_FILE="GEMINI.md"
GEMINI_CONTENT=$(cat "${TASK_GEMINI_FILE}")

TASK_INPUT="Task: ${TASK_NAME}
Skills to load: ${TASK_SKILLS}

${GEMINI_CONTENT}
"

# Invoke the pi agent
nix develop . -c pi --offline --tools bash "${TASK_INPUT}"
