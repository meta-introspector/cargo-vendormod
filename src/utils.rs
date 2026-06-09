use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};
use std::io::Write;
use serde::{Serialize, Deserialize};

// ── chrono_now (existing) ─────────────────────────────────────────────

pub fn chrono_now() -> String {
    let output = std::process::Command::new("date")
        .args(["+%Y-%m-%dT%H:%M:%S"])
        .output()
        .ok();
    match output {
        Some(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).trim().to_string(),
        _ => format!("{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs()),
    }
}

// ── compute_cid ────────────────────────────────────────────────────────

/// Compute a simple CID-like hash for a string
pub fn compute_cid(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let hash = hasher.finalize();
    format!("bafyri{}", hex::encode(hash)[..56].to_string())
}

// ── shmem_put ──────────────────────────────────────────────────────────

/// Max file size to store in shmem (1MB)
pub const SHMEM_MAX_SIZE: usize = 1_048_576;

/// Full path to letta-ipld-memory binary
pub const IPLD_MEMORY_BIN: &str = "/home/mdupont/dasl/ipld-car-ipc-shmem-linux/target/release/letta-ipld-memory";

/// Store a block in the IPLD CAR shmem
pub fn shmem_put(_socket: &str, path: &str, description: &str, data: &[u8]) -> Result<()> {
    // Enforce 1MB cap
    if data.len() > SHMEM_MAX_SIZE {
        eprintln!("[shmem] Skipping {} ({} bytes > 1MB cap)", path, data.len());
        return Ok(());
    }

    // Call letta-ipld-memory put via stdin pipe
    let mut child = std::process::Command::new(IPLD_MEMORY_BIN)
        .arg("put")
        .arg(path)
        .arg(description)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .context("Failed to spawn letta-ipld-memory put")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(data)?;
    }

    let output = child.wait_with_output()?;
    if !output.status.success() {
        eprintln!("[shmem] Failed to store {}: {}", path, String::from_utf8_lossy(&output.stderr));
    } else {
        let cid = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !cid.is_empty() {
            eprintln!("[shmem] Stored {} ({} bytes) -> CID {}", path, data.len(), cid);
        }
    }
    Ok(())
}

// ============================================================
// File Sampler — head, tail, middle, conformal field
// ============================================================

/// Default sample sizes
pub const SAMPLE_HEAD_LINES: usize = 100;
pub const SAMPLE_TAIL_LINES: usize = 100;
pub const SAMPLE_MIDDLE_LINES: usize = 100;
pub const SAMPLE_CONFORMAL_LINES: usize = 200;

/// Sample lines from a large file for storage under the 1MB cap.
/// Strategy: head + tail + middle + conformal (evenly-spaced intervals)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileSample {
    /// Original file path
    pub path: String,
    /// Original file size in bytes
    pub original_size: u64,
    /// Original line count
    pub original_lines: usize,
    /// Number of head lines sampled
    pub head_lines: usize,
    /// Number of tail lines sampled
    pub tail_lines: usize,
    /// Number of middle lines sampled
    pub middle_lines: usize,
    /// Number of conformal lines sampled (evenly-spaced)
    pub conformal_lines: usize,
    /// Total lines in this sample
    pub sample_lines: usize,
    /// Sample size in bytes
    pub sample_size: usize,
    /// Byte-level entropy of the sample
    pub entropy: f64,
    /// Hecke spectral score (simplified)
    pub hecke_score: f64,
    /// Detected CIDs in sample (count)
    pub detected_cids: usize,
    /// The sampled content
    pub content: String,
}

