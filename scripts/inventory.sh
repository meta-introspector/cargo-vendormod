#!/usr/bin/env bash
# Fast flat inventory — no recursion, just check top-level markers
set -euo pipefail

BASE="${1:-/home/mdupont/projects}"
INVENTORY="analysis/inventory.json"
mkdir -p analysis

echo '[' > "$INVENTORY"
FIRST=true

for repo in "$BASE"/*/; do
  [ -d "$repo" ] || continue
  name="$(basename "$repo")"
  
  # Fast file existence checks — all stat calls, no subshells
  local is_git=0; [ -d "$repo/.git" ] && is_git=1
  local has_cargo=0; [ -f "$repo/Cargo.toml" ] && has_cargo=1
  local has_pyproject=0; [ -f "$repo/pyproject.toml" ] && has_pyproject=1
  local has_setup=0; [ -f "$repo/setup.py" ] && has_setup=1
  local has_makefile=0; [ -f "$repo/Makefile" ] && has_makefile=1
  local has_flake=0; [ -f "$repo/flake.nix" ] && has_flake=1
  local has_package=0; [ -f "$repo/package.json" ] && has_package=1
  local has_go=0; [ -f "$repo/go.mod" ] && has_go=1
  local has_cabal=0
  for f in "$repo"/*.cabal; do [ -f "$f" ] && has_cabal=1 && break; done
  
  # Count submodules from .gitmodules
  local sub_count=0
  if [ -f "$repo/.gitmodules" ]; then
    sub_count=$(grep -c '^\s*path\s*=' "$repo/.gitmodules" 2>/dev/null || echo 0)
  fi
  
  # Write output — use printf to avoid heredoc quoting issues
  [ "$FIRST" = "true" ] && FIRST=false || printf ',\n' >> "$INVENTORY"
  printf '{"path":"%s","name":"%s","depth":0,"is_git":%s,"has_cargo":%s,"has_pyproject":%s,"has_setup_py":%s,"has_makefile":%s,"has_flake":%s,"has_package_json":%s,"has_cabal":%s,"has_go_mod":%s,"submodule_count":%s}\n' \
    "$repo" "$name" \
    "$is_git" "$has_cargo" "$has_pyproject" "$has_setup" \
    "$has_makefile" "$has_flake" "$has_package" "$has_cabal" "$has_go" \
    "$sub_count" >> "$INVENTORY"
done

echo ']' >> "$INVENTORY"

# Stats
echo "=== Inventory Complete ==="
echo "File: $INVENTORY ($(wc -c < "$INVENTORY") bytes)"
python3 -c "
import json
with open('$INVENTORY') as f:
    data = json.load(f)
print(f'Entries: {len(data)}')
types = {}
for e in data:
    for k in ['cargo','pyproject','setup_py','makefile','flake','package_json','cabal','go_mod']:
        if e.get('has_'+k.replace('-','_')): types[k]=types.get(k,0)+1
    if e.get('is_git'): types['git']=types.get('git',0)+1
for k,v in sorted(types.items()): print(f'  {k}: {v}')
" 2>/dev/null
