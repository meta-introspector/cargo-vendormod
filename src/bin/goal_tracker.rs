//! Goal Tracker CLI
//!
//! Provides ITIL/ISO9001/Six Sigma compliant goal tracking.

use cargo_vendormod::goal_tracker::{Goal, GoalTracker, DmaicPhase, CrqStatus};

fn main() -> anyhow::Result<()> {
    println!("🎯 Goal Tracker - ITIL/ISO9001/Six Sigma Compliant");
    println!("=====================================================");
    
    let mut tracker = GoalTracker::new();
    
    // Create a sample goal for test coverage
    let mut goal = Goal::new("GOAL-001", "Test Coverage Improvement");
    goal.description = "Add comprehensive test coverage to cargo-vendormod modules".to_string();
    goal.specific = "Add 80+ unit tests across 8 modules".to_string();
    goal.measurable = "Test count tracked via cargo test".to_string();
    goal.achievable = "Yes, all modules are testable".to_string();
    goal.relevant = "Improves code quality and CI reliability".to_string();
    goal.time_bound = "Complete by 2026-05-30".to_string();
    goal.target_value = 80.0;
    goal.current_value = 48.0;
    goal.unit = "tests".to_string();
    goal.dmaic_phase = DmaicPhase::Measure;
    goal.crq_status = CrqStatus::InProgress;
    goal.iso_clause = "9.1".to_string();
    
    tracker.add_goal(goal);
    
    // Calculate metrics
    let metrics = tracker.calculate_overall_metrics();
    
    println!("\n📊 Overall Metrics:");
    println!("   Tests completed: 48/80 ({:.0}%)", metrics.yield_pct);
    println!("   Sigma level: {:.2}", metrics.sigma_level);
    println!("   DPU: {:.4}", metrics.dpu);
    
    println!("\n📋 Goals:");
    for (id, g) in &tracker.goals {
        println!("   [{id}] {name}: {phase} phase ({status})",
            id = id,
            name = g.name,
            phase = g.dmaic_phase,
            status = g.crq_status
        );
    }
    
    Ok(())
}