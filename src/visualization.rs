use crate::group_atlas::{FiniteSimpleGroup, GroupAtlas, GroupComposition};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use num_bigint::BigUint;

/// Tile representation for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupTile {
    pub group: FiniteSimpleGroup,
    pub position: (f64, f64),
    pub size: (u32, u32),
    pub color: String,
    pub connections: Vec<String>,
    pub level: u32,
}

/// Atlas visualization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtlasConfig {
    pub width: u32,
    pub height: u32,
    pub padding: u32,
    pub tile_spacing: u32,
    pub color_palette: HashMap<String, String>,
    pub font_size: u32,
    pub show_labels: bool,
    pub show_connections: bool,
    pub show_hierarchy: bool,
}

/// Atlas visualization renderer
#[derive(Debug, Clone)]
pub struct AtlasRenderer {
    config: AtlasConfig,
    tiles: Vec<GroupTile>,
    compositions: Vec<GroupComposition>,
}

impl AtlasRenderer {
    /// Create a new atlas renderer
    pub fn new(config: AtlasConfig) -> Self {
        Self {
            config,
            tiles: Vec::new(),
            compositions: Vec::new(),
        }
    }

    /// Generate tiles for all groups in the atlas
    pub fn generate_tiles(&mut self, atlas: &GroupAtlas) {
        self.tiles.clear();
        
        let groups_sorted = atlas.get_groups_sorted_by_order();
        let total_groups = groups_sorted.len();
        
        // Calculate grid layout
        let cols = (total_groups as f64).sqrt().ceil() as u32;
        let rows = (total_groups as f32 / cols as f32).ceil() as u32;
        
        let tile_width = (self.config.width - (cols + 1) * self.config.padding) / cols;
        let tile_height = (self.config.height - (rows + 1) * self.config.padding) / rows;
        
        for (i, group) in groups_sorted.iter().enumerate() {
            let row = i / cols as usize;
            let col = i % cols as usize;
            
            let x = self.config.padding + col as u32 * (tile_width + self.config.padding);
            let y = self.config.padding + row as u32 * (tile_height + self.config.padding);
            
            let tile = GroupTile {
                group: group.to_owned(),
                position: (x as f64, y as f64),
                size: (tile_width, tile_height),
                color: group.get_color().to_string(),
                connections: group.visualization.connections.clone(),
                level: group.get_complexity(),
            };
            
            self.tiles.push(tile);
        }
    }

    /// Generate composition tiles
    pub fn generate_composition_tiles(&mut self, atlas: &GroupAtlas) {
        self.compositions.clear();
        
        for composition in &atlas.compositions {
            let parent_tile = self.tiles.iter()
                .find(|tile| tile.group.name == composition.name)
                .cloned();
            
            if let Some(parent) = parent_tile {
                // Create connection lines between component groups and composition
                for group_name in &composition.groups {
                    let component_tile = self.tiles.iter()
                        .find(|tile| tile.group.name == *group_name);
                    
                    if let Some(component) = component_tile {
                        // Add connection visualization (just log for now)
                        println!("Connection: {} -> {}", component.group.name, parent.group.name);
                    }
                }
            }
        }
    }

    /// Add connection between tiles
    fn add_connection(&self, from: &GroupTile, to: &GroupTile) {
        // This would be implemented with actual graphics rendering
        // For now, we'll just track the connections
        println!("Connection: {} -> {}", from.group.name, to.group.name);
    }

