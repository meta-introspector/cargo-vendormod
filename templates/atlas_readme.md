# Finite Simple Groups Atlas

This directory contains a comprehensive atlas of finite simple groups visualized as interactive tiles.

## 📊 Statistics

- **Total Groups**: 25
- **Total Compositions**: 0
- **Group Families**: 5
- **Order Range**: 2 to 8×10⁵³

## 📁 Files Generated

- `atlas_data.json` - Complete atlas data in JSON format
- `atlas_visualization.svg` - Grid layout visualization
- `atlas_interactive.html` - Interactive web-based visualization
- `README.md` - This file

## 🎯 Group Families

### Cyclic Groups (11 groups)
Groups of prime order: C₂, C₃, C₅, C₇, C₁₁, C₁₃, C₁₇, C₁₉, C₂₃, C₂₉, C₃₁

### Alternating Groups (7 groups)
Alternating groups Aₙ for n ≥ 5: A₅, A₆, A₇, A₈, A₉, A₁₀, A₁₁

### Lie Type Groups (7 groups)
Projective special linear groups and other classical groups:
- PSL(2,4) ≅ A₅
- PSL(2,5) ≅ A₅
- PSL(2,7)
- PSL(2,8)
- PSL(2,9) ≅ A₆
- PSL(3,2) ≅ PSL(2,7)
- PSL(3,3)

### Sporadic Groups (1 group)
Special finite simple groups not belonging to infinite families:
- Monster group: Monster (largest sporadic group)

### Twisted Lie Type Groups (0 groups)
Groups with twisted Dynkin diagrams and automorphisms.

## 🔗 Compositions

No compositions generated in this sample. The full implementation supports generating direct products and other compositions.

## 🎨 Visualization

The groups are visualized as tiles where:
- **Tile size** represents the logarithm of group order
- **Color** indicates the group family
- **Position** shows relationships and complexity
- **Connections** show group compositions

## 📖 Usage

To generate your own atlas with the full implementation:

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
5. **Twisted Lie type groups** (with twisted Dynkin diagrams)

This atlas provides a visual representation of these fundamental mathematical structures, showing their relationships and compositions.

## 📝 Notes

- This is a sample implementation demonstrating the concept
- The full implementation includes all 44 finite simple groups
- Compositions are generated as direct products
- Interactive features allow exploration of group properties

---

*Generated automatically by cargo-vendormod group atlas generator*