use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};
use serde::{Serialize, Deserialize};
use petgraph::graph::{Graph, NodeIndex};
use petgraph::dot::{Dot, Config};

/// Data flow graph representing register flow through a workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowGraph {
    pub nodes: Vec<DataFlowNode>,
    pub edges: Vec<DataFlowEdge>,
    pub phases: Vec<PhaseTransition>,
    pub invariants: Vec<Invariant>,
}

/// Node in the data flow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowNode {
    pub id: String,
    pub node_type: DataFlowNodeType,
    pub register: Option<String>,
    pub value_type: Option<String>,
    pub phase: String,
    pub location: String,
    pub properties: HashMap<String, String>,
}

/// Type of data flow node
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFlowNodeType {
    Input,
    Output,
    Register,
    Memory,
    Operation,
    Constant,
    PhaseBoundary,
}

/// Edge in the data flow graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowEdge {
    pub from: String,
    pub to: String,
    pub edge_type: DataFlowEdgeType,
    pub transformation: Option<String>,
    pub preserves_invariant: bool,
    pub properties: HashMap<String, String>,
}

/// Type of data flow edge
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataFlowEdgeType {
    DirectFlow,
    Transformation,
    Copy,
    Move,
    Load,
    Store,
    PhaseTransition,
}

/// Phase transition in the workload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    pub phase_name: String,
    pub input_registers: HashSet<String>,
    pub output_registers: HashSet<String>,
    pub pre_conditions: Vec<String>,
    pub post_conditions: Vec<String>,
    pub invariants_preserved: Vec<String>,
    pub location: String,
}

/// Invariant that must be preserved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invariant {
    pub id: String,
    pub description: String,
    pub applies_to: Vec<String>, // Registers, memory locations, etc.
    pub expression: String,
    pub verified: bool,
    pub verification_method: String,
    pub phases: Vec<String>, // Phases where this invariant applies
}

/// Data flow analyzer for workloads
pub struct DataFlowAnalyzer {
    graph: Graph<DataFlowNode, DataFlowEdge>,
    node_map: HashMap<String, NodeIndex>,
    current_phase: String,
    invariants: Vec<Invariant>,
    phase_transitions: Vec<PhaseTransition>,
}

impl DataFlowAnalyzer {
    /// Create a new DataFlowAnalyzer
    pub fn new(initial_phase: &str) -> Self {
        Self {
            graph: Graph::new(),
            node_map: HashMap::new(),
            current_phase: initial_phase.to_string(),
            invariants: Vec::new(),
            phase_transitions: Vec::new(),
        }
    }
    
    /// Add an input node to the graph
    pub fn add_input(&mut self, name: &str, register: &str, value_type: &str, location: &str) -> Result<()> {
        let node_id = format!("input_{}", name);
        let node = DataFlowNode {
            id: node_id.clone(),
            node_type: DataFlowNodeType::Input,
            register: Some(register.to_string()),
            value_type: Some(value_type.to_string()),
            phase: self.current_phase.clone(),
            location: location.to_string(),
            properties: HashMap::new(),
        };
        
        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        
        Ok(())
    }
    
    /// Add an output node to the graph
    pub fn add_output(&mut self, name: &str, register: &str, value_type: &str, location: &str) -> Result<()> {
        let node_id = format!("output_{}", name);
        let node = DataFlowNode {
            id: node_id.clone(),
            node_type: DataFlowNodeType::Output,
            register: Some(register.to_string()),
            value_type: Some(value_type.to_string()),
            phase: self.current_phase.clone(),
            location: location.to_string(),
            properties: HashMap::new(),
        };
        
        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        
        Ok(())
    }
    
    /// Add a register node
    pub fn add_register(&mut self, name: &str, value_type: &str, location: &str) -> Result<()> {
        let node_id = format!("reg_{}", name);
        let node = DataFlowNode {
            id: node_id.clone(),
            node_type: DataFlowNodeType::Register,
            register: Some(name.to_string()),
            value_type: Some(value_type.to_string()),
            phase: self.current_phase.clone(),
            location: location.to_string(),
            properties: HashMap::new(),
        };
        
        let idx = self.graph.add_node(node);
        self.node_map.insert(node_id, idx);
        
        Ok(())
    }
    
    /// Add a data flow edge between nodes
    pub fn add_flow(&mut self, from: &str, to: &str, edge_type: DataFlowEdgeType, 
                    transformation: Option<&str>, preserves_invariant: bool) -> Result<()> {
        let from_idx = self.node_map.get(from)
            .context(format!("Source node {} not found", from))?;
        let to_idx = self.node_map.get(to)
            .context(format!("Destination node {} not found", to))?;
        
        let edge = DataFlowEdge {
            from: from.to_string(),
            to: to.to_string(),
            edge_type: edge_type.clone(),
            transformation: transformation.map(|s| s.to_string()),
            preserves_invariant,
            properties: HashMap::new(),
        };
        
        self.graph.add_edge(*from_idx, *to_idx, edge);
        
        Ok(())
    }
    
