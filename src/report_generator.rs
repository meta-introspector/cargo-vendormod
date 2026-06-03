//! Report Generator
//! Generates comprehensive text reports on workloads and their state

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SubmoduleInfo {
    pub name: String,
    pub path: String,
    pub url: String,
    pub branch: String,
    pub commit: String,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CrateInfo {
    pub name: String,
    pub version: String,
    pub source: String,
    pub dependencies: Vec<String>,
    pub path: String,
    pub is_workspace_member: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitRepoInfo {
    pub name: String,
    pub path: String,
    pub remote: String,
    pub branch: String,
    pub status: GitStatus,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GitStatus {
    pub clean: bool,
    pub staged: usize,
    pub modified: usize,
    pub untracked: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkloadReport {
    pub generated_at: String,
    pub workspace_path: String,
    pub summary: ReportSummary,
    pub submodules: Vec<SubmoduleInfo>,
    pub crates: Vec<CrateInfo>,
    pub git_repos: Vec<GitRepoInfo>,
    pub workflow_state: WorkflowState,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_submodules: usize,
    pub total_crates: usize,
    pub total_git_repos: usize,
    pub total_external_deps: usize,
    pub total_workspace_members: usize,
    pub state: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowState {
    pub workflows_defined: usize,
    pub scripts_available: usize,
    pub flakes_generated: usize,
    pub last_execution: Option<String>,
    pub status: String,
}

pub struct ReportGenerator {
    workspace_path: PathBuf,
    output_dir: PathBuf,
}

impl ReportGenerator {
    pub fn new(workspace_path: PathBuf, output_dir: PathBuf) -> Self {
        Self {
            workspace_path,
            output_dir,
        }
    }

    pub fn generate_full_report(&self) -> Result<WorkloadReport> {
        let submodules = self.collect_submodules()?;
        let crates = self.collect_crates()?;
        let git_repos = self.collect_git_repos()?;
        let workflow_state = self.collect_workflow_state()?;

        let total_external_deps = crates.iter()
            .filter(|c| !c.is_workspace_member)
            .count();
        let total_workspace_members = crates.iter()
            .filter(|c| c.is_workspace_member)
            .count();

        let state = if submodules.is_empty() && git_repos.len() <= 1 {
            "Clean - No submodules".to_string()
        } else if !submodules.is_empty() {
            format!("Active - {} submodules", submodules.len())
        } else {
            "Initialized".to_string()
        };

        let summary = ReportSummary {
            total_submodules: submodules.len(),
            total_crates: crates.len(),
            total_git_repos: git_repos.len(),
            total_external_deps,
            total_workspace_members,
            state: state.clone(),
        };

        let report = WorkloadReport {
            generated_at: chrono::Local::now().to_rfc3339(),
            workspace_path: self.workspace_path.display().to_string(),
            summary,
            submodules,
            crates,
            git_repos,
            workflow_state,
        };

        Ok(report)
    }

    fn collect_submodules(&self) -> Result<Vec<SubmoduleInfo>> {
        let mut submodules = Vec::new();
        
        // Check .gitmodules file
        let gitmodules_path = self.workspace_path.join(".gitmodules");
        if gitmodules_path.exists() {
            let output = Command::new("git")
                .args(["-C", self.workspace_path.to_str().unwrap(), "submodule", "status"])
                .output()?;

            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.trim().split_whitespace().collect();
                    if parts.len() >= 2 {
                        let commit = parts[0].trim_start_matches(|c| c == '+' || c == '-' || c == 'U');
                        let path = parts[1];
                        
                        submodules.push(SubmoduleInfo {
                            name: path.to_string(),
                            path: path.to_string(),
                            url: "".to_string(),
                            branch: "".to_string(),
                            commit: commit.to_string(),
                            state: if line.contains('+') { "Modified" } 
                                  else if line.contains('-') { "Not initialized" }
                                  else { "Clean" }.to_string(),
                        });
                    }
                }
            }
        }

        Ok(submodules)
    }

    fn collect_crates(&self) -> Result<Vec<CrateInfo>> {
        let mut crates = Vec::new();
        
        // Check for Cargo.toml in workspace
        let cargo_toml_path = self.workspace_path.join("Cargo.toml");
        if cargo_toml_path.exists() {
            let cargo_content = fs::read_to_string(&cargo_toml_path)?;
            let cargo: serde_json::Value = match toml::from_str::<toml::Value>(&cargo_content) {
                Ok(c) => serde_json::to_value(c).unwrap_or_default(),
                Err(_) => serde_json::Value::default(),
            };

            // Check workspace members
            if let Some(members) = cargo.get("workspace").and_then(|w| w.get("members")) {
                if let Some(members) = members.as_array() {
                    for member in members {
                        if let Some(member_str) = member.as_str() {
                            crates.push(CrateInfo {
                                name: member_str.to_string(),
                                version: "workspace".to_string(),
                                source: "workspace-member".to_string(),
                                dependencies: vec![],
                                path: member_str.to_string(),
                                is_workspace_member: true,
                            });
                        }
                    }
                }
            }
        }

        // Check for Cargo.lock to get dependency info
        let cargo_lock_path = self.workspace_path.join("Cargo.lock");
        if cargo_lock_path.exists() {
            let lock_content = fs::read_to_string(&cargo_lock_path)?;
            let lock: toml::Value = toml::from_str(&lock_content).unwrap_or(toml::Value::Boolean(false));
            
            if let Some(packages) = lock.get("package").and_then(|p| p.as_array()) {
                for pkg in packages.iter().take(50) { // Limit to first 50 for readability
                    if let (Some(name), Some(version)) = (
                        pkg.get("name").and_then(|n| n.as_str()),
                        pkg.get("version").and_then(|v| v.as_str()),
                    ) {
                        let source = pkg.get("source")
                            .and_then(|s| s.as_str())
                            .unwrap_or("registry")
                            .to_string();
                        
                        let is_member = crates.iter().any(|c: &CrateInfo| c.name == name);
                        
                        if !is_member && crates.len() < 100 {
                            crates.push(CrateInfo {
                                name: name.to_string(),
                                version: version.to_string(),
                                source,
                                dependencies: vec![],
                                path: "".to_string(),
                                is_workspace_member: false,
                            });
                        }
                    }
                }
            }
        }

        Ok(crates)
    }

    fn collect_git_repos(&self) -> Result<Vec<GitRepoInfo>> {
        let mut repos = Vec::new();
        
        // Check main repo
        let git_dir = self.workspace_path.join(".git");
        if git_dir.exists() {
            let status = self.get_git_status(self.workspace_path.as_path())?;
            
            let remote_output = Command::new("git")
                .args(["-C", self.workspace_path.to_str().unwrap(), "remote", "get-url", "origin"])
                .output();
            
            let branch_output = Command::new("git")
                .args(["-C", self.workspace_path.to_str().unwrap(), "branch", "--show-current"])
                .output();

            let remote = remote_output.ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "unknown".to_string())
                .trim()
                .to_string();

            let branch = branch_output.ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .unwrap_or_else(|| "unknown".to_string())
                .trim()
                .to_string();

            repos.push(GitRepoInfo {
                name: self.workspace_path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                path: self.workspace_path.display().to_string(),
                remote,
                branch,
                status,
            });
        }

        Ok(repos)
    }

    fn get_git_status(&self, path: &Path) -> Result<GitStatus> {
        let output = Command::new("git")
            .args(["-C", path.to_str().unwrap(), "status", "--porcelain"])
            .output()?;

        let mut staged = 0;
        let mut modified = 0;
        let mut untracked = 0;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for line in stdout.lines() {
                if line.starts_with("??") {
                    untracked += 1;
                } else if line.chars().nth(0).map_or(false, |c| c == 'M' || c == 'A' || c == 'D') {
                    staged += 1;
                } else {
                    modified += 1;
                }
            }
        }

        Ok(GitStatus {
            clean: staged == 0 && modified == 0 && untracked == 0,
            staged,
            modified,
            untracked,
        })
    }

    fn collect_workflow_state(&self) -> Result<WorkflowState> {
        let mut workflows_defined = 0;
        let mut flakes_generated = 0;
        
        // Count workflow definitions
        let workflows_dir = self.workspace_path.join("workload/workflows");
        if workflows_dir.exists() {
            if let Ok(entries) = fs::read_dir(workflows_dir) {
                workflows_defined = entries.filter_map(Result::ok).count();
            }
        }

        // Count scripts
        let scripts_dir = self.workspace_path.join("workload/scripts");
        let scripts_available = if scripts_dir.exists() {
            fs::read_dir(scripts_dir).map(|entries| entries.filter_map(Result::ok).count()).unwrap_or(0)
        } else {
            0
        };

        // Count flakes
        let flakes_dirs = [
            self.workspace_path.join("workload/intermediates/layer1_output"),
            self.workspace_path.join("workload/intermediates/layer2_output"),
        ];
        for dir in flakes_dirs.iter() {
            if dir.exists() {
                if let Ok(entries) = fs::read_dir(dir) {
                    flakes_generated += entries.filter_map(Result::ok).count();
                }
            }
        }

        Ok(WorkflowState {
            workflows_defined,
            scripts_available,
            flakes_generated,
            last_execution: None,
            status: if flakes_generated > 0 { "Processed" } else { "Ready" }.to_string(),
        })
    }

    pub fn format_text_report(&self, report: &WorkloadReport) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("╔══════════════════════════════════════════════════════════════════════════════╗\n"));
        output.push_str(&format!("║                                                                              ║\n"));
        output.push_str(&format!("║                    WORKLOAD STATUS REPORT                                    ║\n"));
        output.push_str(&format!("║                                                                              ║\n"));
        output.push_str(&format!("╚══════════════════════════════════════════════════════════════════════════════╝\n"));
        output.push_str(&format!("\n"));
        
        output.push_str(&format!("📅 Generated: {}\n", report.generated_at));
        output.push_str(&format!("📂 Workspace: {}\n\n", report.workspace_path));
        
        output.push_str(&format!("{} SUMMARY {}\n", "═".repeat(35), "═".repeat(37)));
        output.push_str(&format!("  State:              {}\n", report.summary.state));
        output.push_str(&format!("  Total Submodules:   {}\n", report.summary.total_submodules));
        output.push_str(&format!("  Total Crates:       {}\n", report.summary.total_crates));
        output.push_str(&format!("  External Deps:      {}\n", report.summary.total_external_deps));
        output.push_str(&format!("  Workspace Members:  {}\n", report.summary.total_workspace_members));
        output.push_str(&format!("  Git Repositories:   {}\n\n", report.summary.total_git_repos));
        
        if !report.git_repos.is_empty() {
            output.push_str(&format!("{} GIT REPOSITORIES {}\n", "═".repeat(36), "═".repeat(36)));
            for repo in &report.git_repos {
                output.push_str(&format!("  📁 {}\n", repo.name));
                output.push_str(&format!("     Path:  {}\n", repo.path));
                output.push_str(&format!("     Remote: {}\n", repo.remote));
                output.push_str(&format!("     Branch: {}\n", repo.branch));
                output.push_str(&format!("     Clean:  {}\n\n", repo.status.clean));
            }
        }
        
        if !report.submodules.is_empty() {
            output.push_str(&format!("{} SUBMODULES {}\n", "═".repeat(39), "═".repeat(40)));
            for sub in &report.submodules {
                output.push_str(&format!("  📦 {} [{}]\n", sub.name, sub.state));
                output.push_str(&format!("     Path:     {}\n", sub.path));
                output.push_str(&format!("     Commit:   {}\n\n", sub.commit));
            }
        }
        
        if !report.crates.is_empty() {
            output.push_str(&format!("{} CRATES (sample) {}\n", "═".repeat(37), "═".repeat(36)));
            for (i, krate) in report.crates.iter().take(30).enumerate() {
                if i >= 30 { output.push_str("  ... (truncated)\n"); break; }
                let marker = if krate.is_workspace_member { "📦" } else { "📦" };
                output.push_str(&format!("  {} {} v{} ({})\n", marker, krate.name, krate.version, krate.source));
            }
            if report.crates.len() > 30 {
                output.push_str(&format!("  ... and {} more\n", report.crates.len() - 30));
            }
            output.push_str(&format!("\n"));
        }
        
        output.push_str(&format!("{} WORKFLOW STATE {}\n", "═".repeat(38), "═".repeat(36)));
        output.push_str(&format!("  Workflows Defined:  {}\n", report.workflow_state.workflows_defined));
        output.push_str(&format!("  Scripts Available:  {}\n", report.workflow_state.scripts_available));
        output.push_str(&format!("  Flakes Generated:   {}\n", report.workflow_state.flakes_generated));
        output.push_str(&format!("  Status:             {}\n\n", report.workflow_state.status));
        
        output.push_str(&format!("╔══════════════════════════════════════════════════════════════════════════════╗\n"));
        output.push_str(&format!("║                                                                              ║\n"));
        output.push_str(&format!("║                     🚀 CARGO-VENDORMOD READY 🚀                             ║\n"));
        output.push_str(&format!("║                                                                              ║\n"));
        output.push_str(&format!("╚══════════════════════════════════════════════════════════════════════════════╝\n"));
        
        output
    }
}
