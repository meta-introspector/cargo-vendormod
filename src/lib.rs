//! # Cargo-Vendormod Library
//!
//! A library for managing cargo dependencies as git submodules with local mirrors.
//!
//! ## Modules
//!
//! - `config` - Configuration management
//! - `args` - CLI argument parsing
//! - `vendoring` - Submodule operations
//! - `global_dep_graph` - Dependency graph analysis
//! - `layer_processor` - Topological crate processing
//! - `workflow` - High-level workflow orchestration
//! - `git_wrapper` - Git operations
//! - `lockfile_parser` - Cargo.lock parsing
//! - `group_atlas` - Finite simple groups atlas
//! - `visualization` - Atlas visualization and rendering
//! - `pastbin_atlas` - Pastbin integration for atlas views

pub mod config;
pub mod args;
pub mod context;
pub mod vendoring;
pub mod global_dep_graph;
pub mod layer_processor;
pub mod workflow;
pub mod git_wrapper;
pub mod lockfile_parser;
pub mod repo_collection;
pub mod submodule_discovery;
pub mod workspace;
pub mod onboard;
pub mod patch;
pub mod report_generator;
pub mod report_processing;
pub mod process_all_crates;
pub mod workload;
pub mod actions;
pub mod rollup_lock;
pub mod zkperf_integration;
pub mod cargo_tool_discovery;
pub mod error_to_llm_pipeline;
pub mod fix_cargo_toml;
pub mod repo_sync_lib;
pub mod selinux_analyzer;
pub mod trace_compactor;
pub mod group_atlas;
pub mod visualization;
pub mod pastbin_atlas;
pub mod coverage;
pub mod benchmark;
pub mod workload_processor;
pub mod goal_tracker;
pub mod gitmodules_metadata;
pub mod dasl_metadata_processor;
pub mod vendoring_cmds;
pub mod nur_flake;
pub mod crate_flake;
pub mod flake_check;
pub mod lang_detect;
pub mod multi_lang_flake;
pub mod dasl_pipeline;
pub mod lattice;

// Re-export commonly used types
pub use config::{Config, generate_sample_config};
pub use args::Args;

// Re-export types that are used via crate:: paths
pub use rollup_lock::RollupLock;
pub use actions::RepoAction;
pub use cargo_tool_discovery::CargoToolDiscovery;