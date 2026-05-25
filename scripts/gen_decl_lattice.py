#!/usr/bin/env python3
"""Generate decl lattice: each declaration becomes a crate with Cargo.toml + flake.nix."""
import os, re, json
from collections import defaultdict

decls_dir = 'target/decls_all/src'
lattice_dir = 'decl_lattice'

os.makedirs(lattice_dir, exist_ok=True)

# Load decl graph
with open(f'{lattice_dir}/decl_graph.json') as f:
    graph = json.load(f)

decl_nodes = {n['id']: n for n in graph['nodes']}

# Collect extern crate imports per decl
crate_imports = defaultdict(set)
for name, info in [(n['id'], n) for n in graph['nodes']]:
    path = os.path.join(decls_dir, info['file'])
    with open(path) as fh:
        content = fh.read()
    for m in re.finditer(r'use\s+(\w+)::', content):
        ext = m.group(1)
        if ext not in ('crate', 'std', 'self', 'super'):
            crate_imports[name].add(ext)

# Build dependency index
deps_of = defaultdict(list)
for e in graph['edges']:
    deps_of[e['from']].append(e['to'])

# Generate crate dirs
os.makedirs(f'{lattice_dir}/crates', exist_ok=True)

for name, info in sorted(decl_nodes.items(), key=lambda x: len(deps_of.get(x[0], []))):
    simple = info['simple_name']
    safe_name = simple.lower()
    crate_dir = f'{lattice_dir}/crates/{safe_name}'
    src_dir = f'{crate_dir}/src'
    os.makedirs(src_dir, exist_ok=True)

    # Copy decl file as src/lib.rs
    src_path = os.path.join(decls_dir, info['file'])
    with open(src_path) as f:
        content = f.read()
    with open(f'{src_dir}/lib.rs', 'w') as f:
        f.write(content)

    # Generate Cargo.toml
    deps = deps_of.get(name, [])
    dep_lines = []
    for d in deps:
        d_simple = decl_nodes[d]['simple_name'].lower()
        dep_lines.append(f'{d_simple} = {{ path = "../{d_simple}" }}')
    for ext_crate in sorted(crate_imports.get(name, [])):
        dep_lines.append(f'{ext_crate} = "*"')

    nl = "\n"
    cargo_toml = f'''[package]
name = "{safe_name}"
version = "0.1.0"
edition = "2021"
description = "Decl lattice node: {simple}"

[dependencies]
{nl.join(dep_lines)}
'''
    with open(f'{crate_dir}/Cargo.toml', 'w') as f:
        f.write(cargo_toml)

    # Generate flake.nix
    flake_nix = f'''{{
  description = "{safe_name} decl lattice node";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  }};

  outputs = {{ self, nixpkgs, flake-utils, rust-overlay }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {{ inherit system overlays; }};
      in {{
        packages.default = pkgs.rustPlatform.buildRustPackage {{
          pname = "{safe_name}";
          version = "0.1.0";
          src = ./.;
        }};
      }});
}}
'''
    with open(f'{crate_dir}/flake.nix', 'w') as f:
        f.write(flake_nix)

print(f'Generated {len(decl_nodes)} crate directories')
print(f'Location: {lattice_dir}/crates/')

# Generate workspace Cargo.toml
sorted_infos = sorted(decl_nodes.values(), key=lambda x: x['simple_name'].lower())
members = '\n'.join(f'  "crates/{i["simple_name"].lower()}"' for i in sorted_infos)
workspace_cargo = f'''[workspace]
resolver = "2"
members = [
{members}
]
'''
with open(f'{lattice_dir}/Cargo.toml', 'w') as f:
    f.write(workspace_cargo)

# Generate top-level flake.nix
flake_entries = '\n'.join(
    f'      {i["simple_name"].lower()} = ./crates/{i["simple_name"].lower()};'
    for i in sorted_infos
)
top_flake = f'''{{
  description = "Decl lattice -- {len(decl_nodes)} nodes";

  inputs = {{
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  }};

  outputs = {{ self, nixpkgs, flake-utils, rust-overlay }}:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {{ inherit system; }};
      in {{
        packages = {{
{flake_entries}
        }};
      }});
}}
'''
with open(f'{lattice_dir}/flake.nix', 'w') as f:
    f.write(top_flake)

# Lattice metadata
lattice = {
    'nodes': len(decl_nodes),
    'edges': len(graph['edges']),
    'max_depth': max((len(deps_of.get(n, [])) for n in decl_nodes), default=0),
    'kernel_ops': {}
}
with open(f'{lattice_dir}/lattice.json', 'w') as f:
    json.dump(lattice, f, indent=2)

print(f'\nWorkspace: {lattice_dir}/Cargo.toml ({len(decl_nodes)} members)')
print(f'Top flake: {lattice_dir}/flake.nix')
print(f'Lattice: {lattice_dir}/lattice.json')
print(f'\nNext: cd {lattice_dir} && cargo check')
