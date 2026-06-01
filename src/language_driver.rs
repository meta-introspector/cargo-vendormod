//! # Language Driver Trait + Registry
//!
//! Each supported language provides a driver that knows how to:
//! 1. Detect if a directory contains that language
//! 2. Split declarations natively
//! 3. Generate per-declaration flake.nix files
//!
//! Drivers are registered in the global `LANGUAGE_DRIVERS` registry and
//! dispatched by `Language` enum. This mirrors the `zos-plugins`
//! `PluginRegistry` / `PluginVerb` pattern.

use crate::lang_detect::Language;
use crate::multi_lang_flake::FlakeStyle;
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Configuration for a split operation.
#[derive(Debug, Clone, Default)]
pub struct SplitConfig {
    /// Output root directory for split declarations.
    pub output_dir: PathBuf,
    /// Flake generation style (simple / crate2nix / etc.).
    pub flake_style: FlakeStyle,
    /// Source prefix for git-mirror flake inputs (e.g. "github:owner/repo").
    pub mirror_prefix: Option<String>,
    /// Language-specific extra options.
    pub extra: toml::Value,
}

impl SplitConfig {
    pub fn with_output(mut self, dir: impl Into<PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    pub fn with_style(mut self, style: FlakeStyle) -> Self {
        self.flake_style = style;
        self
    }
}

/// Result of a successful split: list of produced declaration directories.
#[derive(Debug, Clone, Default)]
pub struct SplitResult {
    pub decls: Vec<PathBuf>,
    pub manifest: Option<PathBuf>,
}

/// A language driver owns the native splitting + flake-generation logic for one language.
pub trait LanguageDriver: Send + Sync {
    /// Human-readable driver name.
    fn name(&self) -> &'static str;

    /// The language this driver handles.
    fn language(&self) -> Language;

    /// Quick check whether `dir` looks like this language.
    fn detect(&self, dir: &Path) -> bool;

    /// Split declarations in `source_dir`, write outputs under `config.output_dir`.
    fn split_decls(&self, source_dir: &Path, config: &SplitConfig) -> Result<SplitResult>;

    /// Generate a per-declaration `flake.nix` for `decl_name` inside `decl_dir`.
    fn generate_decl_flake(
        &self,
        decl_name: &str,
        decl_dir: &Path,
        config: &SplitConfig,
    ) -> Result<()>;
}

// ---------------------------------------------------------------------------
// Registry
// ---------------------------------------------------------------------------

#[derive(Default)]
pub struct DriverRegistry {
    drivers: Vec<&'static dyn LanguageDriver>,
}

impl DriverRegistry {
    pub const fn new() -> Self {
        Self {
            drivers: Vec::new(),
        }
    }

    /// Register a driver. Order matters: first registered = first matched.
    pub fn register(mut self, driver: &'static dyn LanguageDriver) -> Self {
        self.drivers.push(driver);
        self
    }

    /// Find a driver by `Language`.
    pub fn driver_for(&self, lang: Language) -> Option<&'static dyn LanguageDriver> {
        self.drivers
            .iter()
            .find(|d| d.language() == lang)
            .copied()
    }

    /// Run detection across all registered drivers and return the first match.
    pub fn detect_driver(&self, dir: &Path) -> Option<&'static dyn LanguageDriver> {
        self.drivers.iter().find(|d| d.detect(dir)).copied()
    }

    /// All registered drivers.
    pub fn all(&self) -> &[&'static dyn LanguageDriver] {
        &self.drivers
    }
}

// Global registry — populated at link time by driver modules.
#[ctor::ctor]
static DEFAULT_REGISTRY: DriverRegistry = DriverRegistry::new()
    .register(crate::rust_driver::RustDriver)
    .register(crate::go_driver::GoDriver)
    .register(crate::python_driver::PythonDriver)
    .register(crate::c_driver::CDriver)
    .register(crate::js_driver::JsDriver)
    .register(crate::java_driver::JavaDriver)
    .register(crate::lean4_driver::Lean4Driver);

/// Public handle to the global registry.
pub fn global() -> &'static DriverRegistry {
    &DEFAULT_REGISTRY
}
