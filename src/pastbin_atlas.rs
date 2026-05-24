use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

#[cfg(feature = "pastbin")]
use std::sync::Arc;
#[cfg(feature = "pastbin")]
use chrono::{DateTime, Utc};
#[cfg(feature = "pastbin")]
use uuid::Uuid;
#[cfg(feature = "pastbin")]
use crate::group_atlas::FiniteSimpleGroup;

/// Generate random ID for views
fn generate_random_id() -> String {
    use rand::Rng;
    let rng = rand::thread_rng();
    rng.sample_iter(&rand::distributions::Alphanumeric)
        .take(8)
        .map(char::from)
        .collect()
}

/// View metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewMetadata {
    pub created_by: String,
    pub created_at: String,
    pub tags: Vec<String>,
    pub is_public: bool,
    pub view_id: String,
    pub parent_view: Option<String>,
}

/// Atlas view configuration for custom compositions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasView {
    pub name: String,
    pub description: String,
    pub groups: Vec<GroupSelection>,
    pub composition_rules: CompositionRules,
    pub visualization: VisualizationConfig,
    pub metadata: ViewMetadata,
}

/// Selection criteria for groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupSelection {
    pub family_filter: Option<String>,
    pub order_range: Option<OrderRange>,
    pub complexity_range: Option<ComplexityRange>,
    pub specific_groups: Vec<String>,
    pub exclude_groups: Vec<String>,
}

/// Order range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRange {
    pub min: Option<u64>,
    pub max: Option<u64>,
}

/// Complexity range filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityRange {
    pub min: u32,
    pub max: u32,
}

/// Composition rules for view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionRules {
    pub include_direct_products: bool,
    pub include_semidirect_products: bool,
    pub include_wreath_products: bool,
    pub max_composition_size: usize,
    pub composition_strategy: CompositionStrategy,
}

/// Composition strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompositionStrategy {
    AllPossible,
    Balanced,
    Hierarchical,
    Custom,
}

/// Visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub layout: LayoutType,
    pub color_scheme: ColorScheme,
    pub show_connections: bool,
    pub show_labels: bool,
    pub tile_size: Option<u32>,
    pub spacing: u32,
}

/// Layout type for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutType {
    Grid,
    Hierarchical,
    ForceDirected,
    Circular,
    Spiral,
}

/// Color scheme for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    FamilyBased,
    OrderBased,
    ComplexityBased,
    Custom,
}

/// Result of composition generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionResult {
    pub view_name: String,
    pub selected_groups: Vec<String>,
    pub compositions: Vec<Composition>,
    pub visualization_config: VisualizationConfig,
}

/// Group composition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Composition {
    pub name: String,
    pub component_groups: Vec<String>,
    pub composition_type: String,
    pub order: u64,
    pub complexity: u32,
}

/// Summary of atlas views
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasViewSummary {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub created_at: String,
    pub tags: Vec<String>,
}

/// Modifications for views
#[derive(Debug, Clone)]
pub struct ViewModifications {
    pub name: Option<String>,
    pub description: Option<String>,
    pub groups: Option<Vec<GroupSelection>>,
    pub composition_rules: Option<CompositionRules>,
    pub visualization: Option<VisualizationConfig>,
    pub tags: Option<Vec<String>>,
}

/// Atlas view composer for creating and managing custom atlas views
#[derive(Debug, Clone)]
pub struct AtlasViewComposer {
    views: HashMap<String, AtlasView>,
}

impl AtlasViewComposer {
    /// Create a new atlas view composer
    pub fn new() -> Self {
        Self {
            views: HashMap::new(),
        }
    }

