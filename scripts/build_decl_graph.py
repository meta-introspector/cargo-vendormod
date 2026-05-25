#!/usr/bin/env python3
"""Build decl dependency graph from decl-splitter output."""
import os, re, json
from collections import defaultdict

decls_dir = 'target/decls_all/src'
lattice_dir = 'decl_lattice'
os.makedirs(lattice_dir, exist_ok=True)

decls = {}

# First pass: collect all decl names and their simple names (without prefix)
for root, dirs, files in os.walk(decls_dir):
    for f in files:
        if not f.endswith('.rs') or f == '_decl_module_invocation.rs':
            continue
        path = os.path.join(root, f)
        name = f.replace('.rs', '')
        rel = os.path.relpath(path, decls_dir)

        with open(path) as fh:
            content = fh.read()

        decls[name] = {
            'file': rel,
            'content': content,
            'size': len(content),
            'simple_name': name.split('_')[-1] if '_' in name else name
        }

# Build reverse index: simple name -> full decl names
simple_to_decl = defaultdict(list)
for name, info in decls.items():
    simple_to_decl[info['simple_name']].append(name)

# Second pass: extract references
edges = []
unmatched_refs = defaultdict(int)

for name, info in decls.items():
    content = info['content']
    refs_found = set()

    # Extract all use crate:: references
    for m in re.finditer(r'use\s+crate::(\w+)', content):
        refs_found.add(m.group(1))

    # Match simple names against known decls in body (after use statements)
    parts = content.split(';')
    body = ';'.join(parts[max(0, len(parts)-3):])  # last 3 stmts
    for sname, decl_names in simple_to_decl.items():
        if sname == info['simple_name']:
            continue
        count = len(re.findall(r'\b' + re.escape(sname) + r'\b', body))
        if count > 0:
            refs_found.add(sname)

    # Build edges
    for ref in refs_found:
        if ref in [d['simple_name'] for d in decls.values()]:
            targets = simple_to_decl.get(ref, [])
            for t in targets:
                if t != name:
                    edges.append({'from': name, 'to': t})
        else:
            unmatched_refs[ref] += 1

# Deduplicate edges
seen = set()
unique_edges = []
for e in edges:
    key = (e['from'], e['to'])
    if key not in seen:
        seen.add(key)
        unique_edges.append(e)

print(f'Declarations: {len(decls)}')
print(f'Dependency edges: {len(unique_edges)}')
print(f'Unmatched references: {len(unmatched_refs)}')

# Build graph JSON
graph = {
    'nodes': [{
        'id': name,
        'simple_name': info['simple_name'],
        'file': info['file'],
        'size': info['size']
    } for name, info in sorted(decls.items())],
    'edges': unique_edges
}

with open(f'{lattice_dir}/decl_graph.json', 'w') as f:
    json.dump(graph, f, indent=2)

print(f'Graph saved: {lattice_dir}/decl_graph.json')
print(f'  {len(graph["nodes"])} nodes, {len(graph["edges"])} edges')
