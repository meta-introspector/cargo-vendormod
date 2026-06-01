use clap::{Parser, Subcommand};
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::Path;

use cargo_vendormod::pastbin_atlas::{
    AtlasViewComposer, AtlasView, GroupSelection, CompositionRules, VisualizationConfig,
    OrderRange, ComplexityRange, LayoutType, ColorScheme, CompositionStrategy,
    ViewModifications
};

/// Atlas view composer with pastbin integration
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct AtlasComposerArgs {
    /// Output directory for generated files
    #[arg(long, default_value = "./composer_output")]
    output_dir: String,

    /// Subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Available commands
#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new atlas view
    Create {
        /// View name
        name: String,
        /// View description
        #[arg(long)]
        description: String,
        /// Filter by family (optional)
        #[arg(long)]
        family: Option<String>,
        /// Minimum order (optional)
        #[arg(long)]
        min_order: Option<u64>,
        /// Maximum order (optional)
        #[arg(long)]
        max_order: Option<u64>,
        /// Include direct products
        #[arg(long, default_value = "true")]
        direct_products: bool,
        /// Include semidirect products
        #[arg(long, default_value = "true")]
        semidirect_products: bool,
        /// Layout type
        #[arg(long, default_value = "grid")]
        layout: String,
        /// Make view public
        #[arg(long)]
        public: bool,
    },

    /// List all local views
    List {
        /// Show detailed view information
        #[arg(long)]
        detailed: bool,
    },

    /// Modify an existing view
    Modify {
        /// View ID to modify
        view_id: String,
        /// New name (optional)
        #[arg(long)]
        name: Option<String>,
        /// New description (optional)
        #[arg(long)]
        description: Option<String>,
    },

    /// Share a view via pastbin
    Share {
        /// View ID to share
        view_id: String,
        /// Make public
        #[arg(long)]
        public: bool,
    },

    /// Import a view from pastbin
    Import {
        /// Pastbin paste ID
        paste_id: String,
    },

    /// Delete a local view
    Delete {
        /// View ID to delete
        view_id: String,
    },

    /// Generate composition from view
    Generate {
        /// View ID to generate from
        view_id: String,
    },

    /// Browse public views
    Browse {
        /// Filter by tag (optional)
        #[arg(long)]
        tag: Option<String>,
    },

    /// Export view to file
    Export {
        /// View ID to export
        view_id: String,
        /// Output file path
        output_path: String,
    },

    /// Import view from file
    ImportFile {
        /// Input file path
        input_path: String,
    },

    /// Create view from template
    Template {
        /// Template name
        template_name: String,
        /// New view name
        new_name: String,
        /// New view description
        #[arg(long)]
        description: String,
    },
}

/// Template definitions for common views
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ViewTemplate {
    name: String,
    description: String,
    groups: Vec<GroupSelection>,
    composition_rules: CompositionRules,
    visualization: VisualizationConfig,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AtlasComposerArgs::parse();

    // Create output directory
    fs::create_dir_all(&args.output_dir)?;

    // Initialize composer
    let mut composer = AtlasViewComposer::new();

    // Load existing views if any
    load_existing_views(&mut composer, &args.output_dir);

    match args.command {
        Commands::Create {
            name,
            description,
            family,
            min_order,
            max_order,
            direct_products,
            semidirect_products,
            layout,
            public,
        } => {
            create_view(
                &mut composer,
                name,
                description,
                family,
                min_order,
                max_order,
                direct_products,
                semidirect_products,
                layout,
                public,
                &args.output_dir,
            )?;
        }

        Commands::List { detailed } => {
            list_views(&composer, detailed)?;
        }

        Commands::Modify {
            view_id,
            name,
            description,
        } => {
            modify_view(&mut composer, &view_id, name, description)?;
        }

Commands::Share { view_id, public: _ } => {
             share_view(&mut composer, &view_id, false)?;
         }

        Commands::Import { paste_id } => {
             import_view(&mut composer, &paste_id)?;
         }

        Commands::Delete { view_id } => {
             delete_view(&mut composer, &view_id)?;
         }

        Commands::Generate { view_id } => {
             generate_composition(&composer, &view_id, &args.output_dir)?;
         }

        Commands::Browse { tag } => {
             browse_public_views(&composer, tag)?;
         }

        Commands::Export { view_id, output_path } => {
            export_view(&composer, &view_id, &output_path)?;
        }

        Commands::ImportFile { input_path } => {
            import_view_from_file(&mut composer, &input_path)?;
        }

        Commands::Template {
            template_name,
            new_name,
            description,
        } => {
            create_view_from_template(&mut composer, &template_name, new_name, description)?;
        }
    }

    // Save updated views
    save_views(&composer, &args.output_dir);

    Ok(())
}