    /// Create a new view
    pub fn create_view(
        &mut self,
        name: String,
        description: String,
        groups: Vec<GroupSelection>,
        composition_rules: CompositionRules,
        visualization: VisualizationConfig,
    ) -> String {
        let view_id = generate_random_id();
        let metadata = ViewMetadata {
            created_by: "user".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            tags: Vec::new(),
            is_public: false,
            view_id: view_id.clone(),
            parent_view: None,
        };

        let view = AtlasView {
            name,
            description,
            groups,
            composition_rules,
            visualization,
            metadata,
        };

        self.views.insert(view_id.clone(), view);
        view_id
    }

    /// Get a view by ID
    pub fn get_view(&self, view_id: &str) -> Option<&AtlasView> {
        self.views.get(view_id)
    }

    /// Get all views
    pub fn get_all_views(&self) -> Vec<&AtlasView> {
        self.views.values().collect()
    }

    /// Modify an existing view
    pub fn modify_view(&mut self, view_id: &str, modifications: ViewModifications) -> Result<(), String> {
        if let Some(view) = self.views.get_mut(view_id) {
            if let Some(name) = modifications.name {
                view.name = name;
            }
            if let Some(description) = modifications.description {
                view.description = description;
            }
            if let Some(groups) = modifications.groups {
                view.groups = groups;
            }
            if let Some(composition_rules) = modifications.composition_rules {
                view.composition_rules = composition_rules;
            }
            if let Some(visualization) = modifications.visualization {
                view.visualization = visualization;
            }
            if let Some(tags) = modifications.tags {
                view.metadata.tags = tags;
            }
            Ok(())
        } else {
            Err("View not found".to_string())
        }
    }

    /// Delete a view
    pub fn delete_view(&mut self, view_id: &str) -> bool {
        self.views.remove(view_id).is_some()
    }

    /// Generate composition from view
    pub fn generate_composition(&self, view: &AtlasView) -> CompositionResult {
        // This is a simplified implementation - in a real version, this would
        // actually compute compositions based on the group selections and rules
        CompositionResult {
            view_name: view.name.clone(),
            selected_groups: Vec::new(), // Would be populated based on group selections
            compositions: Vec::new(),    // Would be populated based on composition rules
            visualization_config: view.visualization.clone(),
        }
    }
}

/// Pastbin service for sharing views
#[cfg(feature = "pastbin")]
#[derive(Debug)]
pub struct PastbinService {
    api_key: Option<String>,
}

#[cfg(feature = "pastbin")]
impl PastbinService {
    /// Create a new pastbin service
    pub fn new(api_key: Option<String>) -> Self {
        Self { api_key }
    }

    /// Upload a view to pastbin
    pub async fn upload_view(&self, view: &AtlasView) -> Result<String, String> {
        // This would make an actual HTTP request to pastbin
        // For now, we'll just return a mock paste ID
        println!("📤 Uploading view '{}' to pastbin", view.name);
        Ok(format!("paste_{}", generate_random_id()))
    }

    /// Download a view from pastbin
    pub async fn download_view(&self, paste_id: &str) -> Result<AtlasView, String> {
        // This would make an actual HTTP request to pastbin
        // For now, we'll return an error
        Err("Pastbin integration not implemented in this demo".to_string())
    }
}

#[cfg(not(feature = "pastbin"))]
/// Mock pastbin service for when pastbin feature is not enabled
#[derive(Debug)]
pub struct PastbinService {
    _phantom: std::marker::PhantomData<()>,
}

#[cfg(not(feature = "pastbin"))]
impl PastbinService {
    /// Create a new pastbin service (mock version)
    pub fn new(_api_key: Option<String>) -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    /// Mock upload - returns error since pastbin feature not enabled
    pub async fn upload_view(&self, _view: &AtlasView) -> Result<String, String> {
        Err("Pastbin integration requires compiling with 'pastbin' feature".to_string())
    }

    /// Mock download - returns error since pastbin feature not enabled
    pub async fn download_view(&self, _paste_id: &str) -> Result<AtlasView, String> {
        Err("Pastbin integration requires compiling with 'pastbin' feature".to_string())
    }
}