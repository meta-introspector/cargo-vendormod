//! DASL Metadata Processor
//!
//! Processes ~/dasl/git.txt to extract metadata from .gitmodules files
//! across all DASL projects.

use crate::gitmodules_metadata::{GitmodulesData, SubmoduleEntry};
use std::collections::HashMap;
use std::path::Path;

/// Aggregated metadata across all DASL projects
#[derive(Debug, Clone, serde::Serialize)]
pub struct DaslMetadata {
    pub total_projects: usize,
    pub total_submodules: usize,
    pub by_type: HashMap<String, Vec<SubmoduleEntry>>,
    pub by_fuzzer_engine: HashMap<String, Vec<SubmoduleEntry>>,
    pub all_submodules: Vec<SubmoduleEntry>,
}

impl Default for DaslMetadata {
    fn default() -> Self {
        Self::new()
    }
}

impl DaslMetadata {
    pub fn new() -> Self {
        Self {
            total_projects: 0,
            total_submodules: 0,
            by_type: HashMap::new(),
            by_fuzzer_engine: HashMap::new(),
            all_submodules: Vec::new(),
        }
    }

    /// Process git.txt file and extract all submodule metadata
    pub fn process_git_txt<P: AsRef<Path>>(git_txt_path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(git_txt_path.as_ref())?;
        let mut metadata = DaslMetadata::new();

        for line in content.lines() {
            let path = line.trim();
            if path.ends_with(".gitmodules") {
                let full_path = std::path::Path::new("/home/mdupont/dasl").join(path);
                if full_path.exists() {
                    match GitmodulesData::parse_file(&full_path) {
                        Ok(data) => {
                            metadata.total_projects += 1;
                            metadata.total_submodules += data.submodules.len();

                            for entry in data.submodules.into_values() {
                                // Index by type
                                if let Some(type_ann) = entry.annotations.iter()
                                    .find(|a| a.key == "type") {
                                    metadata.by_type.entry(type_ann.value.clone())
                                        .or_insert_with(Vec::new)
                                        .push(entry.clone());
                                }

                                // Index by fuzzer-engine
                                if let Some(engine) = entry.annotations.iter()
                                    .find(|a| a.key == "fuzzer-engine") {
                                    metadata.by_fuzzer_engine.entry(engine.value.clone())
                                        .or_insert_with(Vec::new)
                                        .push(entry.clone());
                                }

                                metadata.all_submodules.push(entry);
                            }
                        }
                        Err(e) => {
                            eprintln!("Warning: Failed to parse {}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(metadata)
    }

    /// Find all fuzzer harnesses
    pub fn fuzzer_harnesses(&self) -> &[SubmoduleEntry] {
        match self.by_type.get("fuzzer-harness") {
            Some(v) => v.as_slice(),
            None => &[],
        }
    }

    /// Get statistics as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dasl_metadata_new() {
        let metadata = DaslMetadata::new();
        assert_eq!(metadata.total_projects, 0);
        assert_eq!(metadata.total_submodules, 0);
        assert!(metadata.all_submodules.is_empty());
    }

    #[test]
    fn test_fuzzer_harnesses_empty() {
        let metadata = DaslMetadata::new();
        assert_eq!(metadata.fuzzer_harnesses().len(), 0);
    }
}