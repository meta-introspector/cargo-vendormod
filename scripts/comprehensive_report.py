#!/usr/bin/env python3
"""
Generate a comprehensive analysis report for the cargo-vendormod dependency graph.
"""

import json
import sys
from collections import Counter, defaultdict
from pathlib import Path

def analyze(graph_path: str, output_path: str) -> None:
    with open(graph_path) as f:
        graph = json.load(f)
    
    nodes = graph['nodes']
    edges = graph['edges']
    metrics = graph['metrics']
    
    # Build in/out degree maps
    in_degree = Counter()
    out_degree = Counter()
    node_names = {}
    node_sources = {}
    node_versions = {}
    
    for node in nodes:
        node_names[node['id']] = node['crate_name']
        node_sources[node['id']] = node['source']
        node_versions[node['id']] = node['version']
    
    for edge in edges:
        out_degree[edge['from']] += 1
        in_degree[edge['to']] += 1
    
    # Find most depended-upon crates (highest in-degree)
    most_depended = sorted(in_degree.items(), key=lambda x: -x[1])[:20]
    
    # Find crates with most dependencies (highest out-degree)
    most_deps = sorted(out_degree.items(), key=lambda x: -x[1])[:20]
    
    # Categorize dependencies
    # Sources: crates.io, git repos, workspace
    source_counter = Counter()
    for node in nodes:
        src = node['source']
        if node['is_workspace_member']:
            source_counter['workspace'] += 1
        elif src and src.startswith('http'):
            source_counter['git'] += 1
        elif src == 'crates.io':
            source_counter['crates.io'] += 1
        else:
            source_counter[src or 'unknown'] += 1
    
    # Version analysis
    version_counter = Counter()
    for node in nodes:
        if not node['is_workspace_member']:
            v = node['version']
            # Major version grouping
            major = v.split('.')[0] if '.' in v else v
            version_counter[major] += 1
    
    # Dependency chain analysis
    # For the workspace member, find direct deps
    workspace_nodes = [n for n in nodes if n['is_workspace_member']]
    direct_deps = []
    for edge in edges:
        if edge['from'].startswith('workspace:'):
            direct_deps.append(edge['to'])
    
    direct_dep_details = []
    for dep_id in direct_deps:
        if dep_id in node_names:
            direct_dep_details.append({
                'name': node_names[dep_id],
                'version': node_versions.get(dep_id, '?'),
                'source': node_sources.get(dep_id, '?'),
                'in_degree': in_degree.get(dep_id, 0),
                'out_degree': out_degree.get(dep_id, 0)
            })
    
    direct_dep_details.sort(key=lambda x: -x['in_degree'])
    
    # Build the report
    report = []
    report.append("=" * 72)
    report.append("  CARGO-VENDORMOD SELF-DEPENDENCY GRAPH ANALYSIS REPORT")
    report.append("=" * 72)
    report.append("")
    
    # Summary
    report.append(f"Analysis of: {graph.get('workspace_path', 'N/A')}")
    report.append(f"Generated: Self-analysis using cargo-vendormod graph binary")
    report.append("")
    
    # Section 1: Graph Overview
    report.append("─" * 72)
    report.append("SECTION 1: GRAPH OVERVIEW")
    report.append("─" * 72)
    report.append("")
    report.append(f"  Total nodes (packages)  : {metrics['node_count']}")
    report.append(f"  Total edges (deps)      : {metrics['edge_count']}")
    report.append(f"  Workspace members       : {metrics['workspace_members']}")
    report.append(f"  Direct dependencies     : {metrics['direct_dependencies']}")
    report.append(f"  Average degree          : {metrics['average_degree']:.2f}")
    report.append("")
    
    # Section 2: Source Distribution
    report.append("─" * 72)
    report.append("SECTION 2: SOURCE DISTRIBUTION")
    report.append("─" * 72)
    report.append("")
    report.append(f"  {'Source':<20} {'Count':>8} {'%':>8}")
    report.append(f"  {'─'*19} {'─'*8} {'─'*8}")
    total_sources = sum(source_counter.values())
    for src, count in sorted(source_counter.items(), key=lambda x: -x[1]):
        pct = count / total_sources * 100
        report.append(f"  {src:<20} {count:>8} {pct:>7.1f}%")
    report.append("")
    
    # Section 3: Top 20 Most Depended-Upon Crates
    report.append("─" * 72)
    report.append("SECTION 3: TOP 20 MOST DEPENDED-UPON CRATES")
    report.append("─" * 72)
    report.append("")
    report.append(f"  {'Rank':<4} {'Crate':<30} {'Version':<12} {'Depended By':>12}")
    report.append(f"  {'─'*4} {'─'*29} {'─'*11} {'─'*12}")
    for i, (node_id, count) in enumerate(most_depended[:20], 1):
        name = node_names.get(node_id, node_id)
        ver = node_versions.get(node_id, '?')
        # Shorten if needed
        if len(name) > 28:
            name = name[:25] + '...'
        report.append(f"  {i:<4} {name:<30} {ver:<12} {count:>12}")
    report.append("")
    
    # Section 4: Top 20 Crates with Most Dependencies
    report.append("─" * 72)
    report.append("SECTION 4: TOP 20 CRATES WITH MOST DEPENDENCIES")
    report.append("─" * 72)
    report.append("")
    report.append(f"  {'Rank':<4} {'Crate':<30} {'Version':<12} {'Deps':>8}")
    report.append(f"  {'─'*4} {'─'*29} {'─'*11} {'─'*8}")
    for i, (node_id, count) in enumerate(most_deps[:20], 1):
        name = node_names.get(node_id, node_id)
        ver = node_versions.get(node_id, '?')
        if len(name) > 28:
            name = name[:25] + '...'
        report.append(f"  {i:<4} {name:<30} {ver:<12} {count:>8}")
    report.append("")
    
    # Section 5: Direct Dependencies of cargo-vendormod
    report.append("─" * 72)
    report.append("SECTION 5: DIRECT DEPENDENCIES OF cargo-vendormod")
    report.append("─" * 72)
    report.append("")
    report.append(f"  Found {len(direct_dep_details)} direct dependencies")
    report.append("")
    
    # Group by version
    dep_by_version = defaultdict(list)
    for dep in direct_dep_details:
        major = dep['version'].split('.')[0] if '.' in dep['version'] else dep['version']
        dep_by_version[major].append(dep)
    
    for major_ver in sorted(dep_by_version.keys(), key=int):
        deps = dep_by_version[major_ver]
        report.append(f"  v{major_ver}.x ({len(deps)} crates):")
        for dep in deps:
            report.append(f"    - {dep['name']:35s} v{dep['version']:12s}")
        report.append("")
    
    # Section 6: Version Distribution by Major
    report.append("─" * 72)
    report.append("SECTION 6: DEPENDENCY VERSION DISTRIBUTION (by major version)")
    report.append("─" * 72)
    report.append("")
    total_ver = sum(version_counter.values())
    report.append(f"  {'Major v':<10} {'Count':>8} {'%':>8}")
    report.append(f"  {'─'*9} {'─'*8} {'─'*8}")
    for ver, count in sorted(version_counter.items(), key=lambda x: int(x[0]) if x[0].isdigit() else 9999):
        pct = count / total_ver * 100
        report.append(f"  v{ver:<9} {count:>8} {pct:>7.1f}%")
    report.append("")
    
    # Section 7: Git Dependencies
    git_deps = [n for n in nodes if n['source'] and 'http' in n['source']]
    if git_deps:
        report.append("─" * 72)
        report.append("SECTION 7: GIT DEPENDENCIES")
        report.append("─" * 72)
        report.append("")
        report.append(f"  Found {len(git_deps)} git-sourced dependencies")
        for dep in git_deps:
            report.append(f"    - {dep['crate_name']:35s} v{dep['version']:12s}")
            report.append(f"      Source: {dep['source']}")
        report.append("")
    
    # Section 8: Graph Structure Metrics
    report.append("─" * 72)
    report.append("SECTION 8: GRAPH STRUCTURE METRICS")
    report.append("─" * 72)
    report.append("")
    
    # Calculate density
    n = metrics['node_count']
    e = metrics['edge_count']
    max_possible_edges = n * (n - 1)
    density = e / max_possible_edges if max_possible_edges > 0 else 0
    
    report.append(f"  Graph density          : {density:.6f} ({density*100:.4f}%)")
    report.append(f"  Avg in-degree          : {e/n:.2f}")
    report.append(f"  Avg out-degree         : {e/n:.2f}")
    report.append(f"  Max in-degree          : {max(in_degree.values()) if in_degree else 0}")
    report.append(f"  Max out-degree         : {max(out_degree.values()) if out_degree else 0}")
    report.append(f"  SCC count              : {metrics['strongly_connected_components']}")
    report.append(f"  Dev dependencies       : {metrics['dev_dependencies']}")
    report.append(f"  Build dependencies     : {metrics['build_dependencies']}")
    report.append("")
    
    # Conclusion
    report.append("═" * 72)
    report.append("CONCLUSION")
    report.append("═" * 72)
    report.append("")
    report.append(f"  cargo-vendormod v0.2.0 has {len(direct_dep_details)} direct dependencies")
    report.append(f"  spanning {total_sources - 1} unique packages across its dependency tree.")
    report.append(f"  The graph has {n} total nodes and {e} edges with a density of {density:.6f}.")
    report.append("")
    report.append("  Top 5 most depended-upon crates:")
    for i, (node_id, count) in enumerate(most_depended[:5], 1):
        name = node_names.get(node_id, node_id)
        report.append(f"    {i}. {name} (used by {count} crates)")
    report.append("")
    report.append("  The dependency graph has been saved as:")
    report.append(f"    - analysis/self_graph.json (machine-readable)")
    report.append(f"    - analysis/self_graph.dot  (Graphviz format)")
    report.append(f"    - analysis/self_graph.svg  (rendered SVG)")
    report.append(f"    - analysis/analysis.json   (metrics summary)")
    report.append(f"    - analysis/partitions/      (8-way partitioning)")
    report.append("")
    
    report_content = '\n'.join(report)
    
    with open(output_path, 'w') as f:
        f.write(report_content)
    
    print(report_content)

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: comprehensive_report.py <graph.json> [output.md]")
        sys.exit(1)
    
    graph_path = sys.argv[1]
    output_path = sys.argv[2] if len(sys.argv) > 2 else 'self_report.md'
    analyze(graph_path, output_path)
