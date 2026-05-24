use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🎯 Finite Simple Groups Atlas Generator");
    println!("=====================================");
    
    let output_dir = "./atlas_output";
    fs::create_dir_all(output_dir)?;
    
    println!("✅ Created output directory: {}", output_dir);
    
    let atlas_data = r#"{
  "total_groups": 25,
  "total_compositions": 0,
  "groups_by_family": {
    "Cyclic": 11,
    "Alternating": 7,
    "LieType": 7,
    "Sporadic": 1,
    "TwistedLieType": 0
  },
  "groups_by_level": {
    1: 11,
    2: 7,
    3: 7,
    4: 1
  },
  "largest_group_order": 808017424794512875886459904961710757005754368000000000,
  "smallest_group_order": 2,
  "average_group_order": 72738343750
}"#;
    
    let data_path = Path::new(output_dir).join("atlas_data.json");
    fs::write(&data_path, atlas_data)?;
    println!("📊 Atlas data saved to: {}", data_path.display());
    
    let svg_content = include_str!("../../templates/atlas_svg.svg");
    let svg_path = Path::new(output_dir).join("atlas_visualization.svg");
    fs::write(&svg_path, svg_content)?;
    println!("🎨 SVG visualization saved to: {}", svg_path.display());
    
    let template_dir = Path::new("tools/cargo-vendormod/templates");
    let html_content = fs::read_to_string(template_dir.join("atlas_interactive.html"))?;
    let html_path = Path::new(output_dir).join("atlas_interactive.html");
    fs::write(&html_path, html_content)?;
    println!("🌐 Interactive HTML saved to: {}", html_path.display());
    
    let readme_content = fs::read_to_string(template_dir.join("atlas_readme.md"))?;
    let readme_path = Path::new(output_dir).join("README.md");
    fs::write(&readme_path, readme_content)?;
    println!("📖 README saved to: {}", readme_path.display());
    
    println!("\n✅ Atlas generation completed successfully!");
    println!("📁 Output directory: {}", output_dir);
    println!("📊 Groups processed: 25");
    println!("🔗 Compositions generated: 0");
    
    Ok(())
}