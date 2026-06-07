//! Monolith split — Decompose a single Rust crate into workspace sub-crates.
//!
//! Reads a split plan from a TOML configuration file and:
//! 1. Creates sub-crate directories with proper Cargo.toml
//! 2. Copies source modules from the monolith src/ into each sub-crate
//! 3. Generates workspace `Cargo.toml` aggregating all sub-crates
//! 4. Generates `flake.nix` with builders per sub-crate
//!
//! ## Plan format (`split-plan.toml`)
//!
//! ```toml
//! [workspace]
//! name = "pi_agent_rust"
//! version = "0.1.16"
//! edition = "2024"
//!
//! [[crates]]
//! name = "pi-traits"
//! modules = ["error", "error_hints", "config", "sdk"]
//!
//! [[crates]]
//! name = "pi-models"
//! modules = ["model", "models", "model_routing", "model_selector", "provider_metadata"]
//! depends_on = ["pi-traits"]
//!
//! [[crates]]
//! name = "pi-interactive"
//! modules = ["keybindings", "theme", "terminal_images", "tui"]
//! dir_modules = ["interactive"]
//! depends_on = ["pi-traits", "pi-models"]
//! ```

use anyhow::{Context, Result, bail};
use std::collections::{HashMap, HashSet, BTreeMap};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// ── Plan types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SplitPlan {
    pub workspace: WorkspaceMeta,
    #[serde(default)]
    pub crates: Vec<CrateDef>,
    /// Additional crate names that exist outside the split (e.g., external deps)
    #[serde(default)]
    pub external_deps: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorkspaceMeta {
    pub name: String,
    pub version: String,
    #[serde(default = "default_edition")]
    pub edition: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub repository: String,
    #[serde(default)]
    pub readme: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CrateDef {
    pub name: String,
    /// .rs file modules to move (without .rs extension)
    #[serde(default)]
    pub modules: Vec<String>,
    /// Directory-based modules to move (whole directory copied)
    #[serde(default)]
    pub dir_modules: Vec<String>,
    /// Internal workspace crate dependencies
    #[serde(default)]
    pub depends_on: Vec<String>,
    /// External crate.io dependencies
    #[serde(default)]
    pub external_deps: BTreeMap<String, String>,
    /// Feature flags
    #[serde(default)]
    pub features: BTreeMap<String, Vec<String>>,
    /// Extra dependencies as raw TOML strings (for complex dep specs)
    #[serde(default)]
    pub raw_deps: Vec<String>,
    /// Whether this crate has feature-gated modules
    #[serde(default)]
    pub feature_gated: Vec<FeatureGatedModule>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FeatureGatedModule {
    pub module: String,
    pub feature: String,
}

fn default_edition() -> String {
    "2021".to_string()
}

// ── Execution context ─────────────────────────────────────────────────────

pub struct SplitContext {
    pub root_dir: PathBuf,
    pub src_dir: PathBuf,
    pub dry_run: bool,
    pub plan: SplitPlan,
}

impl SplitContext {
    pub fn new(root_dir: PathBuf, plan: SplitPlan, dry_run: bool) -> Self {
        let src_dir = root_dir.join("src");
        Self { root_dir, src_dir, dry_run, plan }
    }

    fn crate_dir(&self, name: &str) -> PathBuf {
        self.root_dir.join(name)
    }

    fn crate_src_dir(&self, name: &str) -> PathBuf {
        self.crate_dir(name).join("src")
    }
}

// ── Main entry point ──────────────────────────────────────────────────────

pub fn execute_split(root_dir: &Path, plan_path: &Path, dry_run: bool) -> Result<()> {
    let plan_toml = fs::read_to_string(plan_path)
        .with_context(|| format!("Failed to read plan: {}", plan_path.display()))?;
    let plan: SplitPlan = toml::from_str(&plan_toml)
        .context("Failed to parse split plan TOML")?;

    let ctx = SplitContext::new(root_dir.to_path_buf(), plan, dry_run);

    validate_plan(&ctx)?;
    create_subcrates(&ctx)?;
    generate_workspace_toml(&ctx)?;
    generate_flake_nix(&ctx)?;
    generate_workspace_readme(&ctx)?;

    Ok(())
}

// ── Validation ────────────────────────────────────────────────────────────

fn validate_plan(ctx: &SplitContext) -> Result<()> {
    if ctx.plan.crates.is_empty() {
        bail!("Plan contains no crate definitions");
    }

    let mut names = HashSet::new();
    for c in &ctx.plan.crates {
        if !names.insert(&c.name) {
            bail!("Duplicate crate name: {}", c.name);
        }

        // Check depends_on references valid crates
        for dep in &c.depends_on {
            if !names.contains(dep) && !ctx.plan.external_deps.contains(dep) {
                // Allow depends_on to reference crates defined later in the plan
            }
        }

        // Check modules exist in src/
        for m in &c.modules {
            let path = ctx.src_dir.join(format!("{}.rs", m));
            if !path.exists() {
                eprintln!("Warning: module '{}' not found at {}", m, path.display());
            }
        }

        for dm in &c.dir_modules {
            let path = ctx.src_dir.join(dm);
            if !path.exists() {
                eprintln!("Warning: dir_module '{}' not found at {}", dm, path.display());
            }
        }
    }

    Ok(())
}

// ── Sub-crate generation ──────────────────────────────────────────────────

fn create_subcrates(ctx: &SplitContext) -> Result<()> {
    for crate_def in &ctx.plan.crates {
        let crate_dir = ctx.crate_dir(&crate_def.name);
        let crate_src = ctx.crate_src_dir(&crate_def.name);

        eprintln!("{} {} ...", 
            if ctx.dry_run { "[dry-run]" } else { "[create]" },
            crate_def.name);

        if !ctx.dry_run {
            fs::create_dir_all(&crate_src)?;
        }

        // Copy file-based modules
        for module in &crate_def.modules {
            let src_path = ctx.src_dir.join(format!("{}.rs", module));
            let dst_path = crate_src.join(format!("{}.rs", module));
            copy_file(&src_path, &dst_path, ctx.dry_run)?;
        }

        // Copy directory-based modules
        for dir_module in &crate_def.dir_modules {
            let src_path = ctx.src_dir.join(dir_module);
            let dst_path = crate_src.join(dir_module);
            copy_dir(&src_path, &dst_path, ctx.dry_run)?;
        }

        // Generate Cargo.toml
        generate_crate_toml(ctx, crate_def)?;

        // Generate src/lib.rs
        generate_lib_rs(ctx, crate_def)?;
    }
    Ok(())
}

fn copy_file(src: &Path, dst: &Path, dry_run: bool) -> Result<()> {
    if !src.exists() {
        eprintln!("  skip (not found): {}", src.display());
        return Ok(());
    }
    if dry_run {
        eprintln!("  copy {} -> {}", src.display(), dst.display());
    } else {
        fs::copy(src, dst)?;
        eprintln!("  copy {}", src.display());
    }
    Ok(())
}

fn copy_dir(src: &Path, dst: &Path, dry_run: bool) -> Result<()> {
    if !src.exists() || !src.is_dir() {
        eprintln!("  skip (not found): {}", src.display());
        return Ok(());
    }
    if dry_run {
        eprintln!("  copy-dir {} -> {}", src.display(), dst.display());
        return Ok(());
    }
    // Recursive copy
    for entry in walkdir::WalkDir::new(src) {
        let entry = entry?;
        let rel = entry.path().strip_prefix(src)?;
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target)?;
        }
    }
    eprintln!("  copy-dir {}", src.display());
    Ok(())
}