    /// Start a new phase
    pub fn start_phase(&mut self, phase_name: &str) -> Result<()> {
        // End current phase transition
        if !self.current_phase.is_empty() {
            let transition = PhaseTransition {
                phase_name: format!("{}=>{}", self.current_phase, phase_name),
                input_registers: HashSet::new(),
                output_registers: HashSet::new(),
                pre_conditions: Vec::new(),
                post_conditions: Vec::new(),
                invariants_preserved: Vec::new(),
                location: "phase_boundary".to_string(),
            };
            self.phase_transitions.push(transition);
        }
        
        self.current_phase = phase_name.to_string();
        
        // Add phase boundary node
        let boundary_id = format!("phase_{}", phase_name);
        let node = DataFlowNode {
            id: boundary_id.clone(),
            node_type: DataFlowNodeType::PhaseBoundary,
            register: None,
            value_type: None,
            phase: phase_name.to_string(),
            location: "phase_start".to_string(),
            properties: HashMap::new(),
        };
        
        let idx = self.graph.add_node(node);
        self.node_map.insert(boundary_id, idx);
        
        Ok(())
    }
    
    /// Add an invariant to track
    pub fn add_invariant(&mut self, id: &str, description: &str, applies_to: &[&str], 
                        expression: &str, verification_method: &str, phases: &[&str]) -> Result<()> {
        let invariant = Invariant {
            id: id.to_string(),
            description: description.to_string(),
            applies_to: applies_to.iter().map(|s| s.to_string()).collect(),
            expression: expression.to_string(),
            verified: false,
            verification_method: verification_method.to_string(),
            phases: phases.iter().map(|s| s.to_string()).collect(),
        };
        
        self.invariants.push(invariant);
        Ok(())
    }
    
    /// Verify invariants across phase transitions
    pub fn verify_invariants(&mut self) -> Result<()> {
        println!("Verifying invariants across phase transitions...");
        
        for invariant in &mut self.invariants {
            // Check if this invariant applies to current phase
            if invariant.phases.contains(&self.current_phase) {
                // In real implementation, this would analyze the actual code
                // For now, we'll simulate verification
                invariant.verified = self.simulate_invariant_verification(invariant);
                
                if invariant.verified {
                    println!("✓ Invariant verified: {}", invariant.description);
                } else {
                    println!("✗ Invariant NOT verified: {}", invariant.description);
                }
            }
        }
        
        Ok(())
    }
    
    /// Simulate invariant verification (mock implementation)
    fn simulate_invariant_verification(&self, invariant: &Invariant) -> bool {
        // In real implementation, this would analyze the actual data flow
        // For simulation, we'll assume most invariants are preserved
        !invariant.expression.contains("unsafe") && 
        !invariant.expression.contains("unchecked")
    }
    
    /// Analyze register flow through the workload
    pub fn analyze_register_flow(&self, register: &str) -> Result<Vec<DataFlowEdge>> {
        let mut flow_path = Vec::new();
        
        // Find all edges involving this register
        for edge in self.graph.edge_weights() {
            if edge.from.contains(register) || edge.to.contains(register) {
                flow_path.push(edge.clone());
            }
        }
        
        Ok(flow_path)
    }
    
    /// Generate visualization of the data flow graph
    pub fn generate_visualization(&self, output_path: &Path) -> Result<()> {
        println!("Generating data flow visualization...");
        
        let dot = Dot::with_config(&self.graph, &[Config::EdgeNoLabel]);
        let dot_content = format!("{:?}", dot);
        
        fs::write(output_path, dot_content)
            .context("Failed to write visualization")?;
        
        println!("Visualization saved to {}", output_path.display());
        Ok(())
    }
    
