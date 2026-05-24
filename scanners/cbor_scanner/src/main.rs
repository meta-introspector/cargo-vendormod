//! CBOR Byte-Level HMM Scanner
//!
//! Trains Hidden Markov Models on raw byte sequences from CBOR files.
//! Detects anomalies in constant integer regions and structural patterns.

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================
// HMM Model for Byte Sequences
// ============================================================

/// HMM state representation - each state emits bytes with certain probabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMMState {
    /// Transition probabilities: P(next_state | current_state)
    pub transitions: HashMap<u8, f64>,
    /// Emission probabilities: P(byte_value | state)
    pub emissions: HashMap<u8, f64>,
    /// Initial probability of this state
    pub initial_prob: f64,
}

/// Complete HMM model with multiple states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMM {
    /// All states in the model, indexed by state ID
    pub states: Vec<HMMState>,
    /// State names for debugging
    pub state_names: Vec<String>,
}

impl HMM {
    pub fn new(num_states: usize) -> Self {
        let mut states = Vec::with_capacity(num_states);
        let mut state_names = Vec::with_capacity(num_states);
        
        for i in 0..num_states {
            state_names.push(format!("S{}", i));
            states.push(HMMState {
                transitions: HashMap::new(),
                emissions: HashMap::new(),
                initial_prob: 1.0 / num_states as f64,
            });
        }
        
        Self { states, state_names }
    }
    
    /// Train HMM using Baum-Welch (EM) algorithm
    pub fn train_baum_welch(
        &mut self,
        sequences: &[Vec<u8>],
        iterations: usize,
    ) -> Result<()> {
        // TODO: Implement Baum-Welch for byte-level HMM
        // For now, use simpler approach
        self.train_simple(sequences);
        Ok(())
    }
    
    /// Simple training: cluster bytes into states based on value ranges
    pub fn train_simple(&mut self, sequences: &[Vec<u8>]) {
        if self.states.is_empty() { return; }
        
        // Count byte frequencies and co-occurrences
        let mut byte_counts = HashMap::<u8, usize>::new();
        let mut byte_pair_counts = HashMap::<(u8, u8), usize>::new();
        
        for seq in sequences {
            for &byte in seq {
                *byte_counts.entry(byte).or_insert(0) += 1;
            }
            for window in seq.windows(2) {
                *byte_pair_counts.entry((window[0], window[1])).or_insert(0) += 1;
            }
        }
        
        // Distribute bytes across states based on value
        // State 0: control bytes (0-31)
        // State 1: printable ASCII (32-126)
        // State 2: extended bytes (127-255)
        // State 3: CBOR major types (0-7 in high 3 bits)
        
        for (i, state) in self.states.iter_mut().enumerate() {
            // Define byte ranges for each state
            let (start, end) = match i {
                0 => (0u8, 32u8),      // Control bytes
                1 => (32u8, 127u8),    // Printable ASCII
                2 => (127u8, 255u8),   // Extended
                3 => (0u8, 255u8),     // All bytes (for CBOR major types)
                _ => (0u8, 255u8),
            };
            
            // Set emission probabilities based on observed frequencies
            let mut state_total = 0usize;
            for byte in start..end {
                let count = *byte_counts.get(&byte).unwrap_or(&0);
                state_total += count;
            }
            
            for byte in start..end {
                let count = *byte_counts.get(&byte).unwrap_or(&0);
                if state_total > 0 {
                    state.emissions.insert(byte, count as f64 / state_total as f64);
                }
            }
            
            // Set transition probabilities
            let mut transition_total = 0usize;
            for prev in start..end {
                for next in 0u8..=255u8 {
                    let count = *byte_pair_counts.get(&(prev, next)).unwrap_or(&0);
                    transition_total += count;
                }
            }
            
            if transition_total > 0 {
                for prev in start..end {
                    for next in 0u8..=255u8 {
                        let count = *byte_pair_counts.get(&(prev, next)).unwrap_or(&0);
                        if count > 0 {
                            state.transitions.insert(next, count as f64 / transition_total as f64);
                        }
                    }
                }
            }
        }
    }
    