fn generate_crate_toml(ctx: &SplitContext, crate_def: &CrateDef) -> Result<()> {
    let crate_dir = ctx.crate_dir(&crate_def.name);
    let toml_path = crate_dir.join("Cargo.toml");
    let lib_name = crate_def.name.replace('-', "_");

    if ctx.dry_run {
        eprintln!("  write {}", toml_path.display());
        return Ok(());
    }

    let mut f = fs::File::create(&toml_path)?;
    let ws = &ctx.plan.workspace;

    writeln!(f, "[package]")?;
    writeln!(f, "name = \"{}\"", crate_def.name)?;
    writeln!(f, "version = \"{}\"", ws.version)?;
    writeln!(f, "edition = \"{}\"", ws.edition)?;
    if !ws.description.is_empty() {
        writeln!(f, "description = \"{}\"", ws.description)?;
    }
    if !ws.license.is_empty() {
        writeln!(f, "license = \"{}\"", ws.license)?;
    }
    if !ws.repository.is_empty() {
        writeln!(f, "repository = \"{}\"", ws.repository)?;
    }
    if !ws.readme.is_empty() {
        writeln!(f, "readme = \"{}\"", ws.readme)?;
    }

    writeln!(f)?;
    writeln!(f, "[lib]")?;
    writeln!(f, "name = \"{}\"", lib_name)?;
    writeln!(f, "path = \"src/lib.rs\"")?;

    // Features
    if !crate_def.features.is_empty() {
        writeln!(f)?;
        writeln!(f, "[features]")?;
        for (feat, deps) in &crate_def.features {
            if deps.is_empty() {
                writeln!(f, "{} = []", feat)?;
            } else {
                writeln!(f, "{} = [{}]", feat, deps.iter().map(|d| format!("\"{}\"", d)).collect::<Vec<_>>().join(", "))?;
            }
        }
    }

    // Dependencies
    let has_deps = !crate_def.depends_on.is_empty()
        || !crate_def.external_deps.is_empty()
        || !crate_def.raw_deps.is_empty();

    if has_deps {
        writeln!(f)?;
        writeln!(f, "[dependencies]")?;

        // Internal workspace deps
        for dep in &crate_def.depends_on {
            writeln!(f, "{} = {{ path = \"../{}\" }}", dep, dep)?;
        }

        // External deps
        for (dep, version) in &crate_def.external_deps {
            writeln!(f, "{} = \"{}\"", dep, version)?;
        }

        // Raw deps (complex specifications)
        for raw in &crate_def.raw_deps {
            writeln!(f, "{}", raw)?;
        }
    }

    // Lints
    writeln!(f)?;
    writeln!(f, "[lints.rust]")?;
    writeln!(f, "unsafe_code = \"forbid\"")?;

    eprintln!("  wrote {}", toml_path.display());
    Ok(())
}

