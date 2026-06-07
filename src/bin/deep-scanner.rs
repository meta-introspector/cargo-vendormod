//! # Deep Scanner for Cached CAR Data
//!
//! Applies deep sampling (head/tail/middle/conformal) to content retrieved from the shmem cache
//! via blocks.json index, and stores results as CAR pages.

use anyhow::Result;
use clap::Parser;
use ipld_car_ipc_shmem_linux::car_store::CarStore;
use std::collections::HashSet;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = DeepScanArgs::parse();
    
    println!("Cache dir: {:?}", args.cache_dir);
    
    let mut store = CarStore::open(&args.cache_dir, 36 * 1024 * 1024 * 1024)?;
    println!("Opened CarStore");
    
    let blocks = store.list_blocks();
    println!("Found {} entries in block index", blocks.len());
    
    let mut scanned_count = 0;
    for entry in &blocks {
        let path = &entry.path;
        
        // Only scan Rust source files
        if !path.ends_with(".rs") {
            continue;
        }
        
        if scanned_count >= args.max_entries {
            break;
        }
        
        println!("Scanning: {}", path);
        
        // Get content from cache
        let content = match store.get_block(path) {
            Some(c) => c,
            None => {
                println!("  (content not available in cache)");
                continue;
            }
        };
        
        let content_str = String::from_utf8_lossy(&content);
        
        if content_str.len() <= 10_000 {
            println!("  (file too small for sampling: {} bytes)", content_str.len());
            continue;
        }
        
        // Sample the content
        let sample = sample_large_content(&content_str);
        
        // Store the sample back into cache
        let sample_path = format!("deep_scanner/{}", path.replace("/", "_"));
        let sample_bytes = serde_json::to_vec(&sample)?;
        let cid = store.put_block(
            &sample_path,
            "Deep sampled content",
            true,
            &sample_bytes,
        )?;
        
        println!("  Sample stored as CID: {}...", &cid[..8]);
        scanned_count += 1;
    }
    
    let stats = store.stats();
    println!("\nDeep scan complete. Scanned {} entries.", scanned_count);

    
    Ok(())
}

#[derive(Parser, Debug)]
pub struct DeepScanArgs {
    /// Cache directory
    #[arg(long, short, default_value = "/mnt/data1/dasl-cache")]
    pub cache_dir: PathBuf,
    
    /// Max entries to scan
    #[arg(long, short, default_value = "100")]
    pub max_entries: usize,
}



fn sample_large_content(content: &str) -> serde_json::Value {
    let lines: Vec<&str> = content.lines().collect();
    let original_lines = lines.len();
    
    let head_n = 100.min(original_lines);
    let tail_n = 100.min(original_lines);
    let middle_n = 100.min(original_lines);
    let conformal_n = 200.min(original_lines);
    
    let mut sampled_content = Vec::new();
    let mut seen_lines = HashSet::new();
    
    // Head
    for i in 0..head_n {
        sampled_content.push(lines[i].to_string());
        seen_lines.insert(i);
    }
    
    // Tail
    let tail_start = original_lines.saturating_sub(tail_n);
    for i in tail_start..original_lines {
        sampled_content.push(lines[i].to_string());
        seen_lines.insert(i);
    }
    
    // Middle
    let mid = original_lines / 2;
    let middle_start = mid.saturating_sub(middle_n / 2);
    let middle_end = (mid + middle_n / 2).min(original_lines);
    for i in middle_start..middle_end {
        if !seen_lines.contains(&i) {
            sampled_content.push(lines[i].to_string());
        }
    }
    
    // Conformal (evenly spaced)
    if original_lines > conformal_n {
        for i in 0..conformal_n {
            let pos = (i * original_lines / conformal_n).min(original_lines - 1);
            if !seen_lines.contains(&pos) {
                sampled_content.push(lines[pos].to_string());
            }
        }
    }
    
    let sample_str = sampled_content.join("\n");
    let entropy = compute_entropy(sample_str.as_bytes());
    
    serde_json::json!({
        "original_lines": original_lines,
        "sample_lines": sampled_content.len(),
        "entropy": entropy,
        "content": sample_str
    })
}

fn compute_entropy(data: &[u8]) -> f64 {
    if data.is_empty() { return 0.0; }
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