    /// Score a byte sequence using Viterbi algorithm
    pub fn viterbi_score(&self, sequence: &[u8]) -> (f64, Vec<usize>) {
        if sequence.is_empty() || self.states.is_empty() {
            return (0.0, vec![]);
        }
        
        let num_states = self.states.len();
        let seq_len = sequence.len();
        
        // Viterbi DP table: (prob, prev_state)
        let mut dp = vec![vec![(f64::NEG_INFINITY, 0usize); num_states]; seq_len];
        
        // Initialize first step
        for s in 0..num_states {
            let emit_prob = self.states[s].emissions.get(&sequence[0]).copied().unwrap_or(1e-10);
            dp[0][s] = (self.states[s].initial_prob.ln() + emit_prob.ln(), s);
        }
        
        // Fill DP table
        for t in 1..seq_len {
            for s in 0..num_states {
                let emit_prob = self.states[s].emissions.get(&sequence[t]).copied().unwrap_or(1e-10);
                let emit_log = emit_prob.ln();
                
                let mut best = f64::NEG_INFINITY;
                let mut best_prev = 0usize;
                
                for prev_s in 0..num_states {
                    let trans_prob = self.states[prev_s].transitions.get(&sequence[t]).copied()
                        .or_else(|| self.states[prev_s].transitions.get(&sequence[t-1]).copied())
                        .unwrap_or(1e-10);
                    let trans_log = trans_prob.ln();
                    let prev_log = dp[t-1][prev_s].0;
                    let total = prev_log + trans_log + emit_log;
                    
                    if total > best {
                        best = total;
                        best_prev = prev_s;
                    }
                }
                
                dp[t][s] = (best, best_prev);
            }
        }
        
        // Backtrack
        let mut path = Vec::with_capacity(seq_len);
        let mut current_state = (0..num_states).max_by(|&a, &b| 
            dp[seq_len-1][a].0.partial_cmp(&dp[seq_len-1][b].0).unwrap()
        ).unwrap_or(0);
        
        for t in (0..seq_len).rev() {
            path.push(current_state);
            if t > 0 {
                current_state = dp[t][current_state].1;
            }
        }
        path.reverse();
        
        let score = dp[seq_len-1].iter().map(|(p, _)| *p).fold(f64::NEG_INFINITY, f64::max);
        (score, path)
    }
}

// ============================================================
// Byte-Level Markov Model (simpler, like dasl scanner)
// ============================================================

/// First-order Markov model for bytes (0-255)
pub type MarkovMatrix = [[f64; 256]; 256];

/// Train Markov model from byte sequences
pub fn train_markov_model(sequences: &[Vec<u8>]) -> MarkovMatrix {
    let mut counts = [[1u64; 256]; 256]; // Laplace smoothing
    let mut totals = [256u64; 256];
    
    for seq in sequences {
        for window in seq.windows(2) {
            let prev = window[0] as usize;
            let next = window[1] as usize;
            counts[prev][next] += 1;
            totals[prev] += 1;
        }
    }
    
    let mut matrix = [[0.0f64; 256]; 256];
    for i in 0..256 {
        let total = totals[i] as f64;
        for j in 0..256 {
            matrix[i][j] = (counts[i][j] as f64) / total;
        }
    }
    matrix
}

/// Score a byte sequence using Markov model
pub fn score_sequence(seq: &[u8], matrix: &MarkovMatrix) -> f64 {
    if seq.len() < 2 { return 0.0; }
    seq.windows(2).map(|w| matrix[w[0] as usize][w[1] as usize])
        .filter(|&p| p > 0.0)
        .map(|p| p.ln())
        .sum()
}

// ============================================================
// CBOR-Specific Analysis
// ============================================================

/// Analyze CBOR file for constant integers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CBORAnalysis {
    pub path: String,
    pub file_size: usize,
    /// All integer constants found
    pub integers: Vec<i128>,
    /// Positions of integers in file
    pub integer_positions: Vec<usize>,
    /// Byte-level Markov score
    pub markov_score: f64,
    /// HMM state path
    pub hmm_states: Vec<usize>,
    /// Anomalous regions
    pub anomalies: Vec<Anomaly>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    pub offset: usize,
    pub length: usize,
    pub description: String,
    pub score: f64,
}