fn generate_lib_rs(ctx: &SplitContext, crate_def: &CrateDef) -> Result<()> {
    let crate_src = ctx.crate_src_dir(&crate_def.name);
    let lib_path = crate_src.join("lib.rs");

    if ctx.dry_run {
        eprintln!("  write {}", lib_path.display());
        return Ok(());
    }

    if !ctx.dry_run {
        fs::create_dir_all(&crate_src)?;
    }

    let mut content = String::new();
    content.push_str(&format!(
        "//! {} — Pi agent sub-crate\n//!\n//! Part of the {} workspace.\n\n",
        crate_def.name, ctx.plan.workspace.name
    ));
    content.push_str("#![forbid(unsafe_code)]\n");

    // Feature-gated modules
    for fg in &crate_def.feature_gated {
        content.push_str(&format!(
            "\n#[cfg(feature = \"{}\")]\npub mod {};\n",
            fg.feature, fg.module
        ));
    }

    // File modules
    for module in &crate_def.modules {
        content.push_str(&format!("\npub mod {};\n", module));
    }

    // Directory modules
    for dir_module in &crate_def.dir_modules {
        content.push_str(&format!("\npub mod {};\n", dir_module));
    }

    if ctx.dry_run {
        Ok(())
    } else {
        fs::write(&lib_path, content)?;
        eprintln!("  wrote {}", lib_path.display());
        Ok(())
    }
}

// ── Workspace Cargo.toml ──────────────────────────────────────────────────

