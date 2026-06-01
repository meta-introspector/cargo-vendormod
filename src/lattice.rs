//! # Lattice Generator
//!
//! Generates a complete "lattice node" for each project in the DASL ecosystem.
//! Each node is a self-contained build context with:
//!
//! - `flake.nix` + `package.nix` — Nix build via nora
//! - `pipelight.toml` — CI pipeline: build → publish to nora → verify
//! - `.cargo/config.toml` — Cargo registry redirect to nora
//! - `SKILL.md` — Project knowledge context for agent consumption
//!
//! The lattice connects projects through shared nora registry references,
//! git mirrors, and dependency relationships.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::crate_flake::{self, FlakeSource, FlakeStyle};

/// .cargo/config.toml content for redirecting to nora registry
const NORA_CARGO_CONFIG: &str = r#"[source.crates-io]
replace-with = "nora"

[source.nora]
registry = "http://127.0.0.1:4000/cargo/index"
"#;

/// A project discovered in the DASL ecosystem
#[derive(Debug, Clone)]
pub struct LatticeProject {
    pub name: String,
    pub crate_name: String,
    pub version: String,
    pub source_dir: PathBuf,
    pub language: String,
    pub description: String,
    pub git_mirror: Option<PathBuf>,
    pub dependencies: Vec<String>,
}

/// Generate a complete lattice node for a project
pub fn generate_lattice_node(
    project: &LatticeProject,
    output_dir: &Path,
    nora: bool,
) -> Result<usize> {
    let node_dir = output_dir.join(&project.crate_name);
    fs::create_dir_all(&node_dir)
        .with_context(|| format!("Failed to create lattice node dir: {}", node_dir.display()))?;

    let mut file_count: usize = 0;

    // 1. Generate flake.nix + package.nix (via crate_flake module)
    if project.language == "Rust" {
        file_count += crate_flake::generate_flake(
            &project.source_dir,
            output_dir,
            None, // no vendor_dir
            FlakeStyle::Simple,
            None, // no nix_common
            FlakeSource::LocalDir,
            nora,
        )?;
    }

    // 2. Generate pipelight.toml
    let pipelight = render_pipelight_toml(project, nora);
    fs::write(node_dir.join("pipelight.toml"), &pipelight)
        .with_context(|| format!("Failed to write pipelight.toml for {}", project.crate_name))?;
    file_count += 1;

    // 3. Generate .cargo/config.toml (for non-nix cargo builds)
    if nora {
        let cargo_dir = node_dir.join(".cargo");
        fs::create_dir_all(&cargo_dir)?;
        fs::write(cargo_dir.join("config.toml"), NORA_CARGO_CONFIG)?;
        file_count += 1;
    }

    // 4. Generate SKILL.md
    let skill = render_skill_md(project);
    fs::write(node_dir.join("SKILL.md"), &skill)
        .with_context(|| format!("Failed to write SKILL.md for {}", project.crate_name))?;
    file_count += 1;

    // 5. Copy Cargo.lock
    let lock_src = project.source_dir.join("Cargo.lock");
    if lock_src.exists() {
        fs::copy(&lock_src, node_dir.join("Cargo.lock"))
            .with_context(|| format!("Failed to copy Cargo.lock from {}", lock_src.display()))?;
        file_count += 1;
    }

    Ok(file_count)
}

/// Render pipelight.toml for a single project
fn render_pipelight_toml(project: &LatticeProject, nora: bool) -> String {
    let crate_name = &project.crate_name;
    let version = &project.version;
    let source_dir = project.source_dir.display();

    let mut lines = Vec::new();

    lines.push(format!("# {} — Pipelight CI pipeline", crate_name));
    lines.push(format!("# Run: pipelight run build-{}", crate_name));
    lines.push(String::new());

    // Pipeline 1: Build
    lines.push("[[pipelines]]".to_string());
    lines.push(format!("name = \"build-{}\"", crate_name));
    lines.push("triggers = [{ actions = [\"manual\"] }]".to_string());
    lines.push(String::new());

    lines.push("[[pipelines.steps]]".to_string());
    lines.push("name = \"nix-build\"".to_string());
    lines.push("commands = [".to_string());
    lines.push(format!("  \"echo '=== Building {} v{} ==='\",", crate_name, version));
    lines.push(format!("  \"cd {} && nix build --no-link --print-build-logs\",", source_dir));
    lines.push("  \"echo '  -> Build complete'\"".to_string());
    lines.push("]".to_string());
    lines.push(String::new());

    // Pipeline 2: Publish to nora
    if nora && project.language == "Rust" {
        lines.push("[[pipelines]]".to_string());
        lines.push(format!("name = \"publish-{}\"", crate_name));
        lines.push("triggers = [{ actions = [\"manual\"] }]".to_string());
        lines.push(String::new());

        lines.push("[[pipelines.steps]]".to_string());
        lines.push("name = \"publish-to-nora\"".to_string());
        lines.push("commands = [".to_string());
        lines.push(format!("  \"echo '=== Publishing {} v{} to nora ==='\",", crate_name, version));
        lines.push(format!("  \"cd {} && cargo publish --registry nora --allow-dirty 2>&1\",", source_dir));
        lines.push("  \"echo '  -> Published'\"".to_string());
        lines.push("]".to_string());
        lines.push(String::new());

        lines.push("[[pipelines.steps]]".to_string());
        lines.push("name = \"verify-in-nora\"".to_string());
        lines.push("commands = [".to_string());
        lines.push(format!("  \"echo '=== Verifying {} in nora ==='\",", crate_name));
        lines.push(format!("  \"curl -s http://127.0.0.1:4000/cargo/api/v1/crates/{} 2>&1 | head -5\",", crate_name));
        lines.push("  \"echo '  -> Verified'\"".to_string());
        lines.push("]".to_string());
        lines.push(String::new());
    }

    // Pipeline 3: Full CI (build + publish + verify)
    if nora && project.language == "Rust" {
        lines.push("[[pipelines]]".to_string());
        lines.push(format!("name = \"ci-{}\"", crate_name));
        lines.push("triggers = [{ actions = [\"manual\"] }]".to_string());
        lines.push(String::new());

        lines.push("[[pipelines.steps]]".to_string());
        lines.push("name = \"build\"".to_string());
        lines.push("commands = [".to_string());
        lines.push(format!("  \"echo '=== CI: {} v{} ==='\",", crate_name, version));
        lines.push(format!("  \"cd {} && nix build --no-link --print-build-logs\",", source_dir));
        lines.push("]".to_string());
        lines.push(String::new());

        lines.push("[[pipelines.steps]]".to_string());
        lines.push("name = \"publish\"".to_string());
        lines.push("commands = [".to_string());
        lines.push(format!("  \"cd {} && cargo publish --registry nora --allow-dirty 2>&1\",", source_dir));
        lines.push("]".to_string());
        lines.push(String::new());

        lines.push("[[pipelines.steps]]".to_string());
        lines.push("name = \"verify\"".to_string());
        lines.push("commands = [".to_string());
        lines.push(format!("  \"curl -s http://127.0.0.1:4000/cargo/api/v1/crates/{} 2>&1 | head -3\",", crate_name));
        lines.push("]".to_string());
    }

    lines.join("\n")
}

