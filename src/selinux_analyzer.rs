//! # SELinux Static Analyzer
//!
//! Rust implementation of `selinux_static_analyze.py`.
//! Parses systemd .service files and generates SELinux policy + access models.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Access requirements extracted from a .service file
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ServiceAccess {
    pub name: String,
    pub user: String,
    pub group: String,
    pub workdir: String,
    pub exec_paths: Vec<String>,
    pub read_paths: Vec<String>,
    pub write_paths: Vec<String>,
    pub env_vars: HashMap<String, String>,
    pub needs_network: bool,
    pub needs_tmp: bool,
    pub no_new_privs: bool,
    pub protect_system: String,
    pub protect_home: String,
    pub cpu_quota: String,
    pub memory_max: String,
    pub after: Vec<String>,
    pub wants: Vec<String>,
}

/// SELinux policy module definition
#[derive(Debug, Clone, Serialize)]
pub struct SELinuxPolicy {
    pub module_name: String,
    pub version: String,
    pub services: Vec<String>,
    pub requires: Vec<String>,
    pub types: Vec<String>,
    pub rules: Vec<String>,
}

/// Lean4 model for formal verification
#[derive(Debug, Clone, Serialize)]
pub struct Lean4Model {
    pub service_name: String,
    pub access_predicates: Vec<String>,
    pub hardening_properties: Vec<String>,
}

/// Parse a systemd .service file
pub fn parse_service(path: impl AsRef<Path>) -> Result<ServiceAccess> {
    let path = path.as_ref();
    let name = path.file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("unknown")
        .replace("-", "_")
        .replace(".", "_");

    let mut svc = ServiceAccess {
        name,
        user: "root".to_string(),
        group: "root".to_string(),
        workdir: "/".to_string(),
        ..Default::default()
    };

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read service file: {}", path.display()))?;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || !line.contains('=') {
            continue;
        }

        let (key, val) = line.split_at(line.find('=').unwrap());
        let key = key.trim();
        let val = val[1..].trim().trim_matches('"');

        match key {
            "User" => svc.user = val.to_string(),
            "Group" => svc.group = val.to_string(),
            "WorkingDirectory" => svc.workdir = val.to_string(),
            "ExecStart" => {
                let executable = val.split_whitespace().next().unwrap_or("");
                svc.exec_paths.push(executable.to_string());
            }
            "ReadWritePaths" => {
                for p in val.split_whitespace() {
                    svc.write_paths.push(p.to_string());
                    svc.read_paths.push(p.to_string());
                }
            }
            "ReadOnlyPaths" => {
                for p in val.split_whitespace() {
                    svc.read_paths.push(p.to_string());
                }
            }
            "Environment" => {
                if let Some((k, v)) = val.split_once('=') {
                    svc.env_vars.insert(k.to_string(), v.to_string());
                }
            }
            "After" => {
                svc.after = val.split_whitespace().map(|s| s.to_string()).collect();
                if svc.after.contains(&"network.target".to_string()) {
                    svc.needs_network = true;
                }
            }
            "Wants" => {
                svc.wants = val.split_whitespace().map(|s| s.to_string()).collect();
            }
            "NoNewPrivileges" if val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("yes") => {
                svc.no_new_privs = true;
            }
            "PrivateTmp" if val.eq_ignore_ascii_case("true") || val.eq_ignore_ascii_case("yes") => {
                svc.needs_tmp = true;
            }
            "ProtectSystem" => svc.protect_system = val.to_string(),
            "ProtectHome" => svc.protect_home = val.to_string(),
            "CPUQuota" => svc.cpu_quota = val.to_string(),
            "MemoryMax" => svc.memory_max = val.to_string(),
            _ => {}
        }
    }

    // Infer read paths
    svc.read_paths.push(svc.workdir.clone());
    for ep in &svc.exec_paths {
        if let Some(dir) = std::path::Path::new(ep).parent() {
            svc.read_paths.push(dir.to_string_lossy().to_string());
        }
    }

    // Deduplicate
    svc.exec_paths.sort();
    svc.exec_paths.dedup();
    svc.read_paths.sort();
    svc.read_paths.dedup();
    svc.write_paths.sort();
    svc.write_paths.dedup();

    Ok(svc)
}

/// Generate SELinux .te policy file content
pub fn generate_te_policy(services: &[ServiceAccess]) -> String {
    let mut lines = vec![
        "policy_module(zkperf_services, 1.0.0)".to_string(),
        "".to_string(),
        "require {".to_string(),
        "    type unconfined_t;".to_string(),
        "    type httpd_t;".to_string(),
        "    type bin_t;".to_string(),
        "    type usr_t;".to_string(),
        "    type tmp_t;".to_string(),
        "    type node_t;".to_string(),
        "    class process { transition signal };".to_string(),
        "    class file { read write getattr open create };".to_string(),
        "    class dir { read search open };".to_string(),
        "    class tcp_socket { create connect read write };".to_string(),
        "    class udp_socket { create connect read write };".to_string(),
        "}".to_string(),
        "".to_string(),
    ];

    // Collect all unique paths
    let mut all_paths = HashSet::new();
    for svc in services {
        all_paths.extend(svc.read_paths.iter().cloned());
        all_paths.extend(svc.write_paths.iter().cloned());
        all_paths.extend(svc.exec_paths.iter().cloned());
    }

    // Define types
    for svc in services {
        let type_name = format!("{}_t", svc.name.replace(['-', '.'], "_"));
        let exec_type = format!("{}_exec_t", svc.name.replace(['-', '.'], "_"));

        lines.push(format!("type {}, user_domain_type, user_usertype;", type_name));
        lines.push(format!("type {};", exec_type));
        lines.push("".to_string());
    }

    // Generate rules
    for svc in services {
        let type_name = format!("{}_t", svc.name.replace(['-', '.'], "_"));

        for path in &svc.read_paths {
            lines.push(format!("allow {} {}:file {{ read getattr open }};", type_name, path));
        }
        for path in &svc.write_paths {
            lines.push(format!("allow {} {}:file {{ write create }};", type_name, path));
        }

        if svc.needs_network {
            lines.push(format!("allow {} node_t:tcp_socket create;", type_name));
            lines.push(format!("allow {} node_t:udp_socket create;", type_name));
        }

        if svc.needs_tmp {
            lines.push(format!("allow {} tmp_t:file {{ read write create }};", type_name));
        }
        lines.push("".to_string());
    }

    lines.join("\n")
}