/// Scan a CBOR file and extract constant integers
pub fn scan_cbor_file(path: &Path) -> Result<CBORAnalysis> {
    let data = fs::read(path)?;
    let file_size = data.len();
    
    // Parse CBOR to find integers
    let mut integers = Vec::new();
    let mut integer_positions = Vec::new();
    
    // Try to parse as CBOR
    if let Ok(value) = ciborium::from_reader::<ciborium::Value, _>(&data[..]) {
        extract_integers(&value, &data, 0, &mut integers, &mut integer_positions);
    }
    
    // Also scan raw bytes for integer patterns
    scan_raw_integers(&data, &mut integers, &mut integer_positions);
    
    Ok(CBORAnalysis {
        path: path.display().to_string(),
        file_size,
        integers,
        integer_positions,
        markov_score: 0.0,
        hmm_states: vec![],
        anomalies: vec![],
    })
}

/// Recursively extract integers from CBOR Value
fn extract_integers(
    value: &ciborium::Value,
    data: &[u8],
    offset: usize,
    integers: &mut Vec<i128>,
    positions: &mut Vec<usize>,
) {
    use ciborium::Value::*;
    
    match value {
        Integer(n) => {
            integers.push((*n).into());
            positions.push(offset);
        }
        Float(f) => {
            // Treat float as integer if it's whole
            if f.fract() == 0.0 {
                integers.push(*f as i128);
                positions.push(offset);
            }
        }
        Array(arr) => {
            for (i, item) in arr.iter().enumerate() {
                // Estimate offset for array element
                let elem_offset = offset + i * 10; // rough estimate
                extract_integers(item, data, elem_offset, integers, positions);
            }
        }
        Map(map) => {
            for (k, v) in map {
                extract_integers(k, data, offset, integers, positions);
                extract_integers(v, data, offset, integers, positions);
            }
        }
        Tag(_, inner) => {
            extract_integers(inner, data, offset, integers, positions);
        }
        Bytes(b) => {
            // Scan bytes for embedded integers
            scan_raw_integers(b, integers, positions);
        }
        Text(_) | Bool(_) | Null => {},
        &_ => {}
    }
}

/// Scan raw byte sequence for integer patterns (little-endian)
fn scan_raw_integers(data: &[u8], integers: &mut Vec<i128>, positions: &mut Vec<usize>) {
    // Look for common integer sizes: u8, u16, u32, u64
    for size in [1, 2, 4, 8] {
        if data.len() < size { continue; }
        for i in 0..=(data.len() - size) {
            let mut val: u64 = 0;
            for (j, &byte) in data[i..i+size].iter().enumerate() {
                val |= (byte as u64) << (8 * j);
            }
            // Only add if not already found via CBOR parsing
            // (This is a simple heuristic; could be improved)
            integers.push(val as i128);
            positions.push(i);
        }
    }
}

// ============================================================
// CLI Interface
// ============================================================

#[derive(Parser, Debug)]
#[command(name = "cbor_scanner")]
#[command(author = "dasl")]
#[command(version = "0.1.0")]
#[command(about = "Byte-level HMM scanner for CBOR files")]
struct Args {
    /// Command to execute
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Train HMM on CBOR files
    Train {
        /// Directory containing CBOR files or file list
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output model file
        #[arg(short, long, default_value = "cbor_hmm.json")]
        output: PathBuf,
        
        /// Number of HMM states
        #[arg(short, long, default_value = "4")]
        states: usize,
    },
    
    /// Scan CBOR files for constants using trained HMM
    Scan {
        /// Directory or file to scan
        #[arg(short, long)]
        input: PathBuf,
        
        /// HMM model file
        #[arg(short, long, default_value = "cbor_hmm.json")]
        model: PathBuf,
        
        /// Output analysis directory
        #[arg(short, long, default_value = "analysis")]
        output: PathBuf,
    },
    
