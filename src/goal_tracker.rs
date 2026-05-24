//! # Goal Tracking System with ITIL/ISO9001/SixSigma
//!
//! Provides goal tracking following ITIL change management, ISO 9001 quality
//! standards, and Six Sigma DMAIC methodology.
//!
//! ## Standards Compliance
//! - ITIL v4: Change enablement, continual improvement
//! - ISO 9001:2015: Clause 9.1 (monitoring), 10.2 (improvement)
//! - Six Sigma DMAIC: Define, Measure, Analyze, Improve, Control

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Six Sigma Quality Metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SixSigmaMetrics {
    pub dpu: f64,        // Defects Per Unit
    pub dpmo: f64,       // Defects Per Million Opportunities
    pub sigma_level: f64, // Sigma level (0-6)
    pub yield_pct: f64,  // Process yield percentage
}

impl Default for SixSigmaMetrics {
    fn default() -> Self {
        Self {
            dpu: 0.0,
            dpmo: 0.0,
            sigma_level: 0.0,
            yield_pct: 0.0,
        }
    }
}

impl SixSigmaMetrics {
    /// Calculate sigma level from DPU
    pub fn calculate_sigma(&mut self) -> f64 {
        // Simplified sigma calculation
        // sigma = 1.5 + normsinv(1 - dpu)
        self.sigma_level = if self.dpu > 0.0 && self.dpu < 1.0 {
            1.5 + ((-2.0 * self.dpu.ln()).sqrt())
        } else if self.dpu >= 1.0 {
            1.5
        } else {
            6.0
        };
        self.sigma_level = self.sigma_level.min(6.0).max(0.0);
        self.sigma_level
    }
}

/// DMAIC Phase for Six Sigma improvement
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DmaicPhase {
    Define,   // Define the problem and goals
    Measure,  // Measure current performance
    Analyze,  // Analyze data to find root cause
    Improve,  // Implement improvements
    Control,  // Control future process performance
}

impl Default for DmaicPhase {
    fn default() -> Self {
        DmaicPhase::Define
    }
}

impl std::fmt::Display for DmaicPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DmaicPhase::Define => write!(f, "Define"),
            DmaicPhase::Measure => write!(f, "Measure"),
            DmaicPhase::Analyze => write!(f, "Analyze"),
            DmaicPhase::Improve => write!(f, "Improve"),
            DmaicPhase::Control => write!(f, "Control"),
        }
    }
}

/// ITIL CRQ Status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CrqStatus {
    Open,
    InProgress,
    QualityGate,
    Done,
    Rejected,
}

impl Default for CrqStatus {
    fn default() -> Self {
        CrqStatus::Open
    }
}

impl std::fmt::Display for CrqStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrqStatus::Open => write!(f, "Open"),
            CrqStatus::InProgress => write!(f, "In Progress"),
            CrqStatus::QualityGate => write!(f, "Quality Gate"),
            CrqStatus::Done => write!(f, "Done"),
            CrqStatus::Rejected => write!(f, "Rejected"),
        }
    }
}

/// SMART Goal Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Goal {
    pub id: String,
    pub name: String,
    pub description: String,
    pub specific: String,
    pub measurable: String,
    pub achievable: String,
    pub relevant: String,
    pub time_bound: String,
    pub target_value: f64,
    pub current_value: f64,
    pub unit: String,
    pub dmaic_phase: DmaicPhase,
    pub crq_status: CrqStatus,
    pub iso_clause: String,
    pub quality_gate_date: Option<String>,
}

impl Goal {
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            specific: String::new(),
            measurable: String::new(),
            achievable: String::new(),
            relevant: String::new(),
            time_bound: String::new(),
            target_value: 0.0,
            current_value: 0.0,
            unit: String::new(),
            dmaic_phase: DmaicPhase::Define,
            crq_status: CrqStatus::Open,
            iso_clause: String::new(),
            quality_gate_date: None,
        }
    }

    pub fn progress_percent(&self) -> f64 {
        if self.target_value == 0.0 {
            return 0.0;
        }
        (self.current_value / self.target_value * 100.0).min(100.0)
    }

    pub fn quality_gate_met(&self, metrics: &SixSigmaMetrics) -> bool {
        metrics.sigma_level >= 3.0
    }
}