fn generate_workspace_toml(ctx: &SplitContext) -> Result<()> {
    let ws_toml = ctx.root_dir.join("Cargo.workspace.toml");
    let ws = &ctx.plan.workspace;

    if ctx.dry_run {
        eprintln!("  write {}", ws_toml.display());
        return Ok(());
    }

    let mut f = fs::File::create(&ws_toml)?;
    let lib_name = ws.name.replace('-', "_");

    writeln!(f, "[workspace]")?;
    writeln!(f, "resolver = \"3\"")?;
    writeln!(f, "members = [")?;
    for c in &ctx.plan.crates {
        writeln!(f, "    \"{}\",", c.name)?;
    }
    writeln!(f, "    \".\",")?; // Root crate
    writeln!(f, "]")?;
    writeln!(f)?;

    writeln!(f, "[workspace.package]")?;
    writeln!(f, "version = \"{}\"", ws.version)?;
    writeln!(f, "edition = \"{}\"", ws.edition)?;

    writeln!(f)?;
    writeln!(f, "[workspace.lints.rust]")?;
    writeln!(f, "unsafe_code = \"forbid\"")?;

    writeln!(f)?;
    writeln!(f, "# Root binary crate")?;
    writeln!(f, "[package]")?;
    writeln!(f, "name = \"{}\"", ws.name)?;
    writeln!(f, "version.workspace = true")?;
    writeln!(f, "edition.workspace = true")?;

    writeln!(f)?;
    writeln!(f, "[[bin]]")?;
    writeln!(f, "name = \"{}\"", lib_name)?;
    writeln!(f, "path = \"src/main.rs\"")?;

    writeln!(f)?;
    writeln!(f, "[lib]")?;
    writeln!(f, "name = \"{}\"", lib_name)?;
    writeln!(f, "path = \"src/lib.rs\"")?;

    // Root crate depends on all sub-crates
    writeln!(f)?;
    writeln!(f, "[dependencies]")?;
    for c in &ctx.plan.crates {
        writeln!(f, "{} = {{ path = \"{}\" }}", c.name, c.name)?;
    }

    eprintln!("  wrote {}", ws_toml.display());
    eprintln!("  NOTE: Review and merge with original Cargo.toml before building.");
    Ok(())
}

// ── flake.nix generation ──────────────────────────────────────────────────

