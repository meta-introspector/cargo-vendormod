use std::fs;
use std::path::Path;
use clap::Parser;
use serde_json;
use num_bigint::BigUint;

use cargo_vendormod::group_atlas::{GroupAtlas, GroupFamily, ClassicalLieType, TwistedLieType, SporadicGroup, n};
use cargo_vendormod::visualization::{AtlasRenderer, AtlasConfig};

/// Atlas generation and visualization tool for finite simple groups
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct AtlasArgs {
    /// Output directory for generated files
    #[arg(long, default_value = "./atlas_output")]
    output_dir: String,

    /// Atlas configuration file (JSON)
    #[arg(long)]
    config_file: Option<String>,

    /// Generate hierarchical view instead of grid layout
    #[arg(long)]
    hierarchical: bool,

    /// Generate interactive HTML view
    #[arg(long)]
    interactive: bool,

    /// Generate SVG only
    #[arg(long)]
    svg_only: bool,

    /// Generate JSON data only
    #[arg(long)]
    json_only: bool,

    /// Show statistics only
    #[arg(long)]
    stats_only: bool,

    /// Generate compositions (direct products)
    #[arg(long)]
    generate_compositions: bool,

    /// Maximum number of groups to include (for testing)
    #[arg(long)]
    max_groups: Option<usize>,

    /// Filter groups by family
    #[arg(long)]
    family_filter: Option<String>,

    /// Filter groups by minimum order
    #[arg(long)]
    min_order: Option<u64>,

    /// Filter groups by maximum order
    #[arg(long)]
    max_order: Option<u64>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = AtlasArgs::parse();

    // Create output directory
    fs::create_dir_all(&args.output_dir)?;

    // Load or create atlas configuration
    let config = match args.config_file {
        Some(path) => {
            let config_content = fs::read_to_string(&path)?;
            serde_json::from_str(&config_content)?
        }
        None => AtlasConfig::default(),
    };

    // Create atlas
    let mut atlas = GroupAtlas::new();

    // Apply filters if specified
    if let Some(max_groups) = args.max_groups {
        atlas.groups = atlas.groups.into_iter().take(max_groups).collect();
    }

    if let Some(family_filter) = args.family_filter {
        let target_family = parse_family_filter(&family_filter)?;
        atlas.groups = atlas.groups
            .into_iter()
            .filter(|g| g.family == target_family)
            .collect();
    }

    if let Some(min_order) = args.min_order {
        atlas.groups = atlas.groups
            .into_iter()
            .filter(|g| g.order >= BigUint::from(min_order as u64))
            .collect();
    }

    if let Some(max_order) = args.max_order {
        atlas.groups = atlas.groups
            .into_iter()
            .filter(|g| g.order <= BigUint::from(max_order as u64))
            .collect();
    }

    // Generate compositions if requested
    if args.generate_compositions {
        atlas.generate_all_direct_products();
    }

    // Create renderer
    let mut renderer = AtlasRenderer::new(config);

    // Generate tiles
    renderer.generate_tiles(&atlas);

    // Generate composition tiles
    if args.generate_compositions {
        renderer.generate_composition_tiles(&atlas);
    }

    // Generate output based on requested formats
    if args.stats_only {
        generate_statistics(&renderer, &atlas, &args.output_dir)?;
    } else if args.json_only {
        generate_json_output(&renderer, &atlas, &args.output_dir)?;
    } else if args.svg_only {
        generate_svg_output(&mut renderer, &atlas, &args.output_dir, args.hierarchical)?;
    } else if args.interactive {
        generate_interactive_output(&renderer, &atlas, &args.output_dir)?;
    } else {
        // Generate all outputs
        generate_all_outputs(&mut renderer, &atlas, &args.output_dir, args.hierarchical)?;
    }

    println!("✅ Atlas generation completed successfully!");
    println!("📁 Output directory: {}", args.output_dir);
    
    if !args.stats_only {
        println!("📊 Groups processed: {}", atlas.groups.len());
        println!("🔗 Compositions generated: {}", atlas.compositions.len());
    }

    Ok(())
}

/// Parse family filter string to GroupFamily enum
fn parse_family_filter(filter: &str) -> Result<GroupFamily, Box<dyn std::error::Error>> {
    match filter.to_lowercase().as_str() {
        "cyclic" => Ok(GroupFamily::Cyclic),
        "alternating" => Ok(GroupFamily::Alternating),
        "lie" | "lietype" => Ok(GroupFamily::LieType(ClassicalLieType::An(
            n { value: 1, field_size: 2 }
        ))),
        "sporadic" => Ok(GroupFamily::Sporadic(SporadicGroup::Mathieu11)),
        "twisted" | "twistedlie" => Ok(GroupFamily::TwistedLieType(TwistedLieType::TwoAn(
            n { value: 1, field_size: 2 }
        ))),
        _ => Err(format!("Unknown family filter: {}", filter).into()),
    }
}