/// Sample a large file: head, tail, middle, conformal field
pub fn sample_file(path: &Path, max_sample_bytes: usize) -> Result<FileSample> {
    let metadata = std::fs::metadata(path)?;
    let original_size = metadata.len();

    // Read the file with lossy UTF-8
    let raw = std::fs::read(path)?;
    let content = String::from_utf8_lossy(&raw);
    let all_lines: Vec<&str> = content.lines().collect();
    let original_lines = all_lines.len();

    if original_lines == 0 {
        return Ok(FileSample {
            path: path.to_string_lossy().to_string(),
            original_size,
            original_lines: 0,
            head_lines: 0,
            tail_lines: 0,
            middle_lines: 0,
            conformal_lines: 0,
            sample_lines: 0,
            sample_size: 0,
            entropy: 0.0,
            hecke_score: 0.0,
            detected_cids: 0,
            content: String::new(),
        });
    }

    let mut sampled: Vec<(usize, &str)> = Vec::new(); // (line_number, line_text)
    let mut seen_lines: HashSet<usize> = HashSet::new();

    // 1. Head
    let head_n = SAMPLE_HEAD_LINES.min(original_lines);
    for i in 0..head_n {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 2. Tail
    let tail_n = SAMPLE_TAIL_LINES.min(original_lines);
    let tail_start = original_lines.saturating_sub(tail_n);
    for i in tail_start..original_lines {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 3. Middle (lines around the midpoint)
    let mid = original_lines / 2;
    let middle_n = SAMPLE_MIDDLE_LINES.min(original_lines);
    let middle_start = mid.saturating_sub(middle_n / 2);
    let middle_end = (mid + middle_n / 2).min(original_lines);
    for i in middle_start..middle_end {
        if !seen_lines.contains(&i) {
            sampled.push((i, all_lines[i]));
            seen_lines.insert(i);
        }
    }

    // 4. Conformal field — evenly-spaced lines across the entire file
    //    This is the "deep_scanner conformal" sampling: uniform intervals
    //    that preserve the file's spectral structure
    let conformal_n = SAMPLE_CONFORMAL_LINES.min(original_lines);
    if original_lines > 1 && conformal_n > 0 {
        let step = original_lines as f64 / conformal_n as f64;
        for k in 0..conformal_n {
            let i = (k as f64 * step) as usize;
            let i = i.min(original_lines - 1);
            if !seen_lines.contains(&i) {
                sampled.push((i, all_lines[i]));
                seen_lines.insert(i);
            }
        }
    }

    // Sort by line number for readability
    sampled.sort_by_key(|(i, _)| *i);

    // Build the sample content with line number markers
    let mut sample_content = String::new();
    sample_content.push_str(&format!("# FileSample: {} ({} bytes, {} lines)\n",
        path.to_string_lossy(), original_size, original_lines));
    sample_content.push_str(&format!("# head={} tail={} middle={} conformal={}\n",
        head_n, tail_n, middle_n, conformal_n));
    sample_content.push_str("# ---\n");

    for (line_num, line) in &sampled {
        // Truncate lines > 500 chars to keep sample small
        let truncated = if line.len() > 500 {
            format!("{}...[truncated, {} chars]", &line[..500], line.len())
        } else {
            line.to_string()
        };
        sample_content.push_str(&format!("{}:{}\n", line_num + 1, truncated));
    }

    // Trim to max_sample_bytes
    if sample_content.len() > max_sample_bytes {
        sample_content.truncate(max_sample_bytes);
        // Find last complete line
        if let Some(pos) = sample_content.rfind('\n') {
            sample_content.truncate(pos + 1);
        }
        sample_content.push_str("# [SAMPLE TRUNCATED TO FIT CAP]\n");
    }

    let sample_size = sample_content.len();
    let sample_lines_actual = sampled.len();

    // Compute entropy of the sample
    let entropy = compute_entropy(sample_content.as_bytes());

    // Simplified Hecke score: spectral density of line lengths
    let hecke_score = compute_hecke_score(&sampled);

    // Count potential CIDs (bafyrei, bafkrei, Qm prefixes)
    let detected_cids = count_cids(&sample_content);

    Ok(FileSample {
        path: path.to_string_lossy().to_string(),
        original_size,
        original_lines,
        head_lines: head_n,
        tail_lines: tail_n,
        middle_lines: middle_n,
        conformal_lines: conformal_n,
        sample_lines: sample_lines_actual,
        sample_size,
        entropy,
        hecke_score,
        detected_cids,
        content: sample_content,
    })
}

/// Compute Shannon entropy of a byte slice
pub fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }
    let mut freq = [0usize; 256];
    for &b in data {
        freq[b as usize] += 1;
    }
    let total = data.len() as f64;
    let mut entropy = 0.0;
    for &count in &freq {
        if count > 0 {
            let p = count as f64 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

/// Simplified Hecke spectral score: variance of line length differences
/// (captures structural rhythm of the file)
pub fn compute_hecke_score(lines: &[(usize, &str)]) -> f64 {
    if lines.len() < 3 {
        return 0.0;
    }
    let lengths: Vec<usize> = lines.iter().map(|(_, l)| l.len()).collect();
    let diffs: Vec<f64> = lengths.windows(2).map(|w| (w[1] as f64 - w[0] as f64).abs()).collect();
    if diffs.is_empty() {
        return 0.0;
    }
    let mean = diffs.iter().sum::<f64>() / diffs.len() as f64;
    let variance = diffs.iter().map(|d| (d - mean).powi(2)).sum::<f64>() / diffs.len() as f64;
    // Hecke-like spectral score: sqrt(variance) * ln(lines)
    variance.sqrt() * (lines.len() as f64).ln().max(1.0)
}

/// Count potential CIDs in text (bafyrei, bafkrei, Qm prefixes)
pub fn count_cids(text: &str) -> usize {
    let mut count = 0;
    for line in text.lines() {
        if line.contains("bafyrei") || line.contains("bafkrei") || line.contains("Qm") {
            count += 1;
        }
    }
    count
}

// ── IngestedSubmodule ──────────────────────────────────────────────────

/// Ingested submodule record
#[derive(Debug, Clone, serde::Serialize)]
pub struct IngestedSubmodule {
    pub name: String,
    pub path: String,
    pub url: String,
    pub branch: Option<String>,
    pub version: String,
    pub git_status: String,
    pub current_commit: String,
    pub has_cargo_toml: bool,
    pub has_flake_nix: bool,
}

// ── Memecache Upgrade ──────────────────────────────────────────────────

/// Memecache upgrade: force-upgrade all crates
pub fn handle_memecache_upgrade(args: &crate::args::MemecacheUpgradeArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Memecache Upgrade");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();
    println!("Workspace:   {}", args.workspace_path.display());
    println!("Strategy:    {}", args.strategy);
    println!("Dry run:     {}", args.dry_run);
    println!("All:         {}", args.all);
    println!();

    // Validate strategy
    if !["conservative", "aggressive", "force"].contains(&args.strategy.as_str()) {
        anyhow::bail!("Invalid strategy '{}'. Must be: conservative, aggressive, or force", args.strategy);
    }

    // Find all Cargo.toml files
    let mut cargo_tomls = Vec::new();
    if args.all {
        for entry in walkdir::WalkDir::new(&args.workspace_path)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name() == Some(std::ffi::OsStr::new("Cargo.toml"))
                && !path.to_string_lossy().contains("/target/")
                && !path.to_string_lossy().contains("/.cargo/")
            {
                cargo_tomls.push(path.to_path_buf());
            }
        }
    } else {
        cargo_tomls.push(args.workspace_path.join("Cargo.toml"));
    }

    println!("Found {} Cargo.toml files to upgrade", cargo_tomls.len());

    // For each workspace, find dependencies and check for upgrades
    let mut total_upgrades = 0;
    let mut total_skipped = 0;
    let mut total_errors = 0;

    for cargo_toml in &cargo_tomls {
        if args.verbose {
            println!("  Scanning: {}", cargo_toml.display());
        }

        let content = match std::fs::read_to_string(cargo_toml) {
            Ok(c) => c,
            Err(_) => { total_errors += 1; continue; }
        };

        let doc: toml::Value = match content.parse() {
            Ok(d) => d,
            Err(_) => { total_errors += 1; continue; }
        };

        // Count dependencies
        let dep_count = count_dependencies(&doc);
        if args.verbose {
            println!("    {} dependencies", dep_count);
        }

        // For now, just report. The actual upgrade logic will call
        // cargo's upgrade_requirement function once we integrate with
        // the cargo source.
        if args.dry_run {
            total_skipped += dep_count;
        } else {
            // TODO: implement actual upgrade using cargo's upgrade_requirement
            total_skipped += dep_count;
        }
    }

    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Upgrade Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Workspaces scanned:  {}", cargo_tomls.len());
    println!("Upgrades available:  {}", total_upgrades);
    println!("Skipped:             {}", total_skipped);
    println!("Errors:              {}", total_errors);
    println!();

    if args.dry_run {
        println!("Dry run — no changes made.");
    }

    Ok(())
}

pub fn count_dependencies(doc: &toml::Value) -> usize {
    let mut count = 0;
    for section in &["dependencies", "dev-dependencies", "build-dependencies"] {
        if let Some(deps) = doc.get(section).and_then(|d| d.as_table()) {
            count += deps.len();
        }
    }
    count
}

// ── Memecache GC ───────────────────────────────────────────────────────

/// Memecache GC: clean stale crate versions
pub fn handle_memecache_gc(args: &crate::args::MemecacheGcArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Memecache GC");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let registry_path = if args.registry_path.starts_with("~") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(args.registry_path.to_string_lossy().replacen("~", &home, 1))
    } else {
        args.registry_path.clone()
    };

    println!("Registry:    {}", registry_path.display());
    println!("Dry run:     {}", args.dry_run);
    println!("Clean git:   {}", args.clean_git);
    println!("Clean targets: {}", args.clean_targets);
    println!();

    // Walk the registry src directory
    let src_path = registry_path.join("src");
    if !src_path.exists() {
        anyhow::bail!("Registry src directory not found at {}", src_path.display());
    }

    // Group crate versions
    let mut crate_versions: HashMap<String, Vec<(String, u64)>> = HashMap::new();

    for entry in walkdir::WalkDir::new(&src_path)
        .max_depth(3)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");

            // Parse crate-version directory name
            if let Some((crate_name, version)) = parse_crate_version(dir_name) {
                let size = dir_size(path);
                crate_versions
                    .entry(crate_name)
                    .or_default()
                    .push((version, size));
            }
        }
    }

    // Find stale versions (keep only latest per semver range)
    let mut stale_count = 0;
    let mut stale_bytes: u64 = 0;
    let mut kept_count = 0;
    let mut kept_bytes: u64 = 0;

    let mut crate_names: Vec<&String> = crate_versions.keys().collect();
    crate_names.sort();

    for crate_name in &crate_names {
        let versions = crate_versions.get(crate_name.as_str()).unwrap();
        if versions.len() <= 1 {
            kept_count += versions.len();
            kept_bytes += versions.iter().map(|(_, s)| *s).sum::<u64>();
            continue;
        }

        // Find the latest version
        let mut sorted = versions.clone();
        sorted.sort_by(|a, b| a.0.cmp(&b.0)); // string sort is close enough for semver
        let latest = sorted.last().unwrap().clone();

        if args.verbose {
            if versions.len() > 3 {
                println!("  {} has {} versions (latest: {})", crate_name, versions.len(), latest.0);
            }
        }

        for (version, size) in versions {
            if version == &latest.0 {
                kept_count += 1;
                kept_bytes += size;
            } else {
                stale_count += 1;
                stale_bytes += size;
            }
        }
    }

    // Report
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  GC Summary");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Unique crates:       {}", crate_versions.len());
    println!("Total versions:      {}", crate_versions.values().map(|v| v.len()).sum::<usize>());
    println!("Versions to keep:    {} ({:.1} MB)", kept_count, kept_bytes as f64 / 1_048_576.0);
    println!("Versions to delete:  {} ({:.1} MB)", stale_count, stale_bytes as f64 / 1_048_576.0);
    println!();

    if args.dry_run {
        println!("Dry run — no changes made.");
    } else if stale_count > 0 {
        println!("Would delete {} stale versions ({:.1} MB) — not yet implemented", 
            stale_count, stale_bytes as f64 / 1_048_576.0);
        println!("Use --dry-run to see what would be deleted.");
    }

    Ok(())
}