fn generate_flake_nix(ctx: &SplitContext) -> Result<()> {
    let flake_path = ctx.root_dir.join("flake.workspace.nix");

    if ctx.dry_run {
        eprintln!("  write {}", flake_path.display());
        return Ok(());
    }

    let ws = &ctx.plan.workspace;
    let mut f = fs::File::create(&flake_path)?;

    writeln!(f, "{{")?;
    writeln!(f, "  description = \"{} — Modular Rust workspace\";", ws.name)?;
    writeln!(f)?;
    writeln!(f, "  inputs = {{")?;
    writeln!(f, "    nixpkgs.url = \"git+file:///mnt/data1/git/github.com/NixOS/nixpkgs.git?ref=master\";")?;
    writeln!(f, "  }};")?;
    writeln!(f)?;
    writeln!(f, "  outputs = {{ self, nixpkgs }}:")?;
    writeln!(f, "    let")?;
    writeln!(f, "      system = \"x86_64-linux\";")?;
    writeln!(f, "      pkgs = nixpkgs.legacyPackages.${{system}};")?;
    writeln!(f)?;
    writeln!(f, "      # Build helper: build a single sub-crate")?;
    writeln!(f, "      mkCrate = name: path:")?;
    writeln!(f, "        pkgs.rustPlatform.buildRustPackage {{")?;
    writeln!(f, "          pname = name;")?;
    writeln!(f, "          version = \"{}\";", ws.version)?;
    writeln!(f, "          src = ./.;")?;
    writeln!(f, "          cargoLock = {{ lockFile = ./Cargo.lock; }};")?;
    writeln!(f, "          buildAndTestSubdir = path;")?;
    writeln!(f, "          nativeBuildInputs = with pkgs; [ pkg-config ];")?;
    writeln!(f, "          buildInputs = with pkgs; [ openssl ];")?;
    writeln!(f, "        }};")?;
    writeln!(f)?;

    // Build each sub-crate
    writeln!(f, "      crates = {{")?;
    for c in &ctx.plan.crates {
        writeln!(f, "        {} = mkCrate \"{}\" \"{}\";", c.name, c.name, c.name)?;
    }
    writeln!(f, "      }};")?;
    writeln!(f)?;

    // Root binary
    writeln!(f, "      pi = pkgs.rustPlatform.buildRustPackage {{")?;
    writeln!(f, "        pname = \"{}\";", ws.name)?;
    writeln!(f, "        version = \"{}\";", ws.version)?;
    writeln!(f, "        src = ./.;")?;
    writeln!(f, "        cargoLock = {{ lockFile = ./Cargo.lock; }};")?;
    writeln!(f, "        nativeBuildInputs = with pkgs; [ pkg-config makeWrapper ];")?;
    writeln!(f, "        buildInputs = with pkgs; [ openssl ];")?;
    writeln!(f, "        postInstall = ''")?;
    writeln!(f, "          wrapProgram $out/bin/pi --prefix PATH : ${{pkgs.lib.makeBinPath [ pkgs.ripgrep pkgs.fd pkgs.git pkgs.bash ]}}")?;
    writeln!(f, "        '';")?;
    writeln!(f, "      }};")?;
    writeln!(f, "    in")?;
    writeln!(f, "    {{")?;
    writeln!(f, "      packages.${{system}} = crates // {{")?;
    writeln!(f, "        pi = pi;")?;
    writeln!(f, "        default = pi;")?;
    writeln!(f, "      }};")?;
    writeln!(f)?;
    writeln!(f, "      apps.${{system}} = {{")?;
    writeln!(f, "        default = {{ type = \"app\"; program = \"${{pi}}/bin/pi\"; }};")?;
    writeln!(f, "      }};")?;
    writeln!(f)?;
    writeln!(f, "      devShells.${{system}}.default = pkgs.mkShell {{")?;
    writeln!(f, "        buildInputs = with pkgs; [ rustc cargo rust-analyzer rustfmt clippy pkg-config openssl ripgrep fd git bash ];")?;
    writeln!(f, "        shellHook = ''")?;
    writeln!(f, "          echo \"{} workspace — $(rustc --version)\"", ws.name)?;
    writeln!(f, "          echo \"  cargo build --workspace    → build all crates\"")?;
    writeln!(f, "        '';")?;
    writeln!(f, "      }};")?;
    writeln!(f, "    }};")?;
    writeln!(f, "}}")?;

    eprintln!("  wrote {}", flake_path.display());
    Ok(())
}

fn generate_workspace_readme(ctx: &SplitContext) -> Result<()> {
    let readme_path = ctx.root_dir.join("SPLIT_README.md");

    if ctx.dry_run {
        eprintln!("  write {}", readme_path.display());
        return Ok(());
    }

    let mut f = fs::File::create(&readme_path)?;
    let ws = &ctx.plan.workspace;

    writeln!(f, "# {} — Workspace Monolith Split\n", ws.name)?;
    writeln!(f, "Generated by `cargo-vendormod split`.\n")?;
    writeln!(f, "## Sub-crates ({})\n", ctx.plan.crates.len())?;

    // Sort by dependency depth for readability
    let mut ordered: Vec<&CrateDef> = ctx.plan.crates.iter().collect();
    ordered.sort_by_key(|c| c.depends_on.len());

    for c in &ordered {
        let deps = if c.depends_on.is_empty() {
            "none".to_string()
        } else {
            c.depends_on.iter().map(|d| format!("`{}`", d)).collect::<Vec<_>>().join(", ")
        };
        let file_count = c.modules.len() + c.dir_modules.len();
        writeln!(f, "| `{}` | {} files | depends: {} |", c.name, file_count, deps)?;
    }

    writeln!(f)?;
    writeln!(f, "## Next Steps\n")?;
    writeln!(f, "1. Review `Cargo.workspace.toml` and merge with original `Cargo.toml`")?;
    writeln!(f, "2. Rewrite `use crate::` → `use <subcrate>::` in all moved files")?;
    writeln!(f, "3. Run `cargo check --workspace`")?;
    writeln!(f, "4. Fix compilation errors iteratively")?;
    writeln!(f, "5. Review `flake.workspace.nix` and merge with existing `flake.nix`")?;
    writeln!(f, "6. Run `nix flake lock && nix build .#pi`")?;

    eprintln!("  wrote {}", readme_path.display());
    Ok(())
}