/// Create a new view
fn create_view(
    composer: &mut AtlasViewComposer,
    name: String,
    description: String,
    family: Option<String>,
    min_order: Option<u64>,
    max_order: Option<u64>,
    direct_products: bool,
    semidirect_products: bool,
    layout: String,
    _public: bool,
    output_dir: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎨 Creating new view: {}", name);

    // Create group selection
    let group_selection = GroupSelection {
        family_filter: family,
        order_range: Some(OrderRange { min: min_order, max: max_order }),
        complexity_range: None,
        specific_groups: Vec::new(),
        exclude_groups: Vec::new(),
    };

    // Create composition rules
    let composition_rules = CompositionRules {
        include_direct_products: direct_products,
        include_semidirect_products: semidirect_products,
        include_wreath_products: false,
        max_composition_size: 3,
        composition_strategy: CompositionStrategy::Balanced,
    };

    // Parse layout
    let layout = match layout.as_str() {
        "grid" => LayoutType::Grid,
        "hierarchical" => LayoutType::Hierarchical,
        "circular" => LayoutType::Circular,
        "spiral" => LayoutType::Spiral,
        _ => LayoutType::Grid,
    };

    // Create visualization config
    let visualization = VisualizationConfig {
        layout,
        color_scheme: ColorScheme::FamilyBased,
        show_connections: true,
        show_labels: true,
        tile_size: None,
        spacing: 10,
    };

    // Create view
    let view_id = composer.create_view(name, description, vec![group_selection], composition_rules, visualization);

    println!("✅ View created with ID: {}", view_id);

    // Generate initial composition
    if let Some(view) = composer.get_view(&view_id) {
        let result = composer.generate_composition(view);
        save_composition_result(&result, output_dir, &view_id)?;
    }

    Ok(())
}

/// List all views
fn list_views(composer: &AtlasViewComposer, detailed: bool) -> Result<(), Box<dyn std::error::Error>> {
    let views = composer.get_all_views();
    
    if views.is_empty() {
        println!("📋 No local views found");
        return Ok(());
    }

    println!("📋 Atlas Views ({} total):", views.len());
    println!("{}", "=".repeat(50));

    for view in views {
        if detailed {
            println!("📄 View: {}", view.name);
            println!("   ID: {}", view.metadata.view_id);
            println!("   Description: {}", view.description);
            println!("   Created by: {}", view.metadata.created_by);
            println!("   Created at: {}", view.metadata.created_at);
            println!("   Tags: {:?}", view.metadata.tags);
            println!("   Groups: {} selections", view.groups.len());
            println!("   Layout: {:?}", view.visualization.layout);
            println!();
        } else {
            println!("📄 {} ({})", view.name, view.metadata.view_id);
        }
    }

    Ok(())
}

/// Modify existing view
fn modify_view(
    composer: &mut AtlasViewComposer,
    view_id: &str,
    name: Option<String>,
    description: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Modifying view: {}", view_id);

    let modifications = ViewModifications {
        name,
        description,
        groups: None,
        composition_rules: None,
        visualization: None,
        tags: None,
    };

    match composer.modify_view(view_id, modifications) {
        Ok(()) => {
            println!("✅ View modified successfully");
        }
        Err(e) => {
            println!("❌ Error modifying view: {}", e);
        }
    }

    Ok(())
}