    /// Render the atlas as SVG
    pub fn render_svg(&self) -> String {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <style>
            .tile {{ stroke: #333; stroke-width: 2; }}
            .tile-label {{ font-family: Arial, sans-serif; font-size: 12px; text-anchor: middle; }}
            .tile-order {{ font-family: Arial, sans-serif; font-size: 10px; fill: #666; text-anchor: middle; }}
            .connection {{ stroke: #666; stroke-width: 1; stroke-dasharray: 5,5; }}
            .composition {{ stroke: #ff6b6b; stroke-width: 3; }}
        </style>
    </defs>
"#,
            self.config.width, self.config.height
        ));

        // Render tiles
        for tile in &self.tiles {
            self.render_tile_svg(&mut svg, tile);
        }

        // Render connections
        if self.config.show_connections {
            self.render_connections_svg(&mut svg);
        }

        // SVG footer
        svg.push_str("</svg>");
        
        svg
    }

    /// Render a single tile as SVG
    fn render_tile_svg(&self, svg: &mut String, tile: &GroupTile) {
        let (x, y) = tile.position;
        let (width, height) = tile.size;
        
        // Tile background
        svg.push_str(&format!(
            r#"<rect x="{}" y="{}" width="{}" height="{}" 
                 fill="{}" class="tile" />
"#,
            x, y, width, height, tile.color
        ));

        // Group name
        if self.config.show_labels {
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="tile-label">{}</text>
"#,
                x + width as f64 / 2.0,
                y + 20.0,
                tile.group.name
            ));
            
            // Order
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="tile-order">|G| = {}</text>
"#,
                x + width as f64 / 2.0,
                y + 35.0,
                tile.group.order
            ));
            
            // Family
            svg.push_str(&format!(
                r#"<text x="{}" y="{}" class="tile-order">{:?}</text>
"#,
                x + width as f64 / 2.0,
                y + 50.0,
                tile.group.family
            ));
        }
    }

    /// Render connections as SVG
    fn render_connections_svg(&self, _svg: &mut String) {
        // This would implement actual connection rendering
        // For now, placeholder
    }

    /// Generate a hierarchical view
    pub fn generate_hierarchical_view(&mut self, _atlas: &GroupAtlas) -> String {
        let mut svg = String::new();
        
        // SVG header
        svg.push_str(&format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">
    <defs>
        <style>
            .level-0 {{ fill: #ff6b6b; stroke: #333; stroke-width: 2; }}
            .level-1 {{ fill: #4ecdc4; stroke: #333; stroke-width: 2; }}
            .level-2 {{ fill: #45b7d1; stroke: #333; stroke-width: 2; }}
            .level-3 {{ fill: #96ceb4; stroke: #333; stroke-width: 2; }}
            .level-4 {{ fill: #ffeaa7; stroke: #333; stroke-width: 2; }}
            .level-5 {{ fill: #fd79a8; stroke: #333; stroke-width: 2; }}
            .tile-label {{ font-family: Arial, sans-serif; font-size: 10px; text-anchor: middle; }}
            .connection {{ stroke: #666; stroke-width: 1; }}
        </style>
    </defs>
"#,
            self.config.width, self.config.height
        ));

        // Group tiles by level
        let mut levels: HashMap<u32, Vec<&GroupTile>> = HashMap::new();
        for tile in &self.tiles {
            levels.entry(tile.level).or_insert_with(Vec::new).push(tile);
        }

        // Render each level
        let level_height = self.config.height / (levels.len() as u32 + 1);
        
        for (level, tiles) in levels {
            let y = (level + 1) as u32 * level_height;
            let tile_width = (self.config.width - (tiles.len() as u32 + 1) * self.config.padding) / tiles.len() as u32;
            
            for (i, tile) in tiles.iter().enumerate() {
                let x = self.config.padding + i as u32 * (tile_width + self.config.padding);
                
                svg.push_str(&format!(
                    r#"<rect x="{}" y="{}" width="{}" height="{}" 
                         class="level-{}" />
"#,
                    x, y, tile_width, level_height - 20, level
                ));
                
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" class="tile-label">{}</text>
"#,
                    (x as f64) + (tile_width as f64 / 2.0),
                    (y as f64) + 20.0,
                    tile.group.name
                ));
                
                svg.push_str(&format!(
                    r#"<text x="{}" y="{}" class="tile-label">|G| = {}</text>
"#,
                    (x as f64) + (tile_width as f64 / 2.0),
                    (y as f64) + 35.0,
                    tile.group.order
                ));
            }
        }
        svg.push_str("</svg>");
        svg
    }

    /// Export atlas data as JSON
    pub fn export_atlas_data(&self, atlas: &GroupAtlas) -> String {
        let total_orders: BigUint = self.tiles.iter().map(|t| t.group.order.clone()).sum();
        let avg_order = &total_orders / BigUint::from(self.tiles.len());
        
        serde_json::json!({
            "config": self.config,
            "tiles": self.tiles,
            "compositions": atlas.compositions,
            "statistics": {
                "total_groups": self.tiles.len(),
                "total_compositions": atlas.compositions.len(),
                "groups_by_family": self.count_groups_by_family(atlas),
                "groups_by_level": self.count_groups_by_level(),
                "largest_group_order": self.tiles.iter().map(|t| t.group.order.clone()).max(),
                "smallest_group_order": self.tiles.iter().map(|t| t.group.order.clone()).min(),
                "average_group_order": avg_order.to_string(),
            }
        }).to_string()
    }

    /// Count groups by family
    fn count_groups_by_family(&self, _atlas: &GroupAtlas) -> HashMap<String, usize> {
        let mut counts = HashMap::new();
        for tile in &self.tiles {
            let family_name = format!("{:?}", tile.group.family);
            *counts.entry(family_name).or_insert(0) += 1;
        }
        counts
    }

    /// Count groups by level
    fn count_groups_by_level(&self) -> HashMap<u32, usize> {
        let mut counts = HashMap::new();
        for tile in &self.tiles {
            *counts.entry(tile.level).or_insert(0) += 1;
        }
        counts
    }

    /// Get interactive HTML view
    pub fn get_interactive_html(&self, atlas: &GroupAtlas) -> String {
        let svg_content = self.render_svg();
        let json_data = self.export_atlas_data(atlas);
        let group_info = self.generate_group_info_html(atlas);
        
        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Finite Simple Groups Atlas</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .container {{ display: flex; gap: 20px; }}
        .svg-container {{ flex: 1; border: 1px solid #ccc; padding: 10px; }}
        .info-panel {{ flex: 1; max-height: 800px; overflow-y: auto; }}
        .tile {{ cursor: pointer; transition: all 0.3s; }}
        .tile:hover {{ opacity: 0.8; }}
        .group-info {{ margin-bottom: 20px; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .stats {{ background: #f5f5f5; padding: 15px; border-radius: 5px; }}
    </style>
</head>
<body>
    <h1>Finite Simple Groups Atlas</h1>
    
    <div class="container">
        <div class="svg-container">
            <h2>Group Tiles</h2>
            {}
        </div>
        
        <div class="info-panel">
            <h2>Atlas Statistics</h2>
            <div class="stats">
                <pre>{}</pre>
            </div>
            
            <h2>Group Details</h2>
            {}
        </div>
    </div>
    
    <script>
        // Interactive functionality would go here
        console.log('Atlas data loaded:', JSON.parse('{}'));
    </script>
</body>
</html>"#,
            svg_content, json_data, group_info, json_data
        )
    }

    /// Generate HTML for group information
    fn generate_group_info_html(&self, _atlas: &GroupAtlas) -> String {
        let mut html = String::new();
        
        for tile in &self.tiles {
            html.push_str(&format!(
                r#"<div class="group-info">
                    <h3>{}</h3>
                    <p><strong>Order:</strong> {}</p>
                    <p><strong>Family:</strong> {:?}</p>
                    <p><strong>Description:</strong> {}</p>
                    <p><strong>Complexity:</strong> {}/5</p>
                    <p><strong>Simple:</strong> {}</p>
                    <p><strong>Perfect:</strong> {}</p>
                    <p><strong>Centerless:</strong> {}</p>
                </div>"#,
                tile.group.name,
                tile.group.order,
                tile.group.family,
                tile.group.description,
                tile.level,
                tile.group.properties.simple,
                tile.group.properties.perfect,
                tile.group.properties.centerless
            ));
        }
        
        html
    }
}

/// Default atlas configuration
impl Default for AtlasConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            padding: 20,
            tile_spacing: 10,
            color_palette: HashMap::new(),
            font_size: 12,
            show_labels: true,
            show_connections: true,
            show_hierarchy: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::group_atlas::GroupAtlas;

    #[test]
    fn test_atlas_renderer_creation() {
        let config = AtlasConfig::default();
        let renderer = AtlasRenderer::new(config);
        assert_eq!(renderer.tiles.len(), 0);
        assert_eq!(renderer.compositions.len(), 0);
    }

    #[test]
    fn test_tile_generation() {
        let config = AtlasConfig::default();
        let mut renderer = AtlasRenderer::new(config);
        let atlas = GroupAtlas::new();
        
        renderer.generate_tiles(&atlas);
        assert!(!renderer.tiles.is_empty());
        
        // Check that each tile has proper properties
        for tile in &renderer.tiles {
            assert!(!tile.group.name.is_empty());
            assert!(tile.group.order > BigUint::from(0u32));
            assert!(!tile.color.is_empty());
        }
    }

    #[test]
    fn test_svg_rendering() {
        let config = AtlasConfig::default();
        let mut renderer = AtlasRenderer::new(config);
        let atlas = GroupAtlas::new();
        
        renderer.generate_tiles(&atlas);
        let svg = renderer.render_svg();
        
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
        assert!(svg.contains("rect"));
        assert!(svg.contains("text"));
    }

    #[test]
    fn test_hierarchical_view() {
        let config = AtlasConfig::default();
        let mut renderer = AtlasRenderer::new(config);
        let atlas = GroupAtlas::new();
        
        let hierarchical_svg = renderer.generate_hierarchical_view(&atlas);
        
        assert!(hierarchical_svg.contains("<svg"));
        assert!(hierarchical_svg.contains("</svg>"));
        assert!(hierarchical_svg.contains("level-"));
    }

    #[test]
    fn test_json_export() {
        let config = AtlasConfig::default();
        let mut renderer = AtlasRenderer::new(config);
        let atlas = GroupAtlas::new();
        
        renderer.generate_tiles(&atlas);
        let json_data = renderer.export_atlas_data(&atlas);
        
        assert!(!json_data.is_empty());
        assert!(json_data.contains("total_groups"));
        assert!(json_data.contains("total_compositions"));
    }
}