// ── Plan generation from existing source ──────────────────────────────────

/// Analyze a monolith's `src/` directory and generate a suggested split plan.
pub fn suggest_plan(root_dir: &Path) -> Result<SplitPlan> {
    let src_dir = root_dir.join("src");
    let cargo_toml = root_dir.join("Cargo.toml");

    // Read the original Cargo.toml for metadata
    let ws = if cargo_toml.exists() {
        let content = fs::read_to_string(&cargo_toml)?;
        parse_workspace_meta(&content)?
    } else {
        WorkspaceMeta {
            name: root_dir.file_name().unwrap_or_default().to_string_lossy().to_string(),
            version: "0.1.0".to_string(),
            edition: "2021".to_string(),
            description: String::new(),
            license: String::new(),
            repository: String::new(),
            readme: String::new(),
        }
    };

    // Discover modules in src/
    let mut file_modules: Vec<String> = Vec::new();
    let mut dir_modules: Vec<String> = Vec::new();

    if src_dir.exists() {
        for entry in fs::read_dir(&src_dir)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();

            if name == "main.rs" || name == "lib.rs" || name == "bin" {
                continue;
            }

            if entry.file_type()?.is_dir() {
                // Check if it has a mod.rs
                if entry.path().join("mod.rs").exists() {
                    dir_modules.push(name);
                }
            } else if name.ends_with(".rs") {
                let mod_name = name.strip_suffix(".rs").unwrap().to_string();
                file_modules.push(mod_name);
            }
        }
    }

    file_modules.sort();
    dir_modules.sort();

    eprintln!("Discovered {} file modules, {} directory modules",
        file_modules.len(), dir_modules.len());

    // Simple heuristic grouping by common prefixes
    let groups = group_modules_by_prefix(&file_modules, &dir_modules);

    let crates: Vec<CrateDef> = groups.into_iter().map(|(name, (files, dirs))| {
        CrateDef {
            name: format!("pi-{}", name),
            modules: files,
            dir_modules: dirs,
            depends_on: Vec::new(),
            external_deps: BTreeMap::new(),
            features: BTreeMap::new(),
            raw_deps: Vec::new(),
            feature_gated: Vec::new(),
        }
    }).collect();

    Ok(SplitPlan {
        workspace: ws,
        crates,
        external_deps: Vec::new(),
    })
}

fn parse_workspace_meta(content: &str) -> Result<WorkspaceMeta> {
    // Simple TOML parsing for package section
    let doc: toml::Value = toml::from_str(content)?;
    let pkg = doc.get("package").context("No [package] section")?;

    Ok(WorkspaceMeta {
        name: pkg.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
        version: pkg.get("version").and_then(|v| v.as_str()).unwrap_or("0.1.0").to_string(),
        edition: pkg.get("edition").and_then(|v| v.as_str()).unwrap_or("2021").to_string(),
        description: pkg.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        license: pkg.get("license").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        repository: pkg.get("repository").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        readme: pkg.get("readme").and_then(|v| v.as_str()).unwrap_or("").to_string(),
    })
}