    /// Generate data flow report
    pub fn generate_report(&self, output_path: &Path) -> Result<()> {
        println!("Generating data flow analysis report...");
        
        let mut report = String::new();
        
        report.push_str("# Data Flow Analysis Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Local::now().format("%Y-%m-%d %H:%M:%S")));
        
        // Graph statistics
        report.push_str(&format!("## Graph Statistics\n\n"));
        report.push_str(&format!("- **Nodes**: {}\n", self.graph.node_count()));
        report.push_str(&format!("- **Edges**: {}\n", self.graph.edge_count()));
        report.push_str(&format!("- **Phases**: {}\n", self.phase_transitions.len() + 1));
        report.push_str(&format!("- **Invariants**: {}\n\n", self.invariants.len()));
        
        // Node breakdown
        let mut node_counts = HashMap::new();
        for node in self.graph.node_weights() {
            *node_counts.entry(node.node_type.clone()).or_insert(0) += 1;
        }
        
        report.push_str("## Node Types\n\n");
        report.push_str("| Type | Count |\n");
        report.push_str("|------|-------|\n");
        for (node_type, count) in &node_counts {
            report.push_str(&format!("| {} | {} |\n", self.format_node_type(node_type), count));
        }
        
        // Phase transitions
        report.push_str("\n## Phase Transitions\n\n");
        for (i, transition) in self.phase_transitions.iter().enumerate() {
            report.push_str(&format!("{}. **{}**\n", i + 1, transition.phase_name));
            report.push_str(&format!("   - Input registers: {}\n", transition.input_registers.len()));
            report.push_str(&format!("   - Output registers: {}\n", transition.output_registers.len()));
            report.push_str(&format!("   - Invariants preserved: {}\n\n", transition.invariants_preserved.len()));
        }
        
        // Invariant verification
        report.push_str("## Invariant Verification\n\n");
        report.push_str("| ID | Description | Verified | Method |\n");
        report.push_str("|----|-------------|----------|--------|\n");
        for invariant in &self.invariants {
            let verified_str = if invariant.verified { "✓ Yes" } else { "✗ No" };
            report.push_str(&format!("| {} | {} | {} | {} |\n",
                invariant.id, invariant.description, verified_str, invariant.verification_method));
        }
        
        // Register flow analysis
        report.push_str("\n## Register Flow Analysis\n\n");
        let registers: HashSet<_> = self.graph.node_weights()
            .filter_map(|n| n.register.as_ref())
            .collect();
        
        for register in registers {
            report.push_str(&format!("### Register: {}\n\n", register));
            
            let flow = self.analyze_register_flow(register)?;
            if flow.is_empty() {
                report.push_str("- No flow data available\n\n");
            } else {
                report.push_str("| From | To | Type | Preserves Invariant |\n");
                report.push_str("|------|----|------|---------------------|\n");
                
                for edge in flow {
                    let preserves_str = if edge.preserves_invariant { "✓" } else { "✗" };
                    report.push_str(&format!("| {} | {} | {} | {} |\n",
                        edge.from, edge.to, self.format_edge_type(&edge.edge_type), preserves_str));
                }
                report.push_str("\n");
            }
        }
        
        fs::write(output_path, report)
            .context("Failed to write data flow report")?;
        
        println!("Data flow report generated at {}", output_path.display());
        Ok(())
    }
    
    /// Export data flow graph to JSON
    pub fn export_to_json(&self, output_path: &Path) -> Result<()> {
        let graph_data = DataFlowGraph {
            nodes: self.graph.node_weights().cloned().collect(),
            edges: self.graph.edge_weights().cloned().collect(),
            phases: self.phase_transitions.clone(),
            invariants: self.invariants.clone(),
        };
        
        let json = serde_json::to_string_pretty(&graph_data)
            .context("Failed to serialize data flow graph")?;
        
        fs::write(output_path, json)
            .context("Failed to write JSON export")?;
        
        Ok(())
    }
    
    /// Helper to format node type
    fn format_node_type(&self, node_type: &DataFlowNodeType) -> &str {
        match node_type {
            DataFlowNodeType::Input => "Input",
            DataFlowNodeType::Output => "Output",
            DataFlowNodeType::Register => "Register",
            DataFlowNodeType::Memory => "Memory",
            DataFlowNodeType::Operation => "Operation",
            DataFlowNodeType::Constant => "Constant",
            DataFlowNodeType::PhaseBoundary => "Phase Boundary",
        }
    }
    
    /// Helper to format edge type
    fn format_edge_type(&self, edge_type: &DataFlowEdgeType) -> &str {
        match edge_type {
            DataFlowEdgeType::DirectFlow => "Direct Flow",
            DataFlowEdgeType::Transformation => "Transformation",
            DataFlowEdgeType::Copy => "Copy",
            DataFlowEdgeType::Move => "Move",
            DataFlowEdgeType::Load => "Load",
            DataFlowEdgeType::Store => "Store",
            DataFlowEdgeType::PhaseTransition => "Phase Transition",
        }
    }
}

/// Workload data flow analyzer that integrates with the compiler
pub struct WorkloadDataFlowAnalyzer {
    analyzer: DataFlowAnalyzer,
    workload_name: String,
    source_files: Vec<PathBuf>,
}

impl WorkloadDataFlowAnalyzer {
    /// Create a new workload data flow analyzer
    pub fn new(workload_name: &str) -> Self {
        Self {
            analyzer: DataFlowAnalyzer::new("initial"),
            workload_name: workload_name.to_string(),
            source_files: Vec::new(),
        }
    }
    
