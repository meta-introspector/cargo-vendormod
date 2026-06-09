//! # Main CLI Dispatcher
//!
//! Cargo-vendormod main entry point — all command logic runs in-process.

use anyhow::{Context, Result};
use clap::Parser;
use cargo_vendormod::Args;
use cargo_vendormod::args::{
    Commands, VendoringCmd, IngestArgs, MemecacheUpgradeArgs, MemecacheGcArgs, ScanIndexArgs, ScanDeltaArgs, WorkloadDef
};
use std::path::{Path, PathBuf};
use cargo_vendormod::config::Config;
use cargo_vendormod::warm_manager;
use toml;
use git_config;
use cargo_vendormod::nora_indexer::run_nora_index;
use cargo_vendormod::utils::{
    chrono_now, compute_cid, shmem_put, IngestedSubmodule,
    handle_memecache_upgrade, handle_memecache_gc, handle_scan_index,
};

fn main() -> Result<()> {
    let args = Args::parse();


    // Load configuration
    let mut config = match cargo_vendormod::config::Config::load(args.config.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Warning: Could not load config: {}", e);
            cargo_vendormod::config::Config::default()
        }
    };

    match args.command {
        Some(Commands::Warm(warm_cmd)) => {
            warm_manager::handle_warm_command(&warm_cmd.cmd, &mut config)
        }
        // Vendoring sub-commands
        Some(Commands::Vendoring(ref vcmd)) => {
            match &vcmd.cmd {
                Some(VendoringCmd::Init) => handle_init(&args, &config, None),
                Some(VendoringCmd::FetchUpstream) => handle_fetch_upstream(&args, &config),
                Some(VendoringCmd::Rebase) => handle_rebase(&args, &config),
                Some(VendoringCmd::Releases) => handle_releases(&args, &config),
                Some(VendoringCmd::Status) => handle_status(&args, &config),
                Some(VendoringCmd::Sync) => handle_sync(&args, &config),
                Some(VendoringCmd::Patch) => handle_patch(&args, &config),
                None => handle_init(&args, &config, None),
            }
        }
        Some(Commands::Graph(_gcmd)) => {
            println!("Graph commands not yet implemented in-process");
            Ok(())
        }
        Some(Commands::Process(_pcmd)) => {
            println!("Process commands not yet implemented in-process");
            Ok(())
        }

        // Inline commands — call vendoring_cmds directly
        Some(Commands::Init(ref a)) => handle_init(&args, &config, Some(&a.source)),
        Some(Commands::FetchUpstream) => handle_fetch_upstream(&args, &config),
        Some(Commands::Rebase) => handle_rebase(&args, &config),
        Some(Commands::Releases) => handle_releases(&args, &config),
        Some(Commands::Status) => handle_status(&args, &config),
        Some(Commands::Sync) => handle_sync(&args, &config),
        Some(Commands::Patch) => handle_patch(&args, &config),

        Some(Commands::BuildGraph(_)) => {
            println!("BuildGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::AnalyzeGraph(_)) => {
            println!("AnalyzeGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::VisualizeGraph(_)) => {
            println!("VisualizeGraph not yet implemented in-process");
            Ok(())
        }
        Some(Commands::PartitionGraph(_)) => {
            println!("PartitionGraph not yet implemented in-process");
            Ok(())
        }

        Some(Commands::ProcessCrates(_)) => {
            println!("ProcessCrates not yet implemented in-process");
            Ok(())
        }
        Some(Commands::ProcessAll(_)) => {
            println!("ProcessAll not yet implemented in-process");
            Ok(())
        }
        Some(Commands::RunWorkflow(_)) => {
            println!("RunWorkflow not yet implemented in-process");
            Ok(())
        }

        Some(Commands::Report(_)) => {
            println!("Report not yet implemented in-process");
            Ok(())
        }

        Some(Commands::InitConfig) => {
            let sample = cargo_vendormod::config::generate_sample_config();
            println!("{}", sample);
            println!("\nCopy the above to ./vendormod.toml or ~/.config/cargo-vendormod/config.toml");
            Ok(())
        }

        Some(Commands::Onboard(_)) => {
            println!("Onboarding not yet implemented in standalone binary");
            Ok(())
        }

        Some(Commands::Edit(_)) => {
            println!("Edit not yet implemented in standalone binary");
            Ok(())
        }

        Some(Commands::Workload(args)) => {
            run_workload_analysis(&args.workspace_path, &args.format)
        }

        Some(Commands::WorkloadList) => {
            let workloads = get_defined_workloads()?;
            println!("═══════════════════════════════════════════════════════════════════");
            println!("  Available Workloads");
            println!("═══════════════════════════════════════════════════════════════════\n");
            for w in &workloads {
                println!("📦 {} ({})", w.name, w.path.split('/').last().unwrap_or(&w.name));
                println!("   {} (L1: {} crates, L2: {} crates)", 
                    w.description, w.layer1_count, w.layer2_count);
                println!();
            }
            println!("Run: cargo-vendormod workload --workspace-path /path/to/workload");
            Ok(())
        }

        Some(Commands::WorkloadWorktree(args)) => {
            let workloads = get_defined_workloads()?;
            run_workload_worktree(&args.name, args.branch.as_deref(), &args.output_dir, &workloads)
        }

        Some(Commands::NurFlake(args)) => {
            let generator = cargo_vendormod::nur_flake::NurFlakeGenerator::new(
                args.repos_json.clone(),
                args.lock_json.clone(),
                args.output.clone(),
            );

            if args.check {
                let is_up_to_date = generator
                    .check()
                    .context("Failed to check flake.nix status")?;
                if is_up_to_date {
                    println!("\u{2713} {} is up to date", args.output.display());
                } else {
                    eprintln!("\u{2717} {} is out of date \u{2014} regenerate needed", args.output.display());
                    eprintln!("  Run: cargo-vendormod nur-flake --repos-json {} --lock-json {} --output {}",
                        args.repos_json.display(),
                        args.lock_json.display(),
                        args.output.display());
                    std::process::exit(1);
                }
            } else {
                let repo_count = generator
                    .generate()
                    .context("Failed to generate flake.nix")?;
                println!("\u{2713} Generated {} with {} repos", args.output.display(), repo_count);
                if args.verbose {
                    eprintln!("  Run: nix flake check --impure  # to validate");
                }
            }

            Ok(())
        }

        Some(Commands::SplitLean4(args)) => {
            eprintln!("[info] SplitLean4: src={} out={} branch={}",
                args.mathlib_src.display(), args.output_dir.display(), args.branch);
            if args.dry_run {
                eprintln!("[dry-run] Would run: {} from {}", args.split_tool.display(), args.mathlib_src.display());
            } else {
                let status = std::process::Command::new(&args.split_tool)
                    .current_dir(&args.mathlib_src)
                    .arg(&args.output_dir)
                    .arg(&args.branch)
                    .status()?;
                if !status.success() {
                    anyhow::bail!("lean-split-tool failed with status {}", status);
                }
            }
            Ok(())
        }

        Some(Commands::Lean4(args)) => {
            eprintln!("[info] Lean4 model generation: inputs={:?} output={}", args.inputs, args.output.display());
            Ok(())
        }

        Some(Commands::FlakeCheck(args)) => {
            let report = cargo_vendormod::flake_check::check_flake_coverage(&args.flakes_dir)?;
            if args.json {
                cargo_vendormod::flake_check::print_json(&report)?;
            } else {
                cargo_vendormod::flake_check::print_report(&report);
            }
            Ok(())
        }

        Some(Commands::Ingest(args)) => {
            handle_ingest(&args, &mut config)
        }

        Some(Commands::MemecacheUpgrade(args)) => {
            handle_memecache_upgrade(&args)
        }

        Some(Commands::MemecacheGc(args)) => {
            handle_memecache_gc(&args)
        }

        Some(Commands::Split(args)) => {
            println!("Split not yet implemented: {}", args.input_file.display());
            Ok(())
        }

        Some(Commands::ScanIndex(args)) => {
            handle_scan_index(&args)
        }
        Some(Commands::ScanDelta(args)) => {
            handle_scan_delta(&args)
        }

        Some(Commands::NoraIndex(args)) => {
            run_nora_index(&args)
        }

        Some(Commands::CreateVirtualWorkspace(_)) => {
            println!("CreateVirtualWorkspace not yet implemented");
            Ok(())
        }
        Some(Commands::DetectProjects(_)) => {
            println!("DetectProjects not yet implemented");
            Ok(())
        }
        Some(Commands::AnalyzeSubmodules) => {
            println!("AnalyzeSubmodules not yet implemented");
            Ok(())
        }

        None => {
            // Show help
            println!("Cargo-vendormod - Git submodule vendoring for Cargo");
            println!("\nUsage: cargo-vendormod <command>");
            println!("\nCommands:");
            println!("  vendoring    - Git submodule operations");
            println!("  graph        - Dependency graph analysis");
            println!("  process      - Crate processing");
            println!("  workload     - Workload performance analysis");
            println!("  init-config  - Generate sample config");
            println!("  nur-flake    - Generate flake.nix for NUR workspace");
            println!("  flake-check  - Check flake coverage for CBOR libs, fuzz, tests");
            Ok(())
        }
    }
}



fn run_workload_worktree(name: &str, _branch: Option<&str>, output_dir: &PathBuf, workloads: &[WorkloadDef]) -> Result<()> {
    println!("🎯 Creating worktree for workload: {}", name);
    
    // Find the workload by name
    if let Some(workload) = workloads.iter().find(|w| w.name == name) {
        let worktree_path = output_dir.join(&workload.name);
        
        // Create output directory if it doesn't exist
        std::fs::create_dir_all(&worktree_path)?;
        
        // Create worktree using git
        let output = std::process::Command::new("git")
            .args(&["worktree", "add", worktree_path.to_str().unwrap()])
            .output()?;
        
        if output.status.success() {
            println!("✅ Worktree created at: {}", worktree_path.display());
        } else {
            eprintln!("❌ Failed to create worktree: {}", String::from_utf8_lossy(&output.stderr));
        }
    } else {
        eprintln!("❌ Unknown workload: {}", name);
        println!("Available workloads:");
        for w in workloads {
            println!("  - {}", w.name);
        }
    }
    
    Ok(())
}

fn run_workload_analysis(path: &PathBuf, format: &str) -> Result<()> {
    use cargo_vendormod::workload_processor::process_workload_recursive;
    
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Workload Performance Report");
    println!("═══════════════════════════════════════════════════════════════════\n");
    
    let metrics = process_workload_recursive(path)?;
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&metrics)?;
            println!("{}", json);
        }
        _ => {
            println!("═══════════════════════════════════════════════════════════════════");
            println!("  Performance Metrics");
            println!("═══════════════════════════════════════════════════════════════════");
            println!("Total git repositories: {}", metrics.total_repos);
            println!("Total Cargo.toml files: {}", metrics.total_cargoTOMs);
            println!("Processing time: {}ms", metrics.elapsed_ms);
            println!("Repositories per second: {:.2}", metrics.repos_per_second);
        }
    }
    Ok(())
}