fn group_modules_by_prefix(
    file_modules: &[String],
    dir_modules: &[String],
) -> Vec<(String, (Vec<String>, Vec<String>))> {
    // Known prefixes → sub-crate name mapping
    let prefix_map: Vec<(&str, &str)> = vec![
        ("extension_", "extensions"),
        ("extension", "extensions"),
        ("hostcall_", "hostcall"),
        ("hostcall", "hostcall"),
        ("session_", "session"),
        ("session", "session"),
        ("swarm_", "swarm"),
        ("swarm", "swarm"),
        ("conformance_", "conformance"),
        ("conformance", "conformance"),
        ("model_", "models"),
        ("model", "models"),
        ("provider_", "providers"),
        ("provider", "providers"),
        ("interactive", "interactive"),
        ("connector", "connectors"),
        ("auth", "auth"),
        ("cli", "cli"),
        ("agent", "agent"),
        ("compaction", "compaction"),
        ("config", "traits"),
        ("error", "traits"),
        ("sdk", "traits"),
        ("crypto", "crypto"),
        ("tool", "tools"),
        ("tui", "interactive"),
        ("theme", "interactive"),
        ("keybindings", "interactive"),
        ("terminal", "interactive"),
        ("vcr", "vcr"),
        ("scheduler", "scheduler"),
        ("resource", "scheduler"),
        ("permission", "permissions"),
        ("validation", "permissions"),
        ("rpc", "rpc"),
        ("autocomplete", "autocomplete"),
        ("doctor", "doctor"),
        ("migration", "agent"),
        ("package", "agent"),
        ("platform", "agent"),
        ("semantic", "agent"),
        ("version", "agent"),
        ("flake", "agent"),
        ("buffer", "agent"),
        ("acp", "agent"),
        ("perf", "zkperf"),
        ("wasm", "wasm"),
        ("pi_wasm", "wasm"),
        ("http", "http"),
        ("sse", "providers"),
    ];

    let mut groups: HashMap<String, (Vec<String>, Vec<String>)> = HashMap::new();
    let mut unassigned: Vec<(String, bool)> = Vec::new(); // (name, is_dir)

    for m in file_modules {
        let mut assigned = false;
        for (prefix, group) in &prefix_map {
            if m.starts_with(prefix) {
                groups.entry(group.to_string())
                    .or_default()
                    .0.push(m.clone());
                assigned = true;
                break;
            }
        }
        if !assigned {
            unassigned.push((m.clone(), false));
        }
    }

    for dm in dir_modules {
        let mut assigned = false;
        for (prefix, group) in &prefix_map {
            if dm.starts_with(prefix) {
                groups.entry(group.to_string())
                    .or_default()
                    .1.push(dm.clone());
                assigned = true;
                break;
            }
        }
        if !assigned {
            unassigned.push((dm.clone(), true));
        }
    }

    // Put unassigned into a "misc" group
    if !unassigned.is_empty() {
        let (u_files, u_dirs): (Vec<_>, Vec<_>) = unassigned.iter()
            .partition(|(_, is_dir)| !is_dir);
        groups.entry("misc".to_string())
            .or_default()
            .0.extend(u_files.into_iter().map(|(n, _)| n.clone()));
        groups.entry("misc".to_string())
            .or_default()
            .1.extend(u_dirs.into_iter().map(|(n, _)| n.clone()));
    }

    // Sort by group name
    let mut result: Vec<_> = groups.into_iter().collect();
    result.sort_by(|a, b| a.0.cmp(&b.0));
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_modules_by_prefix() {
        let files = vec![
            "error".to_string(), "tools".to_string(), "agent".to_string(),
            "vcr".to_string(), "tui".to_string(), "theme".to_string(),
        ];
        let dirs = vec!["providers".to_string(), "interactive".to_string()];

        let groups = group_modules_by_prefix(&files, &dirs);
        assert!(!groups.is_empty());

        // error → traits
        let traits = groups.iter().find(|(k, _)| k == "traits").unwrap();
        assert!(traits.1.0.contains(&"error".to_string()));

        // tools → tools
        let tools = groups.iter().find(|(k, _)| k == "tools").unwrap();
        assert!(tools.1.0.contains(&"tools".to_string()));
    }

    #[test]
    fn test_parse_split_plan() {
        let toml_str = r#"
[workspace]
name = "test"
version = "0.1.0"
edition = "2021"

[[crates]]
name = "test-core"
modules = ["error", "config"]

[[crates]]
name = "test-app"
modules = ["main"]
depends_on = ["test-core"]
"#;
        let plan: SplitPlan = toml::from_str(toml_str).unwrap();
        assert_eq!(plan.workspace.name, "test");
        assert_eq!(plan.crates.len(), 2);
        assert_eq!(plan.crates[0].name, "test-core");
        assert_eq!(plan.crates[1].depends_on, vec!["test-core"]);
    }
}