/// Parse a crate-version directory name like "serde-1.0.228"
pub fn parse_crate_version(dir_name: &str) -> Option<(String, String)> {
    // Find the last '-' that's followed by a digit (crate-version pattern)
    for (i, c) in dir_name.char_indices().rev() {
        if c == '-' {
            let rest = &dir_name[i+1..];
            if rest.starts_with(|c: char| c.is_ascii_digit()) {
                return Some((dir_name[..i].to_string(), rest.to_string()));
            }
        }
    }
    None
}

/// Get directory size in bytes
pub fn dir_size(path: &Path) -> u64 {
    walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter_map(|e| e.metadata().ok())
        .filter(|m| m.is_file())
        .map(|m| m.len())
        .sum()
}

// ── Scan Index ─────────────────────────────────────────────────────────

/// A scanned file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScannedFile {
    pub path: String,
    pub source: String,
    pub exists: bool,
    pub size: u64,
    pub ext: String,
}

/// A scanned .gitmodules entry
#[derive(Debug, Clone, serde::Serialize)]
pub struct ScannedGitmodule {
    pub gitmodules_path: String,
    pub submodule_name: String,
    pub submodule_path: String,
    pub submodule_url: String,
    pub submodule_branch: Option<String>,
}

/// Handle the scan-index command
pub fn handle_scan_index(args: &crate::args::ScanIndexArgs) -> Result<()> {
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Cargo-Vendormod Scan Index");
    println!("═══════════════════════════════════════════════════════════════════");
    println!();

    let mut all_files: Vec<ScannedFile> = Vec::new();
    let mut all_gitmodules: Vec<ScannedGitmodule> = Vec::new();
    let mut source_stats: HashMap<String, usize> = HashMap::new();

    // 1. Read text file lists (--file-list)
    for file_list_path in &args.file_list {
        if args.verbose {
            println!("Reading file list: {}", file_list_path.display());
        }
        let count = read_file_list(file_list_path, &args.base_dir, &args.ext, &mut all_files, &args.max_lines)?;
        source_stats.insert(format!("file_list:{}", file_list_path.display()), count);
    }

    // 2. Read index directories (--index-dir)
    for index_dir in &args.index_dir {
        if args.verbose {
            println!("Scanning index directory: {}", index_dir.display());
        }
        let mut dir_count = 0usize;
        if index_dir.exists() {
            for entry in std::fs::read_dir(index_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_file() {
                    let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    // Sample large files instead of skipping (when --sample is set)
                    let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                    if size > SHMEM_MAX_SIZE as u64 {
                        if args.sample {
                            if args.verbose {
                                println!("  Sampling large file: {} ({:.1} MB)", name, size as f64 / 1_048_576.0);
                            }
                            // Create a sample and store it in shmem
                            match sample_file(&path, SHMEM_MAX_SIZE) {
                                Ok(sample) => {
                                    if args.verbose {
                                        println!("    Sampled: {}/{} lines, {} bytes, entropy={:.2}, hecke={:.2}, cids={}",
                                            sample.sample_lines, sample.original_lines,
                                            sample.sample_size, sample.entropy, sample.hecke_score, sample.detected_cids);
                                    }
                                    // Store the sample in shmem
                                    if args.shmem {
                                        let sample_json = serde_json::to_vec(&sample)?;
                                        let shmem_path = format!("vendormod/samples/{}", name);
                                        let _ = shmem_put("", &shmem_path, &format!("Sample of {} ({} bytes)", name, sample.original_size), &sample_json);
                                    }
                                    // Record the sample as a scanned file entry
                                    all_files.push(ScannedFile {
                                        path: path.to_string_lossy().to_string(),
                                        source: format!("index_dir:{}", index_dir.display()),
                                        exists: true,
                                        size: sample.original_size,
                                        ext: path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string(),
                                    });
                                }
                                Err(e) => {
                                    if args.verbose {
                                        eprintln!("  Warning: Failed to sample {}: {}", name, e);
                                    }
                                }
                            }
                            dir_count += 1;
                            continue;
                        } else {
                            if args.verbose {
                                println!("  Skipping large file: {} ({:.1} MB, use --sample to sample)", name, size as f64 / 1_048_576.0);
                            }
                            continue;
                        }
                    }
                    // Skip binary/non-text files
                    if name.ends_with(".cbor") || name.ends_with(".car") || name.ends_with(".parquet") {
                        continue;
                    }
                    // Skip backup/temp files
                    if name.ends_with('~') || name.ends_with(".bak") || name.starts_with('#') || name.starts_with(".#") {
                        continue;
                    }
                    let count = match read_file_list(&path, &args.base_dir, &args.ext, &mut all_files, &args.max_lines) {
                        Ok(c) => c,
                        Err(e) => {
                            if args.verbose {
                                eprintln!("  Warning: Failed to read {}: {}", name, e);
                            }
                            0
                        }
                    };
                    dir_count += count;
                }
            }
        }
        source_stats.insert(format!("index_dir:{}", index_dir.display()), dir_count);
    }

    // 3. Read .gitmodules files (--gitmodules)
    for gm_path in &args.gitmodules {
        if args.verbose {
            println!("Parsing .gitmodules: {}", gm_path.display());
        }
        let count = read_gitmodules_file(gm_path, &mut all_gitmodules)?;
        source_stats.insert(format!("gitmodules:{}", gm_path.display()), count);
    }

    // 4. Find .gitmodules via plocate (--plocate-pattern)
    if let Some(pattern) = &args.plocate_pattern {
        if args.verbose {
            println!("Searching plocate for: {}", pattern);
        }
        let count = plocate_gitmodules(pattern, &mut all_gitmodules)?;
        source_stats.insert("plocate".to_string(), count);
    }

    // 5. Read parquet files (--parquet) - shell out to python3
    for pq_path in &args.parquet {
        if args.verbose {
            println!("Reading parquet: {}", pq_path.display());
        }
        let count = read_parquet_index(pq_path, &args.base_dir, &args.ext, &mut all_files)?;
        source_stats.insert(format!("parquet:{}", pq_path.display()), count);
    }

    // 6. Deduplicate files by path
    let mut seen_paths: HashSet<String> = HashSet::new();
    all_files.retain(|f| seen_paths.insert(f.path.clone()));

    // 7. Report
    println!();
    println!("═══════════════════════════════════════════════════════════════════");
    println!("  Scan Results");
    println!("═══════════════════════════════════════════════════════════════════");
    println!("Unique files found:     {}", all_files.len());
    println!("Gitmodules entries:     {}", all_gitmodules.len());
    println!();

    // Source breakdown
    println!("Sources:");
    for (source, count) in &source_stats {
        println!("  {}: {} entries", source, count);
    }

    // Extension breakdown
    let mut ext_counts: HashMap<String, usize> = HashMap::new();
    for f in &all_files {
        *ext_counts.entry(f.ext.clone()).or_default() += 1;
    }
    let mut ext_sorted: Vec<_> = ext_counts.iter().collect();
    ext_sorted.sort_by(|a, b| b.1.cmp(a.1));
    println!();
    println!("Top extensions:");
    for (ext, count) in ext_sorted.iter().take(20) {
        println!("  .{}: {} files", ext, count);
    }

    // Existence check
    let existing = all_files.iter().filter(|f| f.exists).count();
    let missing = all_files.len() - existing;
    println!();
    println!("Existing files: {}", existing);
    println!("Missing files:  {}", missing);

    // 8. Write output
    if !args.dry_run {
        std::fs::create_dir_all(&args.output_dir)?;

        match args.format.as_str() {
            "json" => {
                let output = serde_json::json!({
                    "total_files": all_files.len(),
                    "total_gitmodules": all_gitmodules.len(),
                    "sources": source_stats,
                    "extension_counts": ext_counts,
                    "files": all_files,
                    "gitmodules": all_gitmodules,
                });
                let json_path = args.output_dir.join("scan-results.json");
                std::fs::write(&json_path, serde_json::to_string_pretty(&output)?)?;
                println!("Written JSON to {}", json_path.display());
            }
            "text" => {
                let txt_path = args.output_dir.join("scan-results.txt");
                let mut out = String::new();
                for f in &all_files {
                    out.push_str(&format!("{} {} {} {}\n", f.path, f.source, f.exists, f.size));
                }
                std::fs::write(&txt_path, &out)?;
                println!("Written text to {}", txt_path.display());
            }
            _ => {
                // summary only — already printed above
                let summary_path = args.output_dir.join("scan-summary.json");
                let summary = serde_json::json!({
                    "total_files": all_files.len(),
                    "total_gitmodules": all_gitmodules.len(),
                    "sources": source_stats,
                    "extension_counts": ext_counts,
                    "existing": existing,
                    "missing": missing,
                });
                std::fs::write(&summary_path, serde_json::to_string_pretty(&summary)?)?;
                println!("Written summary to {}", summary_path.display());
            }
        }
    }

    // 9. Store in shmem if requested (1MB cap per block)
    if args.shmem {
        // Store summary
        let summary = serde_json::json!({
            "total_files": all_files.len(),
            "total_gitmodules": all_gitmodules.len(),
            "sources": source_stats,
            "existing": existing,
            "missing": missing,
        });
        let summary_bytes = serde_json::to_vec(&summary)?;
        let _ = shmem_put("", "vendormod/scan-summary", "Scan index summary", &summary_bytes);

        // Store gitmodules entries (batch as one if under 1MB)
        let gm_bytes = serde_json::to_vec(&all_gitmodules)?;
        let _ = shmem_put("", "vendormod/scan-gitmodules", &format!("{} gitmodules entries", all_gitmodules.len()), &gm_bytes);

        // Store file entries in chunks of 1000 (to stay under 1MB)
        for (chunk_idx, chunk) in all_files.chunks(1000).enumerate() {
            let chunk_bytes = serde_json::to_vec(chunk)?;
            let _ = shmem_put("", &format!("vendormod/scan-files-{}", chunk_idx), &format!("Files chunk {} ({} entries)", chunk_idx, chunk.len()), &chunk_bytes);
        }

        println!("Stored results in IPLD shmem (1MB cap per block)");
    }

    Ok(())
}