    /// Add source files to analyze
    pub fn add_source_files(&mut self, files: &[PathBuf]) -> Result<()> {
        self.source_files.extend_from_slice(files);
        Ok(())
    }
    
    /// Analyze Rust source code for data flow (simplified mock)
    pub fn analyze_rust_source(&mut self) -> Result<()> {
        println!("Analyzing Rust source for data flow in {}...", self.workload_name);
        
        // In real implementation, this would parse Rust code using syn/quote
        // For now, we'll simulate a typical workload analysis
        
        // Start with initial phase
        self.analyzer.start_phase("input_validation")?;
        
        // Add input registers
        self.analyzer.add_input("transaction_input", "tx_register", "Transaction", "src/validator.rs:42")?;
        self.analyzer.add_input("signature_input", "sig_register", "Signature", "src/validator.rs:43")?;
        
        // Add registers
        self.analyzer.add_register("tx_register", "Transaction", "src/validator.rs:50")?;
        self.analyzer.add_register("sig_register", "Signature", "src/validator.rs:51")?;
        self.analyzer.add_register("temp_hash", "Hash", "src/validator.rs:55")?;
        
        // Add data flows
        self.analyzer.add_flow(
            "input_transaction_input", "reg_tx_register",
            DataFlowEdgeType::DirectFlow, None, true
        )?;
        self.analyzer.add_flow(
            "input_signature_input", "reg_sig_register",
            DataFlowEdgeType::DirectFlow, None, true
        )?;
        
        // Transition to processing phase
        self.analyzer.start_phase("processing")?;
        
        // Add processing registers
        self.analyzer.add_register("processed_tx", "ProcessedTransaction", "src/processor.rs:18")?;
        self.analyzer.add_register("verification_result", "bool", "src/processor.rs:22")?;
        
        // Add processing flows
        self.analyzer.add_flow(
            "reg_tx_register", "reg_processed_tx",
            DataFlowEdgeType::Transformation, Some("process_transaction"), true
        )?;
        self.analyzer.add_flow(
            "reg_sig_register", "reg_verification_result",
            DataFlowEdgeType::Transformation, Some("verify_signature"), true
        )?;
        
        // Add invariants
        self.analyzer.add_invariant(
            "invariant_1",
            "Transaction hash must match signature",
            &["tx_register", "sig_register", "temp_hash"],
            "tx_register.hash() == temp_hash && verify(sig_register, temp_hash)",
            "cryptographic_verification",
            &["input_validation", "processing"]
        )?;
        
        self.analyzer.add_invariant(
            "invariant_2",
            "Processed transaction must maintain original data",
            &["tx_register", "processed_tx"],
            "processed_tx.original_data == tx_register.data",
            "data_integrity_check",
            &["processing"]
        )?;
        
        // Transition to output phase
        self.analyzer.start_phase("output")?;
        
        // Add output
        self.analyzer.add_output("transaction_output", "processed_tx", "ProcessedTransaction", "src/output.rs:15")?;
        self.analyzer.add_output("verification_output", "verification_result", "bool", "src/output.rs:16")?;
        
        // Add final flows
        self.analyzer.add_flow(
            "reg_processed_tx", "output_transaction_output",
            DataFlowEdgeType::DirectFlow, None, true
        )?;
        self.analyzer.add_flow(
            "reg_verification_result", "output_verification_output",
            DataFlowEdgeType::DirectFlow, None, true
        )?;
        
        // Verify invariants
        self.analyzer.verify_invariants()?;
        
        Ok(())
    }
    
    /// Generate complete data flow documentation
    pub fn generate_documentation(&self, output_dir: &Path) -> Result<()> {
        fs::create_dir_all(output_dir)
            .context("Failed to create output directory")?;
        
        // Generate visualization
        let viz_path = output_dir.join("data_flow.dot");
        self.analyzer.generate_visualization(&viz_path)?;
        
        // Generate report
        let report_path = output_dir.join("data_flow_report.md");
        self.analyzer.generate_report(&report_path)?;
        
        // Export JSON
        let json_path = output_dir.join("data_flow.json");
        self.analyzer.export_to_json(&json_path)?;
        
        println!("Documentation generated in {}", output_dir.display());
        Ok(())
    }
    
    /// Get the underlying analyzer
    pub fn analyzer(&self) -> &DataFlowAnalyzer {
        &self.analyzer
    }
}