/// Generate statistics output
fn generate_statistics(renderer: &AtlasRenderer, atlas: &GroupAtlas, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let stats = renderer.export_atlas_data(atlas);
    let stats_path = Path::new(output_dir).join("statistics.json");
    
    fs::write(&stats_path, stats)?;
    println!("📈 Statistics saved to: {}", stats_path.display());
    
    // Print summary to console
    print_summary_stats(atlas);
    
    Ok(())
}

/// Generate JSON output
fn generate_json_output(renderer: &AtlasRenderer, atlas: &GroupAtlas, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let json_data = renderer.export_atlas_data(atlas);
    let json_path = Path::new(output_dir).join("atlas_data.json");
    
    fs::write(&json_path, json_data)?;
    println!("📄 Atlas data saved to: {}", json_path.display());
    
    Ok(())
}

/// Generate SVG output
fn generate_svg_output(renderer: &mut AtlasRenderer, atlas: &GroupAtlas, output_dir: &str, hierarchical: bool) -> Result<(), Box<dyn std::error::Error>> {
    let svg_content = if hierarchical {
        renderer.generate_hierarchical_view(atlas)
    } else {
        renderer.render_svg()
    };
    
    let svg_path = Path::new(output_dir).join(if hierarchical {
        "atlas_hierarchical.svg"
    } else {
        "atlas_grid.svg"
    });
    
    fs::write(&svg_path, svg_content)?;
    println!("🎨 SVG saved to: {}", svg_path.display());
    
    Ok(())
}

/// Generate interactive HTML output
fn generate_interactive_output(renderer: &AtlasRenderer, atlas: &GroupAtlas, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let html_content = renderer.get_interactive_html(atlas);
    let html_path = Path::new(output_dir).join("atlas_interactive.html");
    
    fs::write(&html_path, html_content)?;
    println!("🌐 Interactive HTML saved to: {}", html_path.display());
    
    Ok(())
}

/// Generate all output formats
fn generate_all_outputs(renderer: &mut AtlasRenderer, atlas: &GroupAtlas, output_dir: &str, hierarchical: bool) -> Result<(), Box<dyn std::error::Error>> {
    // Generate statistics
    generate_statistics(renderer, atlas, output_dir)?;
    
    // Generate JSON
    generate_json_output(renderer, atlas, output_dir)?;
    
    // Generate SVG
    generate_svg_output(renderer, atlas, output_dir, hierarchical)?;
    
    // Generate interactive HTML
    generate_interactive_output(renderer, atlas, output_dir)?;
    
    // Generate README
    generate_readme(atlas, output_dir)?;
    
    Ok(())
}