/// Read a text file list (one path per line)
pub fn read_file_list(
    file_path: &Path,
    base_dir: &Path,
    ext_filter: &[String],
    results: &mut Vec<ScannedFile>,
    max_lines: &usize,
) -> Result<usize> {
    let content = match std::fs::read_to_string(file_path) {
        Ok(c) => c,
        Err(_) => {
            // Try reading as bytes and lossy-converting for non-UTF8 files
            let bytes = std::fs::read(file_path)
                .with_context(|| format!("Failed to read {}", file_path.display()))?;
            String::from_utf8_lossy(&bytes).to_string()
        }
    };
    let source_name = file_path.file_name().and_then(|n| n.to_str()).unwrap_or("unknown").to_string();
    let mut count = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        // Apply max_lines limit
        if *max_lines > 0 && count >= *max_lines {
            break;
        }

        // Skip URLs (https://, http://, git@, ssh://)
        if line.starts_with("https://") || line.starts_with("http://")
            || line.starts_with("git@") || line.starts_with("ssh://")
        {
            // Record as URL reference, not a file path
            results.push(ScannedFile {
                path: line.to_string(),
                source: source_name.clone(),
                exists: false,
                size: 0,
                ext: "url".to_string(),
            });
            count += 1;
            continue;
        }

        // Strip trailing colons (directory indicators from org-mode/git)
        let cleaned = line.trim_end_matches(':');

        // Expand ~ to home directory
        let expanded = if cleaned.starts_with("~/") {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/mdupont".to_string());
            cleaned.replacen("~", &home, 1)
        } else {
            cleaned.to_string()
        };

        // Resolve path (absolute or relative to base_dir)
        let resolved = if expanded.starts_with('/') {
            PathBuf::from(&expanded)
        } else {
            base_dir.join(&expanded)
        };

        let ext = resolved.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_string();

        // Apply extension filter
        if !ext_filter.is_empty() && !ext_filter.iter().any(|e| e.trim_start_matches('.') == ext) {
            continue;
        }

        let exists = resolved.exists();
        let size = if exists {
            resolved.metadata().map(|m| m.len()).unwrap_or(0)
        } else {
            0
        };

        results.push(ScannedFile {
            path: resolved.to_string_lossy().to_string(),
            source: source_name.clone(),
            exists,
            size,
            ext,
        });

        count += 1;
    }

    Ok(count)
}

