//! # Flake Coverage Check
//!
//! Reports the status of Nix flake coverage for all target CBOR libraries,
//! fuzz tools, and test suites. Used by `cargo-vendormod flake-check` to
//! show which projects have flakes and which are still missing them.

use anyhow::{Context, Result};
use serde::Serialize;
use std::path::Path;

// ── Project classification ──────────────────────────────────────────────

/// Category a target project belongs to.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProjectCategory {
    /// Nix buildable project provided via flake
    MainProject,
    /// CBOR/IPLD serialisation library
    CborLib,
    /// Fuzzing harness or fuzz-testing tool
    FuzzTool,
    /// Conformance test suite or test fixtures
    TestSuite,
}

impl std::fmt::Display for ProjectCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectCategory::MainProject => write!(f, "Main Project"),
            ProjectCategory::CborLib => write!(f, "CBOR Lib"),
            ProjectCategory::FuzzTool => write!(f, "Fuzz Tool"),
            ProjectCategory::TestSuite => write!(f, "Test Suite"),
        }
    }
}

/// A single project that we expect to have a Nix flake.
#[derive(Debug, Clone, Serialize)]
pub struct TargetProject {
    /// Short display name
    pub name: String,
    /// Category of the project
    pub category: ProjectCategory,
    /// Git source (e.g. "ipld/rust-ipld-core")
    pub source: String,
    /// Expected flake directory name under the flakes root
    pub flake_dir: String,
    /// Whether `flake.nix` was found
    #[serde(skip)]
    pub has_flake_nix: bool,
    /// Whether `flake.lock` was found
    #[serde(skip)]
    pub has_flake_lock: bool,
}