/// Generate README file
fn generate_readme(atlas: &GroupAtlas, output_dir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let readme_content = format!(
        r#"# Finite Simple Groups Atlas

This directory contains a comprehensive atlas of finite simple groups visualized as interactive tiles.

## 📊 Statistics

- **Total Groups**: {}
- **Total Compositions**: {}
- **Group Families**: {}
- **Order Range**: {} to {}

## 📁 Files Generated

- `atlas_data.json` - Complete atlas data in JSON format
- `atlas_grid.svg` - Grid layout visualization
- `atlas_hierarchical.svg` - Hierarchical view by complexity
- `atlas_interactive.html` - Interactive web-based visualization
- `statistics.json` - Detailed statistics and metrics
- `README.md` - This file

## 🎯 Group Families

### Cyclic Groups ({} groups)
Groups of prime order: C₂, C₃, C₅, C₇, C₁₁, C₁₃, C₁₇, C₁₉, C₂₃, C₂₉, C₃₁

### Alternating Groups ({} groups)
Alternating groups Aₙ for n ≥ 5: A₅, A₆, A₇, A₈, A₉, A₁₀, A₁₁

### Lie Type Groups ({} groups)
Projective special linear groups and other classical groups:
- PSL(2,4) ≅ A₅
- PSL(2,5) ≅ A₅
- PSL(2,7)
- PSL(2,8)
- PSL(2,9) ≅ A₆
- PSL(3,2) ≅ PSL(2,7)
- PSL(3,3)

### Sporadic Groups ({} groups)
Special finite simple groups not belonging to infinite families:
- Mathieu groups: M₁₁, M₁₂, M₂₂, M₂₃, M₂₄
- Conway groups: Co₁, Co₂, Co₃
- Fischer groups: Fi₂₂, Fi₂₃, Fi₂₄
- Monster group: Monster (largest sporadic group)
- And others...

### Twisted Lie Type Groups ({} groups)
Groups with twisted Dynkin diagrams and automorphisms.

## 🔗 Compositions

Generated direct products of groups:
{}

## 🎨 Visualization

The groups are visualized as tiles where:
- **Tile size** represents the logarithm of group order
- **Color** indicates the group family
- **Position** shows relationships and complexity
- **Connections** show group compositions

## 📖 Usage

To generate your own atlas:

```bash
# Generate complete atlas
cargo run --bin group_atlas

# Generate with filters
cargo run --bin group_atlas -- --family-filter cyclic

# Generate hierarchical view
cargo run --bin group_atlas -- --hierarchical

# Generate interactive HTML only
cargo run --bin group_atlas -- --interactive

# Generate compositions
cargo run --bin group_atlas -- --generate-compositions
```

## 🔬 Mathematical Background

Finite simple groups are the "atoms" of group theory - they cannot be decomposed into smaller normal subgroups. The classification theorem states that every finite simple group belongs to one of these categories:

1. **Cyclic groups** of prime order
2. **Alternating groups** Aₙ for n ≥ 5
3. **Groups of Lie type** (classical and exceptional)
4. **Sporadic groups** (26 exceptional cases)

This atlas provides a visual representation of these fundamental mathematical structures, showing their relationships and compositions.

## 📝 Notes

- The atlas includes a representative selection of groups for visualization
- Larger groups (like the Monster) are included for completeness
- Compositions are generated as direct products
- Interactive features allow exploration of group properties

---

*Generated automatically by cargo-vendormod group atlas generator*
"#,
        atlas.groups.len(),
        atlas.compositions.len(),
        count_families(atlas),
        get_min_order(atlas),
        get_max_order(atlas),
        count_groups_by_family(atlas, "Cyclic"),
        count_groups_by_family(atlas, "Alternating"),
        count_groups_by_family(atlas, "LieType"),
        count_groups_by_family(atlas, "Sporadic"),
        count_groups_by_family(atlas, "TwistedLieType"),
        format_compositions(atlas)
    );
    
    let readme_path = Path::new(output_dir).join("README.md");
    fs::write(&readme_path, readme_content)?;
    println!("📖 README saved to: {}", readme_path.display());
    
    Ok(())
}

/// Count number of families
fn count_families(atlas: &GroupAtlas) -> usize {
    let mut families = std::collections::HashSet::new();
    for group in &atlas.groups {
        families.insert(format!("{:?}", group.family));
    }
    families.len()
}

/// Get minimum group order
fn get_min_order(atlas: &GroupAtlas) -> String {
    atlas.groups.iter().map(|g| g.order.clone()).min().map(|o| o.to_string()).unwrap_or_default()
}

/// Get maximum group order
fn get_max_order(atlas: &GroupAtlas) -> String {
    atlas.groups.iter().map(|g| g.order.clone()).max().map(|o| o.to_string()).unwrap_or_default()
}

/// Count groups by family name
fn count_groups_by_family(atlas: &GroupAtlas, family_name: &str) -> usize {
    atlas.groups.iter()
        .filter(|g| format!("{:?}", g.family) == family_name)
        .count()
}

/// Format compositions for README
fn format_compositions(atlas: &GroupAtlas) -> String {
    if atlas.compositions.is_empty() {
        "No compositions generated".to_string()
    } else {
        atlas.compositions.iter()
            .take(5) // Show first 5 to avoid overwhelming output
            .map(|c| format!("- {}", c.name))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Print summary statistics to console
fn print_summary_stats(atlas: &GroupAtlas) {
    println!("\n📊 Atlas Summary:");
    println!("   Total Groups: {}", atlas.groups.len());
    println!("   Total Compositions: {}", atlas.compositions.len());
    println!("   Families: {}", count_families(atlas));
    println!("   Order Range: {} to {}", get_min_order(atlas), get_max_order(atlas));
    
    // Count by family
    let cyclic_count = count_groups_by_family(atlas, "Cyclic");
    let alternating_count = count_groups_by_family(atlas, "Alternating");
    let lie_count = count_groups_by_family(atlas, "LieType");
    let sporadic_count = count_groups_by_family(atlas, "Sporadic");
    let twisted_count = count_groups_by_family(atlas, "TwistedLieType");
    
    println!("   Cyclic: {}", cyclic_count);
    println!("   Alternating: {}", alternating_count);
    println!("   Lie Type: {}", lie_count);
    println!("   Sporadic: {}", sporadic_count);
    println!("   Twisted Lie: {}", twisted_count);
}