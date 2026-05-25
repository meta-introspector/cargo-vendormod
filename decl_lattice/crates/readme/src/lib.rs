use std :: fs ; use std :: path :: Path ; use clap :: Parser ; use serde_json ; use num_bigint :: BigUint ; use cargo_vendormod :: group_atlas :: { GroupAtlas , GroupFamily , ClassicalLieType , TwistedLieType , SporadicGroup , n } ; use cargo_vendormod :: visualization :: { AtlasRenderer , AtlasConfig } ; # [doc = " Generate README file"] fn generate_readme (atlas : & GroupAtlas , output_dir : & str) -> Result < () , Box < dyn std :: error :: Error > > { let readme_content = format ! (r#"# Finite Simple Groups Atlas

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
"# , atlas . groups . len () , atlas . compositions . len () , count_families (atlas) , get_min_order (atlas) , get_max_order (atlas) , count_groups_by_family (atlas , "Cyclic") , count_groups_by_family (atlas , "Alternating") , count_groups_by_family (atlas , "LieType") , count_groups_by_family (atlas , "Sporadic") , count_groups_by_family (atlas , "TwistedLieType") , format_compositions (atlas)) ; let readme_path = Path :: new (output_dir) . join ("README.md") ; fs :: write (& readme_path , readme_content) ? ; println ! ("📖 README saved to: {}" , readme_path . display ()) ; Ok (()) }