/// Pre-defined list of all projects we want flakes for.
///
/// **Goal #1**: Nix flakes for all CBOR libs, fuzz tools, and test suites.
///
/// All repos must have flakes — Rust projects use crate2nix from nix-common,
/// non-Rust projects (specs, codec-fixtures) provide devShells with their
/// toolchains. libipld-fuzz is inside the ipld/libipld repo but gets its own
/// flake directory for independent fuzz-target builds.
///
/// Current tally: **9 targets total**.
pub fn target_projects() -> Vec<TargetProject> {
    vec![
        // ── Main Project ──────────────────────────────────────────
        TargetProject {
            name: "dasl".into(),
            category: ProjectCategory::MainProject,
            source: "n0-computer/dasl".into(),
            flake_dir: "dasl".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        // ── CBOR / IPLD Libraries ─────────────────────────────────
        TargetProject {
            name: "ipld-core".into(),
            category: ProjectCategory::CborLib,
            source: "ipld/rust-ipld-core".into(),
            flake_dir: "ipld-core".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        TargetProject {
            name: "serde_ipld_dagcbor".into(),
            category: ProjectCategory::CborLib,
            source: "ipld/serde_ipld_dagcbor".into(),
            flake_dir: "serde_ipld_dagcbor".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        TargetProject {
            name: "libipld (w/ fuzz)".into(),
            category: ProjectCategory::CborLib,
            source: "ipld/libipld".into(),
            flake_dir: "libipld".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        TargetProject {
            name: "ipld-dagpb".into(),
            category: ProjectCategory::CborLib,
            source: "ipld/rust-ipld-dagpb".into(),
            flake_dir: "rust-ipld-dagpb".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        TargetProject {
            name: "serde_cbor (pyfisch)".into(),
            category: ProjectCategory::CborLib,
            source: "pyfisch/cbor".into(),
            flake_dir: "pyfisch-cbor".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        // ── Fuzz Tools ───────────────────────────────────────────
        TargetProject {
            name: "libipld-fuzz".into(),
            category: ProjectCategory::FuzzTool,
            source: "ipld/libipld (fuzz/ dir)".into(),
            flake_dir: "libipld-fuzz".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        // ── Test Suites ───────────────────────────────────────────
        TargetProject {
            name: "codec-fixtures".into(),
            category: ProjectCategory::TestSuite,
            source: "ipld/codec-fixtures".into(),
            flake_dir: "codec-fixtures".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
        TargetProject {
            name: "specs".into(),
            category: ProjectCategory::TestSuite,
            source: "ipld/specs".into(),
            flake_dir: "specs".into(),
            has_flake_nix: false,
            has_flake_lock: false,
        },
    ]
}

// ── Runtime scan ───────────────────────────────────────────────────────

/// Result of scanning a set of target projects.
#[derive(Debug, Clone, Serialize)]
pub struct FlakeCoverageReport {
    /// Per-project status entries
    pub projects: Vec<ProjectStatus>,
    /// Overall stats
    pub summary: CoverageSummary,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProjectStatus {
    pub name: String,
    pub category: ProjectCategory,
    pub source: String,
    pub status: ProjectFlakeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProjectFlakeStatus {
    /// Both flake.nix and flake.lock present
    Complete,
    /// Only flake.nix (no lock file)
    Partial,
    /// Neither present
    Missing,
}

impl std::fmt::Display for ProjectFlakeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectFlakeStatus::Complete => write!(f, "✅ Complete"),
            ProjectFlakeStatus::Partial => write!(f, "⚠ Partial"),
            ProjectFlakeStatus::Missing => write!(f, "❌ Missing"),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CoverageSummary {
    pub total: usize,
    pub complete: usize,
    pub partial: usize,
    pub missing: usize,
    pub coverage_pct: f64,
}

/// Scan the `flakes_dir` and determine which target projects have flakes.
pub fn check_flake_coverage(flakes_dir: &Path) -> Result<FlakeCoverageReport> {
    let mut projects: Vec<ProjectStatus> = Vec::new();
    let mut complete = 0usize;
    let mut partial = 0usize;
    let mut missing = 0usize;

    for target in target_projects() {
        let project_dir = flakes_dir.join(&target.flake_dir);
        let has_nix = project_dir.join("flake.nix").exists();
        let has_lock = project_dir.join("flake.lock").exists();

        let status = if has_nix && has_lock {
            complete += 1;
            ProjectFlakeStatus::Complete
        } else if has_nix {
            partial += 1;
            ProjectFlakeStatus::Partial
        } else {
            missing += 1;
            ProjectFlakeStatus::Missing
        };

        projects.push(ProjectStatus {
            name: target.name,
            category: target.category,
            source: target.source,
            status,
        });
    }

    let total = projects.len();
    let coverage_pct = if total > 0 {
        (complete as f64 / total as f64) * 100.0
    } else {
        0.0
    };

    Ok(FlakeCoverageReport {
        projects,
        summary: CoverageSummary {
            total,
            complete,
            partial,
            missing,
            coverage_pct,
        },
    })
}

// ── Display helpers ────────────────────────────────────────────────────

/// Print a human-readable coverage report to stdout.
pub fn print_report(report: &FlakeCoverageReport) {
    println!();
    println!("═══════════════════════════════════════════════════════════════════════");
    println!("  Flake Coverage Report — Goal #1");
    println!("  Nix flakes for all CBOR libs, fuzz tools, and test suites");
    println!("═══════════════════════════════════════════════════════════════════════\n");

    // Group by category
    use std::collections::BTreeMap;
    let mut by_category: BTreeMap<&str, Vec<&ProjectStatus>> = BTreeMap::new();
    for p in &report.projects {
        let key = match p.category {
            ProjectCategory::MainProject => "Main Projects",
            ProjectCategory::CborLib => "CBOR / IPLD Libraries",
            ProjectCategory::FuzzTool => "Fuzz Tools",
            ProjectCategory::TestSuite => "Test Suites",
        };
        by_category.entry(key).or_default().push(p);
    }

    for (category_label, projects) in &by_category {
        println!("  ┌─ {} ─────────────────────────────────────────────", category_label);
        for p in projects {
            println!("  │ {}  {}  ({})", p.status, p.name, p.source);
        }
        println!("  └────────────────────────────────────────────────────────────────");
        println!();
    }

    // Summary bar
    let s = &report.summary;
    let bar_width: usize = 40;
    let filled = (s.coverage_pct / 100.0 * bar_width as f64).round() as usize;
    let empty = bar_width.saturating_sub(filled);

    println!("  Summary:");
    println!("    Total projects : {}", s.total);
    println!("    Complete       : {}  {}/{}",
        if s.complete == s.total { "✅ All done!" } else { "" },
        s.complete, s.total
    );
    println!("    Partial        : {}  (flake.nix without flake.lock)", s.partial);
    println!("    Missing        : {}  (no flake yet)", s.missing);
    println!("    Coverage       : {:.1}%", s.coverage_pct);
    print!("    [{}{}]", "█".repeat(filled), "░".repeat(empty));
    println!("  {:.0}%\n", s.coverage_pct);
}

/// Serialise the report as JSON.
pub fn print_json(report: &FlakeCoverageReport) -> Result<()> {
    let json = serde_json::to_string_pretty(report)?;
    println!("{}", json);
    Ok(())
}