/// Render SKILL.md for a project
fn render_skill_md(project: &LatticeProject) -> String {
    let crate_name = &project.crate_name;
    let version = &project.version;
    let language = &project.language;
    let source_dir = project.source_dir.display();
    let description = if project.description.is_empty() {
        format!("{} crate (v{})", crate_name, version)
    } else {
        project.description.clone()
    };

    let dep_list = if project.dependencies.is_empty() {
        "  (none detected)".to_string()
    } else {
        project
            .dependencies
            .iter()
            .map(|d| format!("  - {}", d))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let mirror_section = if let Some(ref mirror) = project.git_mirror {
        format!(
            "## Git Mirror\n\n\
             - Bare mirror: `{}`\n\
             - Working copy: `{}`\n\
             - Remotes: `origin` → local mirror, `upstream` → upstream",
            mirror.display(),
            source_dir,
        )
    } else {
        "## Git Mirror\n\n(no mirror configured)".to_string()
    };

    let pipelight_section = if project.language == "Rust" {
        format!(
            "## Pipelight\n\n\
             ```bash\n\
             pipelight run build-{crate_name}    # nix build\n\
             pipelight run publish-{crate_name}  # publish to nora\n\
             pipelight run ci-{crate_name}       # build + publish + verify\n\
             ```"
        )
    } else {
        format!(
            "## Pipelight\n\n\
             ```bash\n\
             pipelight run build-{crate_name}    # nix build\n\
             ```"
        )
    };

    let nora_section = if project.language == "Rust" {
        format!(
            "## Nora\n\n\
             This crate is served from the local nora registry.\n\
             Other flakes consume it via:\n\n\
             ```toml\n\
             [source.nora]\n\
             registry = \"http://127.0.0.1:4000/cargo/index\"\n\
             ```"
        )
    } else {
        String::new()
    };

    format!(
        "# {crate_name}\n\n\
         {description}\n\n\
         ## Metadata\n\n\
         - **Language**: {language}\n\
         - **Version**: {version}\n\
         - **Source**: `{source_dir}`\n\
         - **Registry**: nora (http://127.0.0.1:4000/cargo/index)\n\n\
         {mirror_section}\n\n\
         ## Dependencies\n\n\
         {dep_list}\n\n\
         ## Nix Build\n\n\
         ```bash\n\
         nix build\n\
         ```\n\n\
         {pipelight_section}\n\n\
         {nora_section}\n",
        crate_name = crate_name,
        description = description,
        language = language,
        version = version,
        source_dir = source_dir,
        mirror_section = mirror_section,
        dep_list = dep_list,
        pipelight_section = pipelight_section,
        nora_section = nora_section,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_pipelight_rust() {
        let project = LatticeProject {
            name: "ipld-core".to_string(),
            crate_name: "libipld-core".to_string(),
            version: "0.4.3".to_string(),
            source_dir: PathBuf::from("/home/mdupont/dasl/rust/ipld-core"),
            language: "Rust".to_string(),
            description: "IPLD core types".to_string(),
            git_mirror: Some(PathBuf::from("/home/mdupont/git/github.com/ipld/rust-ipld-core.git")),
            dependencies: vec!["cid".to_string(), "serde".to_string()],
        };
        let toml = render_pipelight_toml(&project, true);
        assert!(toml.contains("build-libipld-core"));
        assert!(toml.contains("publish-libipld-core"));
        assert!(toml.contains("ci-libipld-core"));
        assert!(toml.contains("cargo publish --registry nora"));
    }

    #[test]
    fn test_render_skill_md() {
        let project = LatticeProject {
            name: "ipld-core".to_string(),
            crate_name: "libipld-core".to_string(),
            version: "0.4.3".to_string(),
            source_dir: PathBuf::from("/home/mdupont/dasl/rust/ipld-core"),
            language: "Rust".to_string(),
            description: "IPLD core types".to_string(),
            git_mirror: None,
            dependencies: vec!["cid".to_string()],
        };
        let md = render_skill_md(&project);
        assert!(md.contains("# libipld-core"));
        assert!(md.contains("cid"));
        assert!(md.contains("nora"));
    }
}
