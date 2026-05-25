---
name: split-decls
description: Split Rust source files into individual declaration files using the decl-splitter tool. Each declaration (function, struct, enum, trait, type, impl block, etc.) becomes its own .rs file with all imports preserved. Use when the user asks to extract declarations, split Rust source, decompose modules, or analyze code structure. Triggers on: "decl splitter", "split declarations", "decl-splitter", "extract decls", "decompose rust".
---

# Split Declarations

Split Rust `.rs` files into individual declaration files using decl-splitter.

## Prerequisites

- `decl-splitter` binary (build from forgecode decl-splitter-work worktree)

## Build decl-splitter

```bash
# The decl-splitter lives in forgecode's decl-splitter-work branch
cd /mnt/data1/time-2026/05-may/15/forgecode-decl-splitter/tools/decl_splitter
cargo build --bin impl-lattice --bin decl-patterns --bin decl-splitter --bin decl-lattice 2>&1
# Or from the worktree root:
cargo build --manifest-path /mnt/data1/time-2026/05-may/15/forgecode-decl-splitter/tools/decl_splitter/Cargo.toml
```

## Usage

```bash
# Single file
decl-splitter -i <input.rs> -o <output-dir>

# Batch: all .rs files in a project (parallel)
export DS=/mnt/data1/time-2026/05-may/15/forgecode-decl-splitter/tools/decl_splitter/target/debug/decl-splitter
find <project-dir> -name '*.rs' -not -path '*/vendor/*' -not -path '*/target/*' \
  | xargs -P4 -I{} $DS -i {} -o /tmp/decls/<project>/{}
```

## Output Structure

```
/tmp/decls/<project>/
└── <relative-path-to-rs-file>/
    ├── _decl_module_invocation.rs    # Module re-export file
    ├── <DeclarationName1>.rs         # Each declaration = separate file
    ├── <DeclarationName2>.rs
    └── ...
```

### Declaration File Format

Each declaration file is self-contained:
- All `use` imports preserved at the top
- One single declaration (function, struct, enum, trait, etc.)
- No cross-file references (those become edges in the decl graph)

### Module Invocation File

The `_decl_module_invocation.rs` re-imports all declarations back into scope:

```rust
pub use <DeclarationName1>;
pub use <DeclarationName2>;
```

## Running on Multiple Projects

```bash
# From the Makefile
make split-project N=<project-name>

# Manual batch
DS=... && find <src> -name '*.rs' -not -path '*/target/*' -not -path '*/vendor/*' | \
  xargs -P4 -I{} $DS -i {} -o /tmp/decls_<name>/{}
```

## Known Sizes

| Project | Source files | Declarations |
|---------|-------------|--------------|
| cargo-vendormod | 78 | 516 |
| fractran-vm | 10,759 | 946,846 |
| ragit | 1,341 | 3,735 |
| arti-tor-rs | 837 | 7,335 |
| streamofrandom | 167 | 678 |

## Next Step

After splitting declarations, use the `decl-lattice` skill to build the
dependency graph and generate individual crates for each declaration.
