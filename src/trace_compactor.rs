//! # Trace Compactor Module
//!
//! Rust implementation of `compact-sample-trace.py`.
//! Compacts normalized sample-trace JSON by removing deterministic derived fields.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Register labels for trace annotations
const REGISTER_LABELS: [&str; 6] = [
    "idx", "log10(period+1)", "log10(ts_gap+1)", "pid", "tid", "cpu_mode",
];

/// CPU mode signal mapping
const CPU_MODE_SIGNAL: [(&str, f64); 2] = [
    ("Kernel", -1.0),
    ("User", 1.0),
];

/// Compact sample trace representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompactTrace {
    pub trace_kind: String,
    pub source_trace_kind: Option<String>,
    pub source_dir: Option<String>,
    pub artifact: Option<serde_json::Value>,
    pub template_set: Option<serde_json::Value>,
    pub shard_family_counts: serde_json::Value,
    pub events: Vec<String>,
    pub rows: Vec<TraceRow>,
}

/// Single row in compact trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRow {
    pub step: i64,
    pub event_idx: usize,
    pub timestamp: i64,
    pub period: i64,
    pub pid: i64,
    pub tid: i64,
    pub cpu_mode: String,
    pub cid: serde_json::Value,
}

/// Annotation from source trace
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Annotation {
    pub transition: String,
    pub step: i64,
    pub timestamp: i64,
    pub period: i64,
    pub next_state: Vec<i64>,
    pub cpu_mode: String,
    pub cid: serde_json::Value,
}

/// Source trace structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SourceTrace {
    pub trace_kind: String,
    pub source_dir: Option<String>,
    pub step_annotations: Vec<Annotation>,
    #[serde(flatten)]
    pub metadata: serde_json::Value,
}

/// Load JSON trace from file
pub fn load_trace(path: impl AsRef<Path>) -> Result<SourceTrace> {
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read trace file: {}", path.display()))?;
    let trace: SourceTrace = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON from: {}", path.display()))?;
    Ok(trace)
}

/// Write compact trace to file
pub fn write_compact_trace(path: impl AsRef<Path>, trace: &CompactTrace) -> Result<()> {
    let path = path.as_ref();
    let content = serde_json::to_string_pretty(trace)
        .context("Failed to serialize compact trace")?;
    fs::write(path, content)
        .with_context(|| format!("Failed to write to: {}", path.display()))?;
    Ok(())
}

/// Convert value to integer, handling floats and integers
fn as_int(value: serde_json::Value) -> i64 {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                i
            } else if let Some(f) = n.as_f64() {
                f as i64
            } else {
                0
            }
        }
        _ => 0,
    }
}

/// Create a matrix row for trace encoding
fn matrix_row(step: i64, period: i64, ts_gap: i64, pid: i64, tid: i64, cpu_mode: &str) -> Vec<f64> {
    let period_val = if period > 0 { (period as f64 + 1.0).log10() } else { 0.0 };
    let ts_gap_val = if ts_gap != 0 { (ts_gap.abs() as f64 + 1.0).log10() } else { 0.0 };
    let cpu_signal = CPU_MODE_SIGNAL
        .iter()
        .find(|(mode, _)| *mode == cpu_mode)
        .map(|(_, sig)| *sig)
        .unwrap_or(0.0);

    vec![
        step as f64,
        period_val,
        ts_gap_val,
        pid as f64,
        tid as f64,
        cpu_signal,
    ]
}

/// Encode a source trace into compact format
pub fn encode_trace(trace: SourceTrace) -> CompactTrace {
    let mut events: Vec<String> = Vec::new();
    let mut event_to_index: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut rows: Vec<TraceRow> = Vec::new();

    let metadata = trace.metadata.as_object().cloned().unwrap_or_default();

    for annotation in trace.step_annotations {
        let event_index = event_to_index
            .entry(annotation.transition.clone())
            .or_insert_with(|| {
                events.push(annotation.transition.clone());
                events.len() - 1
            });

        let _ts_gap = annotation.timestamp
            - rows.last().map(|r: &TraceRow| r.timestamp).unwrap_or(0);

        let pid = annotation.next_state.get(3).copied().unwrap_or(0);
        let tid = annotation.next_state.get(4).copied().unwrap_or(0);

        rows.push(TraceRow {
            step: annotation.step,
            event_idx: *event_index,
            timestamp: annotation.timestamp,
            period: annotation.period,
            pid,
            tid,
            cpu_mode: annotation.cpu_mode,
            cid: annotation.cid,
        });
    }

    CompactTrace {
        trace_kind: "sample_trace_compact/v1".to_string(),
        source_trace_kind: Some(trace.trace_kind),
        source_dir: trace.source_dir,
        artifact: metadata.get("artifact").cloned(),
        template_set: metadata.get("template_set").cloned(),
        shard_family_counts: metadata.get("shard_family_counts").cloned().unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        events,
        rows,
    }
}