/// ISO 9001:2015 Compliance Record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsoComplianceRecord {
    pub clause: String,
    pub requirement: String,
    pub status: String,
    pub evidence: Vec<String>,
}

/// Goal Tracker - Main orchestrator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalTracker {
    pub goals: HashMap<String, Goal>,
    pub metrics: SixSigmaMetrics,
    pub compliance_records: Vec<IsoComplianceRecord>,
}

impl Default for GoalTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl GoalTracker {
    pub fn new() -> Self {
        Self {
            goals: HashMap::new(),
            metrics: SixSigmaMetrics::default(),
            compliance_records: Vec::new(),
        }
    }

    pub fn add_goal(&mut self, goal: Goal) {
        self.goals.insert(goal.id.clone(), goal);
    }

    pub fn advance_phase(&mut self, goal_id: &str) -> anyhow::Result<()> {
        let goal = self.goals.get_mut(goal_id)
            .ok_or_else(|| anyhow::anyhow!("Goal not found"))?;
        
        goal.dmaic_phase = match goal.dmaic_phase {
            DmaicPhase::Define => DmaicPhase::Measure,
            DmaicPhase::Measure => DmaicPhase::Analyze,
            DmaicPhase::Analyze => DmaicPhase::Improve,
            DmaicPhase::Improve => DmaicPhase::Control,
            DmaicPhase::Control => DmaicPhase::Control, // Hold at Control
        };
        
        Ok(())
    }

    pub fn update_crq_status(&mut self, goal_id: &str, status: CrqStatus) -> anyhow::Result<()> {
        let goal = self.goals.get_mut(goal_id)
            .ok_or_else(|| anyhow::anyhow!("Goal not found"))?;
        goal.crq_status = status;
        Ok(())
    }

    pub fn calculate_overall_metrics(&self) -> SixSigmaMetrics {
        let goals_count = self.goals.len() as f64;
        if goals_count == 0.0 {
            return SixSigmaMetrics::default();
        }
        
        let total_progress: f64 = self.goals.values()
            .map(|g| g.progress_percent())
            .sum();
        
        let avg_progress = total_progress / goals_count;
        // Convert progress to defects
        let dpu = (100.0 - avg_progress) / 100.0;
        
        let mut metrics = SixSigmaMetrics {
            dpu,
            dpmo: dpu * 1_000_000.0,
            sigma_level: 0.0,
            yield_pct: avg_progress,
        };
        metrics.calculate_sigma();
        metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigma_calculation() {
        let mut metrics = SixSigmaMetrics {
            dpu: 0.002,
            dpmo: 2000.0,
            sigma_level: 0.0,
            yield_pct: 99.8,
        };
        metrics.calculate_sigma();
        assert!(metrics.sigma_level > 4.0);
    }

    #[test]
    fn test_goal_progress() {
        let goal = Goal {
            id: "test".to_string(),
            name: "Test Goal".to_string(),
            description: String::new(),
            specific: String::new(),
            measurable: String::new(),
            achievable: String::new(),
            relevant: String::new(),
            time_bound: String::new(),
            target_value: 100.0,
            current_value: 75.0,
            unit: "%".to_string(),
            dmaic_phase: DmaicPhase::Measure,
            crq_status: CrqStatus::InProgress,
            iso_clause: "9.1".to_string(),
            quality_gate_date: None,
        };
        assert_eq!(goal.progress_percent(), 75.0);
    }

    #[test]
    fn test_goal_tracker() {
        let mut tracker = GoalTracker::new();
        let goal = Goal::new("g1", "Test");
        tracker.add_goal(goal);
        assert_eq!(tracker.goals.len(), 1);
    }
}