    /// Extract constants from CBOR files
    Extract {
        /// CBOR file or directory
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output JSON file
        #[arg(short, long, default_value = "constants.json")]
        output: PathBuf,
    },
}

// CBOR Scanner Main Binary

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        Command::Train { input, output, states } => {
            let files = collect_cbor_files(&input)?;
            let sequences: Vec<Vec<u8>> = files.iter()
                .filter_map(|p| std::fs::read(p).ok())
                .collect();
            
            if sequences.is_empty() {
                anyhow::bail!("No CBOR files found");
            }
            
            let mut hmm = HMM::new(states);
            hmm.train_simple(&sequences);
            
            let json = serde_json::to_string_pretty(&hmm)?;
            std::fs::write(&output, json)?;
            println!("Trained HMM with {} states, saved to {}", states, output.display());
            println!("Trained on {} files, {} total bytes", 
                sequences.len(), 
                sequences.iter().map(|s| s.len()).sum::<usize>());
        }
        
        Command::Scan { input, model, output } => {
            std::fs::create_dir_all(&output)?;
            
            let model_data = std::fs::read_to_string(&model)
                .with_context(|| format!("Failed to read model: {}", model.display()))?;
            let hmm: HMM = serde_json::from_str(&model_data)?;
            
            let files = if input.is_dir() {
                collect_cbor_files(&input)?
            } else {
                vec![input]
            };
            
            for path in &files {
                let data = std::fs::read(path)?;
                let (score, states) = hmm.viterbi_score(&data);
                let states_clone = states.clone();
                
                let analysis = CBORAnalysis {
                    path: path.display().to_string(),
                    file_size: data.len(),
                    integers: vec![],
                    integer_positions: vec![],
                    markov_score: score,
                    hmm_states: states_clone,
                    anomalies: detect_anomalies(&data, &hmm, &states),
                };
                
                let out_path = output.join(format!("{}.json", path.file_name().unwrap().to_string_lossy()));
                let json = serde_json::to_string_pretty(&analysis)?;
                std::fs::write(&out_path, json)?;
                
                println!("Scanned: {} -> score={:.2}, states={:?}", 
                    path.display(), score, states.iter().take(10).collect::<Vec<_>>());
            }
        }
        
        Command::Extract { input, output } => {
            let files = if input.is_dir() {
                collect_cbor_files(&input)?
            } else {
                vec![input]
            };
            
            let mut all_analysis = Vec::new();
            
            for path in &files {
                let analysis = scan_cbor_file(path)?;
                let integer_count = analysis.integers.len();
                all_analysis.push(analysis);
                
                println!("Extracted {} integers from {}", 
                    integer_count, path.display());
            }
            
            let json = serde_json::to_string_pretty(&all_analysis)?;
            std::fs::write(&output, json)?;
            println!("Saved constants to {}", output.display());
        }
    }
    
    Ok(())
}

/// Collect all .cbor files from a directory
fn collect_cbor_files(path: &Path) -> Result<Vec<PathBuf>> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path().extension().map(|s| s == "cbor").unwrap_or(false) {
            files.push(entry.path().to_path_buf());
        }
    }
    Ok(files)
}

/// Detect anomalous regions in byte sequence
fn detect_anomalies(data: &[u8], hmm: &HMM, states: &[usize]) -> Vec<Anomaly> {
    let mut anomalies = Vec::new();
    
    // Look for rare state transitions or low-probability regions
    if states.len() < 2 { return anomalies; }
    
    for i in 1..states.len() {
        let prev_state = states[i-1];
        let curr_state = states[i];
        let byte = data[i];
        
        // Check if transition is unlikely
        let trans_prob = hmm.states[prev_state].transitions.get(&byte).copied().unwrap_or(0.0);
        if trans_prob < 0.001 {
            anomalies.push(Anomaly {
                offset: i,
                length: 1,
                description: format!("Rare transition from S{} to S{} on byte 0x{:02x}", 
                    prev_state, curr_state, byte),
                score: trans_prob,
            });
        }
    }
    
    anomalies
}