// ── Vendoring handler functions (in-process) ──────────────────────────────

fn handle_init(args: &Args, config: &cargo_vendormod::config::Config, source: Option<&PathBuf>) -> Result<()> {
    let source_repo = source.cloned()
        .or_else(|| args.root_dir.canonicalize().ok())
        .unwrap_or_else(|| PathBuf::from("."));
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    cargo_vendormod::vendoring_cmds::cmd_init(&source_repo, &sub_path, &mir_path, args.dry_run)
}

fn handle_fetch_upstream(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_fetch_upstream(
        &sub_path, &mir_path, git_exe, args.dry_run, args.verbose, config.default_threads,
    )
}

fn handle_rebase(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_rebase(
        &sub_path, &config.target_branch, git_exe, args.dry_run, args.verbose,
    )
}

fn handle_releases(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_releases(
        &sub_path, &mir_path, git_exe, &config.version_branch_format,
        config.create_version_branches, args.dry_run,
    )
}

fn handle_status(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_status(&sub_path, git_exe)
}

fn handle_sync(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    let mir_path = args.mirrors_path.clone().unwrap_or_else(|| config.mirrors_dir.clone());
    let git_exe = &config.git_path;
    cargo_vendormod::vendoring_cmds::cmd_sync(
        &sub_path, &mir_path, &config.target_branch, git_exe,
        args.dry_run, args.verbose, config.default_threads,
    )
}

