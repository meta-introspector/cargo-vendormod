//! Gitmodules Metadata Parser with Git Config Integration
//!
//! Parses .gitmodules files and .git/config for DASL/IPLD metadata storage.
//! Reference: /mnt/data1/time-2026/02-february/22/dasl/rust/ipld-core/onboarding.sh

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Annotation type for gitmodules entries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub key: String,
    pub value: String,
}

/// Git submodule entry from .gitmodules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmoduleEntry {
    pub path: String,
    pub url: String,
    pub branch: Option<String>,
    pub annotations: Vec<Annotation>,
    pub tracked_attributes: HashMap<String, String>,
}

/// Git config section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSection {
    pub name: String,
    pub values: HashMap<String, String>,
}

/// Parsed .git/config file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    pub sections: HashMap<String, ConfigSection>,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl GitConfig {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    /// Parse .git/config file
    pub fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&content)
    }

    /// Parse .git/config content
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut config = GitConfig::new();
        let mut current_section: Option<String> = None;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                let section_name = trimmed
                    .trim_start_matches('[')
                    .trim_end_matches(']')
                    .to_string();
                current_section = Some(section_name.clone());
                config.sections.entry(section_name.clone()).or_insert(ConfigSection {
                    name: section_name,
                    values: HashMap::new(),
                });
            } else if let Some((key, value)) = trimmed.split_once('=') {
                if let Some(section) = current_section.as_ref() {
                    if let Some(config_section) = config.sections.get_mut(section) {
                        config_section.values.insert(
                            key.trim().to_string(),
                            value.trim().to_string(),
                        );
                    }
                }
            }
        }

        Ok(config)
    }

    /// Get a config value by section and key
    pub fn get(&self, section: &str, key: &str) -> Option<&String> {
        self.sections.get(section)?.values.get(key)
    }
}

/// Parsed .gitmodules file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitmodulesData {
    pub submodules: HashMap<String, SubmoduleEntry>,
    pub total_count: usize,
    pub git_config: Option<GitConfig>,
}

impl Default for GitmodulesData {
    fn default() -> Self {
        Self::new()
    }
}

impl GitmodulesData {
    pub fn new() -> Self {
        Self {
            submodules: HashMap::new(),
            total_count: 0,
            git_config: None,
        }
    }

    /// Parse .gitmodules file with annotation support
    pub fn parse_file<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        Self::parse(&content)
    }

    /// Parse .gitmodules content string
    pub fn parse(content: &str) -> anyhow::Result<Self> {
        let mut data = GitmodulesData::new();
        let mut current_name: Option<String> = None;
        let mut current_path: Option<String> = None;
        let mut current_url: Option<String> = None;
        let mut annotations = Vec::new();
        let mut in_annotations = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if let (Some(name), Some(path), Some(url)) = 
                    (current_name.take(), current_path.take(), current_url.take()) {
                    data.submodules.insert(name.clone(), SubmoduleEntry {
                        path,
                        url,
                        branch: None,
                        annotations: annotations.clone(),
                        tracked_attributes: HashMap::new(),
                    });
                    annotations.clear();
                }
                in_annotations = false;
                continue;
            }

            if trimmed.starts_with("[submodule \"") {
                let name = trimmed
                    .trim_start_matches("[submodule \"")
                    .trim_end_matches("\"]");
                current_name = Some(name.to_string());
                in_annotations = false;
            } else if trimmed.starts_with("path = ") {
                current_path = Some(trimmed.trim_start_matches("path = ").to_string());
            } else if trimmed.starts_with("url = ") {
                current_url = Some(trimmed.trim_start_matches("url = ").to_string());
            } else if trimmed.starts_with("branch = ") {
                if let Some(ref mut entry) = data.submodules.get_mut(current_name.as_ref().unwrap()) {
                    entry.branch = Some(trimmed.trim_start_matches("branch = ").to_string());
                }
            } else if trimmed.starts_with("# gemini-annotations:") {
                in_annotations = true;
            } else if trimmed.starts_with("#   ") && in_annotations {
                let annotation = trimmed.trim_start_matches("#   ");
                if let Some((key, value)) = annotation.split_once(": ") {
                    annotations.push(Annotation {
                        key: key.to_string(),
                        value: value.to_string(),
                    });
                }
            }
        }

        if let (Some(name), Some(path), Some(url)) = 
            (current_name, current_path, current_url) {
            data.submodules.insert(name, SubmoduleEntry {
                path,
                url,
                branch: None,
                annotations,
                tracked_attributes: HashMap::new(),
            });
        }

        data.total_count = data.submodules.len();
        Ok(data)
    }

    /// Parse .git/config and attach to data
    pub fn with_git_config(mut self, config_path: &Path) -> anyhow::Result<Self> {
        self.git_config = Some(GitConfig::parse_file(config_path)?);
        Ok(self)
    }

    /// Find submodules by annotation key/value
    pub fn find_by_annotation(&self, key: &str, value: &str) -> Vec<&SubmoduleEntry> {
        self.submodules.values()
            .filter(|entry| {
                entry.annotations.iter().any(|a| a.key == key && a.value == value)
            })
            .collect()
    }

    /// Set tracked attribute on submodule
    pub fn set_attribute(&mut self, name: &str, key: &str, value: &str) -> bool {
        if let Some(entry) = self.submodules.get_mut(name) {
            entry.tracked_attributes.insert(key.to_string(), value.to_string());
            true
        } else {
            false
        }
    }

    /// Get all submodules with a specific attribute
    pub fn find_by_attribute(&self, key: &str) -> Vec<&SubmoduleEntry> {
        self.submodules.values()
            .filter(|entry| entry.tracked_attributes.contains_key(key))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty() {
        let data = GitmodulesData::parse("").unwrap();
        assert_eq!(data.total_count, 0);
    }

    #[test]
    fn test_parse_simple() {
        let content = r#"[submodule "test"]
	path = test
	url = https://example.com/test.git
"#;
        let data = GitmodulesData::parse(content).unwrap();
        assert_eq!(data.total_count, 1);
        assert!(data.submodules.contains_key("test"));
    }

    #[test]
    fn test_git_config_parse() {
        let content = r#"[core]
	repositoryformatversion = 0
	bare = false
"#;
        let config = GitConfig::parse(content).unwrap();
        assert!(config.sections.contains_key("core"));
        assert_eq!(config.get("core", "bare"), Some(&"false".to_string()));
    }

    #[test]
    fn test_attribute_tracking() {
        let content = r#"[submodule "test"]
	path = test
	url = https://example.com/test.git
"#;
        let mut data = GitmodulesData::parse(content).unwrap();
        data.set_attribute("test", "language", "rust");
        data.set_attribute("test", "type", "library");
        
        let found = data.find_by_attribute("language");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].tracked_attributes.get("language"), Some(&"rust".to_string()));
    }
}