fn share_view(_composer: &mut AtlasViewComposer, view_id: &str, public: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("📤 Sharing view: {} (public: {})", view_id, public);
    println!("⚠️ Note: Pastbin integration requires compiling with 'pastbin' feature");
    println!("   Use: cargo run --features pastbin --bin atlas_composer share {} --public", view_id);
    Ok(())
}

/// Import view from pastbin
fn import_view(_composer: &mut AtlasViewComposer, paste_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📥 Importing view from pastbin: {}", paste_id);
    println!("⚠️ Note: Pastbin integration requires compiling with 'pastbin' feature");
    println!("   Use: cargo run --features pastbin --bin atlas_composer import {}", paste_id);
    Ok(())
}

/// Delete view
fn delete_view(composer: &mut AtlasViewComposer, view_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🗑️ Deleting view: {}", view_id);

    if composer.delete_view(view_id) {
        println!("✅ View deleted successfully");
    } else {
        println!("❌ View not found");
    }

    Ok(())
}

/// Generate composition from view
fn generate_composition(composer: &AtlasViewComposer, view_id: &str, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Generating composition for view: {}", view_id);

    if let Some(view) = composer.get_view(view_id) {
        let result = composer.generate_composition(view);
        save_composition_result(&result, output_dir, view_id)?;
        println!("✅ Composition generated and saved");
    } else {
        println!("❌ View not found");
    }

    Ok(())
}

/// Browse public views
fn browse_public_views(_composer: &AtlasViewComposer, tag: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🌐 Browsing public views");
    println!("⚠️ Note: Pastbin integration requires compiling with 'pastbin' feature");
    println!("   Use: cargo run --features pastbin --bin atlas_composer browse --tag {}", tag.as_deref().unwrap_or("all"));
    Ok(())
}

/// Export view to file
fn export_view(composer: &AtlasViewComposer, view_id: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("💾 Exporting view to: {}", output_path);

    if let Some(view) = composer.get_view(view_id) {
        let json_data = serde_json::to_string_pretty(view)?;
        fs::write(output_path, json_data)?;
        println!("✅ View exported successfully");
    } else {
        println!("❌ View not found");
    }

    Ok(())
}

/// Import view from file
fn import_view_from_file(composer: &mut AtlasViewComposer, input_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("📥 Importing view from file: {}", input_path);

    let content = fs::read_to_string(input_path)?;
    let view: AtlasView = serde_json::from_str(&content)?;

    let view_id = composer.create_view(
        view.name.clone(),
        view.description.clone(),
        view.groups.clone(),
        view.composition_rules.clone(),
        view.visualization.clone(),
    );

    println!("✅ View imported successfully with ID: {}", view_id);

    Ok(())
}

/// Create view from template
fn create_view_from_template(
    composer: &mut AtlasViewComposer,
    template_name: &str,
    new_name: String,
    description: String,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Creating view from template: {}", template_name);

    let template = get_template(template_name)?;
    
    let view_id = composer.create_view(
        new_name,
        description,
        template.groups,
        template.composition_rules,
        template.visualization,
    );

    println!("✅ View created from template with ID: {}", view_id);

    Ok(())
}