/// Generate Lean4 formal model
pub fn generate_lean4_model(service: &ServiceAccess) -> String {
    let mut model = String::new();

    model.push_str(&format!("/-- Lean4 Model for {} --/\n", service.name));
    model.push_str("namespace SELinux\n\n");
    model.push_str(&format!("structure {} :=\n", service.name));
    model.push_str("  (user : String)\n");
    model.push_str("  (group : String)\n");
    model.push_str("  (workdir : String)\n");
    model.push_str("  (has_network : Bool)\n");
    model.push_str("  (has_tmp : Bool)\n");
    model.push_str("  (protected_system : Bool)\n");
    model.push_str("  (no_new_privs : Bool)\n\n");

    model.push_str(&format!("def {}.safe_access (s : {}) : Prop :=\n", service.name, service.name));
    model.push_str("  s.no_new_privs ∧\n");
    model.push_str("  (s.protected_system = true → s.has_tmp = false) ∧\n");
    model.push_str("  (s.has_network → s.user = \"root\" ∨ s.group = \"root\")\n\n");

    model.push_str(&format!("theorem {}_safety : ∀ (s : {}), s.safe_access s :=\n", service.name, service.name));
    model.push_str("  by sorry\n\n");

    model.push_str("end SELinux\n");

    model
}

/// Process multiple service files
pub fn process_service_files(paths: &[impl AsRef<Path>]) -> Result<Vec<ServiceAccess>> {
    let mut services = Vec::new();

    for path in paths {
        match parse_service(path) {
            Ok(svc) => {
                println!("  ✅ Parsed: {}", svc.name);
                services.push(svc);
            }
            Err(e) => {
                println!("  ⚠️  Failed to parse {:?}: {}", path.as_ref(), e);
            }
        }
    }

    Ok(services)
}

/// Generate access report (JSON)
pub fn generate_access_report(services: &[ServiceAccess]) -> Result<String> {
    let json = serde_json::to_string_pretty(&services)
        .context("Failed to serialize access report")?;

    Ok(json)
}

// Additional hardening utilities

/// Check for common hardening requirements
pub fn check_hardening(service: &ServiceAccess) -> Vec<String> {
    let mut issues = Vec::new();

    if !service.no_new_privs {
        issues.push("Missing NoNewPrivileges".to_string());
    }
    if service.protect_system.is_empty() {
        issues.push("Missing ProtectSystem".to_string());
    }
    if service.protect_home.is_empty() {
        issues.push("Missing ProtectHome".to_string());
    }
    if !service.needs_tmp && service.write_paths.iter().any(|p| p.contains("/tmp/")) {
        issues.push("Writes to /tmp without PrivateTmp".to_string());
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn create_test_service_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        fs::write(file.path(), content).unwrap();
        file
    }

    #[test]
    fn test_parse_basic_service() {
        let content = r#"
[Unit]
Description=Test Service
After=network.target

[Service]
User=nobody
Group=nobody
ExecStart=/usr/bin/test
WorkingDirectory=/var/lib/test
NoNewPrivileges=yes

[Install]
WantedBy=multi-user.target
"#;
        let file = create_test_service_file(content);
        let svc = parse_service(file.path()).unwrap();

        assert_eq!(svc.user, "nobody");
        assert_eq!(svc.group, "nobody");
        assert_eq!(svc.exec_paths, vec!["/usr/bin/test"]);
        assert!(svc.needs_network);
        assert!(svc.no_new_privs);
    }

    #[test]
    fn test_hardening_check() {
        let svc = ServiceAccess {
            name: "test".to_string(),
            no_new_privs: false,
            protect_system: "".to_string(),
            protect_home: "read-only".to_string(),
            ..Default::default()
        };

        let issues = check_hardening(&svc);
        assert!(issues.iter().any(|i| i.contains("NoNewPrivileges")));
        assert!(issues.iter().any(|i| i.contains("ProtectSystem")));
    }

    #[test]
    fn test_te_policy_generation() {
        let svc = ServiceAccess {
            name: "test_service".to_string(),
            read_paths: vec!["/usr/bin/test".to_string()],
            write_paths: vec!["/var/log/test".to_string()],
            needs_network: true,
            ..Default::default()
        };

        let policy = generate_te_policy(&[svc]);
        assert!(policy.contains("test_service_t"));
        assert!(policy.contains("tcp_socket"));
        assert!(policy.contains("udp_socket"));
    }

    #[test]
    fn test_lean4_model_generation() {
        let svc = ServiceAccess {
            name: "test".to_string(),
            user: "nobody".to_string(),
            no_new_privs: true,
            ..Default::default()
        };

        let model = generate_lean4_model(&svc);
        assert!(model.contains("structure test"));
        assert!(model.contains("no_new_privs"));
        assert!(model.contains("safe_access"));
    }
}