/// Read a .gitmodules file and extract submodule entries
pub fn read_gitmodules_file(
    gitmodules_path: &Path,
    results: &mut Vec<ScannedGitmodule>,
) -> Result<usize> {
    let content = std::fs::read_to_string(gitmodules_path)
        .with_context(|| format!("Failed to read {}", gitmodules_path.display()))?;

    let git_config: git_config::File = content.as_str().try_into()
        .context("Failed to parse .gitmodules")?;

    let gm_path_str = gitmodules_path.to_string_lossy().to_string();
    let mut count = 0;

    for section in git_config.sections() {
        let header = section.header();
        if header.name() != "submodule" {
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

        if !path_val.is_empty() {
            results.push(ScannedGitmodule {
                gitmodules_path: gm_path_str.clone(),
                submodule_name: sub_name,
                submodule_path: path_val,
                submodule_url: url_val,
                submodule_branch: branch_val,
            });
            count += 1;
        }
    }

    Ok(count)
}

/// Find .gitmodules via plocate and parse them
pub fn plocate_gitmodules(
    pattern: &str,
    results: &mut Vec<ScannedGitmodule>,
) -> Result<usize> {
    let output = std::process::Command::new("plocate")
        .args(["-l", "500", pattern])
        .output()
        .context("Failed to run plocate (is it installed?)")?;

    if !output.status.success() {
        eprintln!("plocate failed: {}", String::from_utf8_lossy(&output.stderr));
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut total = 0;

    for line in stdout.lines() {
        let line = line.trim();
        if line.is_empty() || !line.ends_with(".gitmodules") {
            continue;
        }
        let gm_path = PathBuf::from(line);
        if !gm_path.exists() {
            continue;
        }
        match read_gitmodules_file(&gm_path, results) {
            Ok(count) => total += count,
            Err(e) => {
                eprintln!("  Warning: Failed to parse {}: {}", gm_path.display(), e);
            }
        }
    }

    Ok(total)
}

/// Read a parquet file as a file index (shell out to python3 + pyarrow)
pub fn read_parquet_index(
    parquet_path: &Path,
    base_dir: &Path,
    ext_filter: &[String],
    results: &mut Vec<ScannedFile>,
) -> Result<usize> {
    let pq_str = parquet_path.to_string_lossy().to_string();
    let base_str = base_dir.to_string_lossy().to_string();
    let ext_filter_json = serde_json::to_string(ext_filter)?;

    // Python script to read parquet and output paths as JSON
    let python_script = r#"
import pyarrow.parquet as pq
import json
import sys
import os

pq_path = sys.argv[1]
base_dir = sys.argv[2]
ext_filter = json.loads(sys.argv[3])

t = pq.read_table(pq_path)
df = t.to_pandas()

# Find path columns
path_col = None
for col in df.columns:
    if col.lower() in ('path', 'file_path', 'filepath', 'filename'):
        path_col = col
        break

if path_col is None:
    # Try first string column
    for col in df.columns:
        if df[col].dtype == object:
            path_col = col
            break

if path_col is None:
    print(json.dumps([]))
    sys.exit(0)

paths = df[path_col].dropna().tolist()
if ext_filter:
    exts = set(e.lstrip('.') for e in ext_filter)
    paths = [p for p in paths if isinstance(p, str) and os.path.splitext(p)[1].lstrip('.') in exts]

output = []
for p in paths:
    if not isinstance(p, str):
        continue
    resolved = p if p.startswith('/') else os.path.join(base_dir, p)
    exists = os.path.exists(resolved)
    size = os.path.getsize(resolved) if exists else 0
    ext = os.path.splitext(resolved)[1].lstrip('.')
    output.append({
        "path": resolved,
        "source": os.path.basename(pq_path),
        "exists": exists,
        "size": size,
        "ext": ext
    })

print(json.dumps(output))
"#;

    let output = std::process::Command::new("python3")
        .arg("-c")
        .arg(python_script)
        .arg(&pq_str)
        .arg(&base_str)
        .arg(&ext_filter_json)
        .output()
        .context("Failed to run python3 for parquet reading")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("Warning: python3 parquet reader failed: {}", stderr);
        return Ok(0);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let files: Vec<ScannedFile> = serde_json::from_str(&stdout).unwrap_or_default();
    let count = files.len();
    results.extend(files);

    Ok(count)
}