/// Decode a compact trace back to source format
pub fn decode_trace(compact: CompactTrace) -> Result<SourceTrace> {
    if compact.trace_kind != "sample_trace_compact/v1" {
        anyhow::bail!("Unsupported trace_kind: {}", compact.trace_kind);
    }

    let mut step_annotations = Vec::new();
    let mut prev_timestamp = 0;

    for row in &compact.rows {
        let period = row.period;
        let ts_gap = row.timestamp - prev_timestamp;
        prev_timestamp = row.timestamp;

        let _matrix = matrix_row(row.step, period, ts_gap, row.pid, row.tid, &row.cpu_mode);

        step_annotations.push(Annotation {
            transition: compact.events.get(row.event_idx).cloned().unwrap_or_default(),
            step: row.step,
            timestamp: row.timestamp,
            period,
            next_state: vec![0, 0, 0, row.pid, row.tid],
            cpu_mode: row.cpu_mode.clone(),
            cid: row.cid.clone(),
        });
    }

    let mut metadata = serde_json::Map::new();
    metadata.insert("artifact".to_string(), compact.artifact.unwrap_or(serde_json::Value::Null));
    metadata.insert("template_set".to_string(), compact.template_set.unwrap_or(serde_json::Value::Null));
    metadata.insert("shard_family_counts".to_string(), compact.shard_family_counts);

    Ok(SourceTrace {
        trace_kind: compact.source_trace_kind.unwrap_or_default(),
        source_dir: compact.source_dir,
        step_annotations,
        metadata: serde_json::Value::Object(metadata),
    })
}

/// Process a trace file: load, encode, save
pub fn compact_trace_file(input_path: impl AsRef<Path>, output_path: impl AsRef<Path>) -> Result<()> {
    let trace = load_trace(&input_path)?;
    let compact = encode_trace(trace);
    write_compact_trace(&output_path, &compact)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::NamedTempFile;

    fn create_test_trace() -> SourceTrace {
        SourceTrace {
            trace_kind: "sample_trace/v1".to_string(),
            source_dir: Some("/test/traces".to_string()),
            step_annotations: vec![
                Annotation {
                    transition: "event1".to_string(),
                    step: 0,
                    timestamp: 1000,
                    period: 10,
                    next_state: vec![0, 0, 0, 1234, 5678],
                    cpu_mode: "User".to_string(),
                    cid: json!(["test", 1]),
                },
                Annotation {
                    transition: "event2".to_string(),
                    step: 1,
                    timestamp: 2000,
                    period: 5,
                    next_state: vec![0, 0, 0, 1234, 5678],
                    cpu_mode: "Kernel".to_string(),
                    cid: json!(["test", 2]),
                },
            ],
            metadata: json!({
                "artifact": "test.artifact",
                "template_set": "test-set",
                "shard_family_counts": {"f1": 10}
            }),
        }
    }

    #[test]
    fn test_encode_decode_roundtrip() {
        let original = create_test_trace();
        let compact = encode_trace(original.clone());
        let decoded = decode_trace(compact).unwrap();

        assert_eq!(original.step_annotations.len(), decoded.step_annotations.len());
        assert_eq!(original.trace_kind, decoded.trace_kind);
    }

    #[test]
    fn test_file_roundtrip() {
        let trace = create_test_trace();
        let input_file = NamedTempFile::new().unwrap();
        let output_file = NamedTempFile::new().unwrap();

        // Write original trace to temp file
        let content = serde_json::to_string_pretty(&trace).unwrap();
        fs::write(input_file.path(), content).unwrap();

        // Compact the trace
        compact_trace_file(input_file.path(), output_file.path()).unwrap();

        // Load compacted trace
        let compacted: CompactTrace = serde_json::from_str(
            &fs::read_to_string(output_file.path()).unwrap()
        ).unwrap();

        assert_eq!(compacted.events.len(), 2);
        assert_eq!(compacted.rows.len(), 2);
        assert_eq!(compacted.trace_kind, "sample_trace_compact/v1");
    }

    #[test]
    fn test_matrix_row_generation() {
        let row = matrix_row(10, 100, 500, 1234, 5678, "User");
        assert_eq!(row.len(), 6);
        assert!((row[1] - 2.00432).abs() < 0.001); // log10(101)
        assert!((row[2] - 2.69976).abs() < 0.001); // log10(501)
        assert!((row[5] - 1.0).abs() < 0.001); // User mode signal
    }

    #[test]
    fn test_cpu_mode_unknown() {
        let row = matrix_row(0, 0, 0, 0, 0, "UnknownMode");
        assert_eq!(row[5], 0.0);
    }
}