# Plan: Applying Cargo-Vendormod Tools to ~/dasl/

## Executive Summary
The ~/dasl repository is primarily a documentation/organization hub for the DASL/IPLD-Core project, containing:
- Numerous `.org` files (planning, fuzzing, coverage, infrastructure docs)
- Various data directories (fuzztest, fuzzing, deep_scanner_data, perf-tools, case-studies, etc.)
- Scripts and analysis tools
- **NO Rust source files, Cargo.toml, or Cargo.lock files**

Since cargo-vendormod tools are designed to analyze Rust projects (processing git submodules and Cargo.toml files), direct application will yield limited results. However, we can still extract value by:
1. Using workload_processor to confirm absence of Rust projects
2. Applying conceptual frameworks to analyze documentation structure
3. Leveraging auxiliary tools that don't require Rust projects

## Phase 1: Initial Assessment & Baseline Measurement

### 1.1 Workload Processor Analysis (Primary Tool)
```bash
cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod
nix develop -c cargo run --bin workload_processor -- ~/dasl
```

**Expected Output:**
- Total repositories: 0 (no git submodules with .git directories found)
- Total Cargo.toml files: 0
- Processing time: minimal
- JSON output confirming no Rust projects detected

**Purpose:** Establish baseline confirming ~/dasl contains no Rust projects for analysis.

### 1.2 File Structure Inventory
```bash
# Count files by type to understand documentation composition
find ~/dasl -type f | grep -E "\.org$" | wc -l      # Planning/documentation files
find ~/dasl -type f | grep -E "\.(sh|py)$" | wc -l  # Scripts
find ~/dasl -type f | grep -E "\.(txt|md)$" | wc -l # Text/markdown
find ~/dasl -type d | grep -vE "^\.$" | wc -l       # Directories
```

## Phase 2: Conceptual Application of Tool Principles

Even without Rust projects, we can apply the *principles* behind the cargo-vendormod tools to analyze ~/dasl's documentation structure:

### 2.1 Mathematical Atlas Principles Applied to Documentation
Instead of analyzing GitHub metrics, we can analyze:
- **Documentation "stars"** = Reference frequency/citations between .org files
- **Documentation "forks"** = Branches/variants of planning documents
- **Documentation "contributors"** = Edit frequency/authorship patterns
- **Documentation "language"** = File type classification (.org, .sh, .py, etc.)
- **Documentation "size"** = File size/complexity metrics
- **Documentation "last commit"** = Recency of updates

### 2.2 Coverage Analysis Principles Applied to Documentation
Instead of source/test file ratios, we can analyze:
- **Planning vs. Implementation ratio** = .org files in root vs. implementation directories
- **Documentation completeness** = Ratio of planned topics (.org files) to implemented features
- **Cross-reference density** = Internal links between documentation files
- **Metadata coverage** = Presence of frontmatter/tags in .org files

### 2.3 Performance Analysis Principles Applied to Documentation
Instead of code metrics, we can analyze:
- **Documentation density** = Words per file by category
- **Update frequency** = Git history analysis of documentation changes
- **Access patterns** = Which files are referenced most (if we had access logs)
- **Structural complexity** = Directory nesting depth and organization

## Phase 3: Tool Adaptation & Extension

### 3.1 Creating Documentation-Specific Analyzers
We could adapt the existing tools to work with documentation:

**Simple Documentation Atlas:**
```rust
// Conceptual adaptation of simple_repository_mathematical_atlas.rs
struct DocumentationFile {
    name: String,
    path: String,
    word_count: usize,
    reference_count: usize, // Internal links to other .org files
    last_updated_days: usize,
    category: String, // fuzzing, testing, infra, etc.
    complexity_score: usize, // Based on headings, lists, code blocks
}

struct DocumentationView {
    name: String,
    description: String,
    files: Vec<DocumentationFile>,
}
```

**Documentation Coverage Tile:**
Instead of source/test ratios, analyze:
- Planning documentation vs. implementation evidence ratio
- Cross-referencing completeness
- Metadata tag coverage

### 3.2 Existing Tool Applications
Some existing tools may still provide value:

**workload_report.rs** - Could generate reports on documentation statistics
**cli_tile_renderer.rs** - Could render documentation metrics in tile format
**goal_tracker.rs** - Could track documentation completion goals
**dasl_metadata_processor.rs** - Specifically designed for DASL metadata (promising!)

## Phase 4: Recommended Workflow

### 4.1 Immediate Actions (Next 15 minutes)
1. Run workload_processor to establish baseline (5 min)
2. Create file inventory of ~/dasl by type (5 min)  
3. Run any applicable existing tools (5 min)

### 4.2 Short-Term Actions (Next 2 hours)
1. Analyze .org file structure and categorization
2. Extract metrics from documentation (word counts, heading levels, etc.)
3. Create conceptual mappings of documentation relationships
4. Generate initial report using adapted principles

### 4.3 Medium-Term Actions (Next 1 day)
1. Develop documentation-specific analysis scripts
2. Create visualizations of documentation structure
3. Identify gaps and recommendations for documentation improvement
4. Package findings for DASL team consumption

## Phase 5: Success Metrics & Deliverables

### 5.1 Quantitative Metrics
- Total documentation files analyzed
- Categories identified and file distribution
- Average file size and complexity by category
- Cross-reference density between files
- Documentation age/distribution (recent vs. legacy)

### 5.2 Qualitative Insights
- Documentation organization effectiveness
- Gaps in coverage or outdated content
- Recommendations for restructuring or consolidation
- Opportunities for better cross-referencing

### 5.3 Deliverables
1. **Baseline Report** - Output from workload_processor confirming no Rust projects
2. **Documentation Inventory** - Complete file listing with categorization
3. **Structure Analysis** - Metrics and insights on documentation organization
4. **Recommendation Report** - Specific actionable improvements for DASL documentation
5. **Adapted Tools** - Any scripts or modifications created for documentation analysis

## Risk Assessment & Mitigation

### Risks:
1. **Low yield from Rust-focused tools** - Since no Rust projects exist
2. **Misapplication of concepts** - Forcing Rust-analogies where they don't fit
3. **Missing context** - Not understanding the purpose of various documentation files

### Mitigations:
1. **Set appropriate expectations** - Focus on what can be learned, not what Rust tools were designed for
2. **Use conceptual adaptation** - Apply principles rather than direct tool usage
3. **Supplement with domain knowledge** - Review sample .org files to understand purpose

## Immediate Next Steps

```bash
# 1. Establish baseline with workload processor
cd /mnt/data1/nix/vendor/rust/cargo2nix/submodules/cargo-clean/tools/cargo-vendormod
nix develop -c cargo run --bin workload_processor -- ~/dasl

# 2. Create documentation inventory
find ~/dasl -type f -name "*.org" | wc -l
find ~/dasl -type f -name "*.sh" | wc -l
find ~/dasl -type f -name "*.py" | wc -l
ls -la ~/dasl/ | grep -E "^d" | wc -l  # directories

# 3. Sample examination of key documentation
head -20 ~/dasl/01-overview.org
head -20 ~/dasl/02-fuzzing-plan.org
ls -la ~/dasl/fuzzing/
ls -la ~/dasl/fuzztest/ | head -5
```