/// Get template by name
fn get_template(template_name: &str) -> Result<ViewTemplate, Box<dyn std::error::Error>> {
    match template_name {
        "beginner" => Ok(ViewTemplate {
            name: "Beginner Explorer".to_string(),
            description: "Simple view for learning basic groups".to_string(),
            groups: vec![GroupSelection {
                family_filter: Some("Cyclic".to_string()),
                order_range: Some(OrderRange { min: Some(2), max: Some(31) }),
                complexity_range: Some(ComplexityRange { min: 1, max: 1 }),
                specific_groups: Vec::new(),
                exclude_groups: Vec::new(),
            }],
            composition_rules: CompositionRules {
                include_direct_products: true,
                include_semidirect_products: false,
                include_wreath_products: false,
                max_composition_size: 2,
                composition_strategy: CompositionStrategy::Balanced,
            },
            visualization: VisualizationConfig {
                layout: LayoutType::Grid,
                color_scheme: ColorScheme::FamilyBased,
                show_connections: true,
                show_labels: true,
                tile_size: Some(80),
                spacing: 15,
            },
        }),
        "advanced" => Ok(ViewTemplate {
            name: "Advanced Research".to_string(),
            description: "Complex view for advanced group theory".to_string(),
            groups: vec![
                GroupSelection {
                    family_filter: Some("LieType".to_string()),
                    order_range: None,
                    complexity_range: Some(ComplexityRange { min: 3, max: 5 }),
                    specific_groups: Vec::new(),
                    exclude_groups: Vec::new(),
                },
                GroupSelection {
                    family_filter: Some("Sporadic".to_string()),
                    order_range: None,
                    complexity_range: Some(ComplexityRange { min: 4, max: 4 }),
                    specific_groups: Vec::new(),
                    exclude_groups: Vec::new(),
                },
            ],
            composition_rules: CompositionRules {
                include_direct_products: true,
                include_semidirect_products: true,
                include_wreath_products: true,
                max_composition_size: 5,
                composition_strategy: CompositionStrategy::Hierarchical,
            },
            visualization: VisualizationConfig {
                layout: LayoutType::Hierarchical,
                color_scheme: ColorScheme::ComplexityBased,
                show_connections: true,
                show_labels: true,
                tile_size: Some(100),
                spacing: 20,
            },
        }),
        "educational" => Ok(ViewTemplate {
            name: "Educational Focus".to_string(),
            description: "View optimized for teaching and learning".to_string(),
            groups: vec![
                GroupSelection {
                    family_filter: Some("Alternating".to_string()),
                    order_range: Some(OrderRange { min: Some(60), max: Some(10000) }),
                    complexity_range: Some(ComplexityRange { min: 2, max: 3 }),
                    specific_groups: Vec::new(),
                    exclude_groups: Vec::new(),
                },
                GroupSelection {
                    family_filter: Some("LieType".to_string()),
                    order_range: Some(OrderRange { min: Some(168), max: Some(10000) }),
                    complexity_range: Some(ComplexityRange { min: 2, max: 3 }),
                    specific_groups: Vec::new(),
                    exclude_groups: Vec::new(),
                },
            ],
            composition_rules: CompositionRules {
                include_direct_products: true,
                include_semidirect_products: false,
                include_wreath_products: false,
                max_composition_size: 3,
                composition_strategy: CompositionStrategy::Balanced,
            },
            visualization: VisualizationConfig {
                layout: LayoutType::Grid,
                color_scheme: ColorScheme::FamilyBased,
                show_connections: true,
                show_labels: true,
                tile_size: Some(90),
                spacing: 12,
            },
        }),
        _ => Err(format!("Unknown template: {}", template_name).into()),
    }
}

/// Save composition result
fn save_composition_result(result: &cargo_vendormod::pastbin_atlas::CompositionResult, output_dir: &str, view_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = Path::new(output_dir).join(format!("composition_{}.json", view_id));
    let json_data = serde_json::to_string_pretty(result)?;
    fs::write(&output_path, json_data)?;
    Ok(())
}

/// Load existing views
fn load_existing_views(_composer: &mut AtlasViewComposer, _output_dir: &str) {
    // This is now handled by the composer's built-in storage
    println!("📂 Views loaded from storage");
}

/// Save views
fn save_views(_composer: &AtlasViewComposer, output_dir: &str) {
    let views_dir = Path::new(output_dir).join("views");
    fs::create_dir_all(&views_dir).unwrap();
    
    // Views are now saved to the composer's built-in storage
    println!("💾 Views saved to storage");
}