fn handle_patch(args: &Args, config: &cargo_vendormod::config::Config) -> Result<()> {
    let sub_path = args.submodules_path.clone().unwrap_or_else(|| config.submodules_dir.clone());
    cargo_vendormod::vendoring_cmds::cmd_patch(&sub_path)
}

// ── Ingest: read old submodules directory and take control ─────────────

fn handle_ingest(args: &IngestArgs, config: &mut Config) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Ingest: Reading old submodules");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Source:      {}", args.source_dir.display());
    println!("Output:      {}", args.output_dir.display());
    println!("Shmem:       {}", if args.shmem { "enabled" } else { "disabled" });
    println!("Dry run:     {}", args.dry_run);
    println!();

    // 1. Parse .gitmodules using git-config parser (handles git config format, not strict TOML)
    let gitmodules_path = args.gitmodules_path.clone()
        .unwrap_or_else(|| args.source_dir.join(".gitmodules"));

    if !gitmodules_path.exists() {
        anyhow::bail!(".gitmodules not found at {}", gitmodules_path.display());
    }

    let gitmodules_raw = std::fs::read_to_string(&gitmodules_path)
        .context("Failed to read .gitmodules")?;

    // Parse using git-config crate (handles submodule "path/with/slashes" correctly)
    let git_config: git_config::File = gitmodules_raw.as_str().try_into()
        .context("Failed to parse .gitmodules as git config")?;

    // Collect all submodule sections
    let mut submodule_entries: Vec<(String, String, String, Option<String>)> = Vec::new(); // (name, path, url, branch)
    for section in git_config.sections() {
        let header = section.header();
        let section_name = header.name();
        if section_name != "submodule" {
            continue;
        }
        let sub_name = header.subsection_name()
            .map(|s| s.to_string())
            .unwrap_or_default();

        let path_val = git_config.string("submodule", Some(sub_name.as_str()), "path")
            .unwrap_or_default()
            .to_string();
        let url_val = git_config.string("submodule", Some(sub_name.as_str()), "url")
            .unwrap_or_default()
            .to_string();
        let branch_val = git_config.string("submodule", Some(sub_name.as_str()), "branch")
            .map(|s| s.to_string());

        if !path_val.is_empty() && !url_val.is_empty() {
            submodule_entries.push((sub_name.clone(), path_val, url_val, branch_val));
        }
    }

    println!("Found {} registered submodules in .gitmodules", submodule_entries.len());

    // 2. Walk each submodule and collect state
    let mut registered: Vec<IngestedSubmodule> = Vec::new();
    let mut dirty_count = 0;
    let mut missing_count = 0;
    let mut clean_count = 0;
    let mut has_cargo_toml = 0;
    let mut has_flake_nix = 0;

    for (name, path_val, url_val, branch_val) in &submodule_entries {
        let sub_path = args.source_dir.join(path_val);
        let exists = sub_path.exists() && sub_path.is_dir();

        let git_status = if args.no_git_status {
            // Fast path: skip git status check, just check if dir exists
            if exists {
                clean_count += 1;
                "unknown (skipped)".to_string()
            } else {
                missing_count += 1;
                "missing".to_string()
            }
        } else if exists {
            let output = std::process::Command::new("git")
                .args(["-C", sub_path.to_str().unwrap_or(""), "status", "--porcelain"])
                .output()
                .ok();
            match output {
                Some(o) if o.status.success() => {
                    let stdout = String::from_utf8_lossy(&o.stdout);
                    if stdout.trim().is_empty() {
                        clean_count += 1;
                        "clean".to_string()
                    } else {
                        dirty_count += 1;
                        format!("dirty ({} changes)", stdout.lines().count())
                    }
                }
                _ => {
                    dirty_count += 1;
                    "unknown".to_string()
                }
            }
        } else {
            missing_count += 1;
            "missing".to_string()
        };

        let version = if exists {
            std::fs::read_to_string(sub_path.join("Cargo.toml"))
                .ok()
                .and_then(|c| c.parse::<toml::Value>().ok())
                .and_then(|doc| {
                    doc.get("package")
                        .and_then(|p| p.get("version"))
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| "unknown".to_string())
        } else {
            "N/A".to_string()
        };

        let has_cargo = exists && sub_path.join("Cargo.toml").exists();
        let has_flake = exists && sub_path.join("flake.nix").exists();

        if has_cargo { has_cargo_toml += 1; }
        if has_flake { has_flake_nix += 1; }

        let current_commit = if args.no_git_status || !exists {
            "N/A".to_string()
        } else {
            std::process::Command::new("git")
                .args(["-C", sub_path.to_str().unwrap_or(""), "rev-parse", "HEAD"])
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_else(|| "N/A".to_string())
        };

        let ingested = IngestedSubmodule {
            name: name.clone(),
            path: path_val.clone(),
            url: url_val.clone(),
            branch: branch_val.clone(),
            version,
            git_status,
            current_commit,
            has_cargo_toml: has_cargo,
            has_flake_nix: has_flake,
        };

        if args.verbose {
            println!("  {} {} ({}) [{}]", 
                if ingested.has_cargo_toml { "📦" } else { "📁" },
                ingested.name, ingested.version, ingested.git_status);
        }

        registered.push(ingested);
    }

    // 3. Report summary
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Ingest Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Total submodules:    {}", registered.len());
    println!("Clean:               {}", clean_count);
    println!("Dirty:               {}", dirty_count);
    println!("Missing:             {}", missing_count);
    println!("Have Cargo.toml:     {}", has_cargo_toml);
    println!("Have flake.nix:      {}", has_flake_nix);
    println!();

    // 4. Write vendormod-registry.json
    if !args.dry_run {
        std::fs::create_dir_all(&args.output_dir)
            .context("Failed to create output directory")?;

        let registry = serde_json::json!({
            "version": "0.2.0",
            "source_dir": args.source_dir.to_string_lossy().to_string(),
            "total_submodules": registered.len(),
            "clean_count": clean_count,
            "dirty_count": dirty_count,
            "missing_count": missing_count,
            "has_cargo_toml": has_cargo_toml,
            "has_flake_nix": has_flake_nix,
            "submodules": registered,
        });

        let registry_path = args.output_dir.join("vendormod-registry.json");
        let json = serde_json::to_string_pretty(&registry)?;
        std::fs::write(&registry_path, &json)?;
        println!("Written registry to {}", registry_path.display());

        // 5. Write vendormod.lock (CID-indexed)
        let lock_path = args.output_dir.join("vendormod.lock");
        let mut lock_content = String::new();
        lock_content.push_str("# vendormod.lock — CID-indexed submodule registry\n");
        lock_content.push_str("# Generated by cargo-vendormod ingest\n");
        lock_content.push_str(&format!("# Total: {} submodules\n\n", registered.len()));

        for sm in &registered {
            let cid = compute_cid(&format!("{}:{}:{}", sm.url, sm.branch.as_deref().unwrap_or("main"), sm.current_commit));
            lock_content.push_str(&format!("{} {} {} {} {}\n",
                cid, sm.name, sm.version, sm.git_status, sm.url));
        }

        std::fs::write(&lock_path, &lock_content)?;
        println!("Written lock file to {}", lock_path.display());
    }

    // 6. Store in shmem if requested
    if args.shmem && !args.dry_run {
        println!();
        println!("Storing submodule metadata in shmem...");
        for sm in &registered {
            if sm.has_cargo_toml {
                let path = format!("vendormod/submodules/{}/metadata", sm.name);
                let description = format!("{} {} ({} bytes)", sm.name, sm.version, 0);
                // Store the Cargo.toml content in shmem
                let cargo_toml_path = args.source_dir.join(&sm.path).join("Cargo.toml");
                if let Ok(content) = std::fs::read(&cargo_toml_path) {
                    let _ = shmem_put(&args.shmem_socket, &path, &description, &content);
                    if args.verbose {
                        println!("  Stored {} ({} bytes)", sm.name, content.len());
                    }
                }
            }
        }
    }

    // 7. Update warm_dirs config
    let now = chrono_now();
    for dir in &mut config.warm_dirs {
        if dir.path == args.source_dir {
            dir.last_warmed = Some(now.clone());
        }
    }
    config.save()?;
    if args.verbose {
        println!("[info] Updated warm status for {}", args.source_dir.display());
    }

    println!();
    println!("Ingest complete. {} submodules registered.", registered.len());
    Ok(())
}

// ── Stub functions for yet-to-be-implemented commands ──────────────────

fn get_defined_workloads() -> Result<Vec<WorkloadDef>> {
    // TODO: Implement workload loading from vendormod.toml or default set
    Ok(Vec::new())
}

fn handle_scan_delta(_args: &ScanDeltaArgs) -> Result<()> {
    // TODO: Implement scan delta using IPLD shmem snapshots
    println!("ScanDelta not yet implemented");
    Ok(())
}
