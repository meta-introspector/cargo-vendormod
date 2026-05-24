use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use std::path::PathBuf;
use num_bigint::BigUint;
use num_traits::FromPrimitive;

/// Represents a finite simple group with its mathematical properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FiniteSimpleGroup {
    pub name: String,
    pub order: BigUint,
    pub family: GroupFamily,
    pub description: String,
    pub properties: GroupProperties,
    pub visualization: GroupVisualization,
}

/// Family of the finite simple group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GroupFamily {
    Cyclic,
    Alternating,
    LieType(ClassicalLieType),
    Sporadic(SporadicGroup),
    TwistedLieType(TwistedLieType),
}

/// Classical Lie type groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ClassicalLieType {
    An(n), // PSL(n+1, q)
    Bn(n), // PΩ(2n+1, q)
    Cn(n), // PSp(2n, q)
    Dn(n), // PΩ(2n, q)
    Exceptional(ExceptionalType),
}

/// Exceptional Lie types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExceptionalType {
    G2,
    F4,
    E6,
    E7,
    E8,
    SuzukiRee(SuzukiReeType),
}

/// Suzuki-Ree types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SuzukiReeType {
    Suzuki,
    Ree,
}

/// Twisted Lie type groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TwistedLieType {
    TwoAn(n), // PSU(n+1, q)
    TwoBn(n), // Ω⁻(2n+1, q)
    TwoDn(n), // Ω⁺(2n, q)
    TwoG2,    // Suz(q)
    TwoF4,    // 2F4(q)
    TwoE6,    // 2E6(q)
    ThreeD4,  // 3D4(q)
}

/// Sporadic groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SporadicGroup {
    Mathieu11,
    Mathieu12,
    Mathieu22,
    Mathieu23,
    Mathieu24,
    ConwayCo1,
    ConwayCo2,
    ConwayCo3,
    Fischer22,
    Fischer23,
    Fischer24,
    HigmanSims,
    McLaughlin,
    Held,
    Rudvalis,
    Suzuki,
    ONan,
    HaradaNorton,
    Lyons,
    Thompson,
    BabyMonster,
    Monster,
    Janko1,
    Janko2,
    Janko3,
    Janko4,
}

/// Group properties for mathematical characterization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupProperties {
    pub simple: bool,
    pub perfect: bool,
    pub centerless: bool,
    pub schur_multiplier: Vec<u64>,
    pub outer_automorphism_group: String,
    pub conjugacy_classes: u32,
    pub maximal_subgroups: Vec<String>,
}

/// Visualization properties for tile representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupVisualization {
    pub tile_size: (u32, u32),
    pub color_scheme: String,
    pub pattern: String,
    pub complexity: u32,
    pub connections: Vec<String>,
}

/// Parameter for Lie type groups
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct n {
    pub value: u32,
    pub field_size: u32,
}

impl fmt::Display for n {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}({})", self.value, self.field_size)
    }
}

impl FiniteSimpleGroup {
    /// Create a new finite simple group
    pub fn new(
        name: String,
        order: BigUint,
        family: GroupFamily,
        description: String,
        properties: GroupProperties,
        visualization: GroupVisualization,
    ) -> Self {
        Self {
            name,
            order,
            family,
            description,
            properties,
            visualization,
        }
    }

    /// Get the tile size based on group order
    pub fn get_tile_size(&self) -> (u32, u32) {
        // Use string length as approximation for log10
        let digits = self.order.to_string().len() as u32;
        let size_factor = digits.max(1);
        let base_size = 64;
        (base_size + size_factor * 16, base_size + size_factor * 16)
    }

    /// Get color based on family
    pub fn get_color(&self) -> &str {
        match &self.family {
            GroupFamily::Cyclic => "#FF6B6B",
            GroupFamily::Alternating => "#4ECDC4",
            GroupFamily::LieType(_) => "#45B7D1",
            GroupFamily::Sporadic(_) => "#96CEB4",
            GroupFamily::TwistedLieType(_) => "#FFEAA7",
        }
    }

    /// Get complexity score
    pub fn get_complexity(&self) -> u32 {
        match &self.family {
            GroupFamily::Cyclic => 1,
            GroupFamily::Alternating => 2,
            GroupFamily::LieType(_) => 3,
            GroupFamily::Sporadic(_) => 4,
            GroupFamily::TwistedLieType(_) => 5,
        }
    }

    /// Check if this group can compose with another
    pub fn can_compose_with(&self, other: &FiniteSimpleGroup) -> bool {
        // Groups can compose if they have different families or are from same family but different orders
        self.family != other.family || self.order != other.order
    }

    /// Get the order of the direct product
    pub fn direct_product_order(&self, other: &FiniteSimpleGroup) -> BigUint {
        &self.order * &other.order
    }
}

/// Atlas of finite simple groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupAtlas {
    pub groups: Vec<FiniteSimpleGroup>,
    pub compositions: Vec<GroupComposition>,
}

/// Composition of groups
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupComposition {
    pub name: String,
    pub groups: Vec<String>,
    pub composition_type: CompositionType,
    pub order: BigUint,
    pub visualization: GroupVisualization,
}

/// Type of group composition
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CompositionType {
    DirectProduct,
    SemidirectProduct,
    WreathProduct,
    CentralProduct,
}

impl GroupAtlas {
    /// Create a new atlas with all finite simple groups
    pub fn new() -> Self {
        let groups = Self::create_all_finite_simple_groups();
        let compositions = Vec::new();
        
        Self {
            groups,
            compositions,
        }
    }

    /// Create all finite simple groups
    fn create_all_finite_simple_groups() -> Vec<FiniteSimpleGroup> {
        let mut groups = Vec::new();

        // Cyclic groups of prime order (only simple ones)
        for p in [2u128, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31].iter() {
            groups.push(Self::create_cyclic_group(*p));
        }

        // Alternating groups A_n for n >= 5
        for n in [5, 6, 7, 8, 9, 10, 11].iter() {
            groups.push(Self::create_alternating_group(*n));
        }

        // Lie type groups (simplified representation)
        groups.push(Self::create_lie_type_group(
            "PSL(2,4)".to_string(),
            60,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 1, field_size: 4 })),
            "Projective special linear group PSL(2,4) ≅ A₅".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(2,5)".to_string(),
            60,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 1, field_size: 5 })),
            "Projective special linear group PSL(2,5) ≅ A₅".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(2,7)".to_string(),
            168,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 1, field_size: 7 })),
            "Projective special linear group PSL(2,7)".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(2,8)".to_string(),
            504,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 1, field_size: 8 })),
            "Projective special linear group PSL(2,8)".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(2,9)".to_string(),
            360,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 1, field_size: 9 })),
            "Projective special linear group PSL(2,9) ≅ A₆".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(3,2)".to_string(),
            168,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 2, field_size: 2 })),
            "Projective special linear group PSL(3,2) ≅ PSL(2,7)".to_string(),
        ));

        groups.push(Self::create_lie_type_group(
            "PSL(3,3)".to_string(),
            5616,
            GroupFamily::LieType(ClassicalLieType::An(n { value: 2, field_size: 3 })),
            "Projective special linear group PSL(3,3)".to_string(),
        ));

        // Sporadic groups (simplified representation)
        groups.push(Self::create_sporadic_group(
            "Mathieu11".to_string(),
            BigUint::from(7920u32),
            GroupFamily::Sporadic(SporadicGroup::Mathieu11),
            "Mathieu group M₁₁".to_string(),
        ));

        groups.push(Self::create_sporadic_group(
            "Mathieu12".to_string(),
            BigUint::from(95040u32),
            GroupFamily::Sporadic(SporadicGroup::Mathieu12),
            "Mathieu group M₁₂".to_string(),
        ));

        groups.push(Self::create_sporadic_group(
            "Mathieu22".to_string(),
            BigUint::from(443520u32),
            GroupFamily::Sporadic(SporadicGroup::Mathieu22),
            "Mathieu group M₂₂".to_string(),
        ));

        groups.push(Self::create_sporadic_group(
            "Mathieu23".to_string(),
            BigUint::from(10200960u32),
            GroupFamily::Sporadic(SporadicGroup::Mathieu23),
            "Mathieu group M₂₃".to_string(),
        ));

        groups.push(Self::create_sporadic_group(
            "Mathieu24".to_string(),
            BigUint::from(244823040u32),
            GroupFamily::Sporadic(SporadicGroup::Mathieu24),
            "Mathieu group M₂₄".to_string(),
        ));

        groups.push(Self::create_sporadic_group(
            "Monster".to_string(),
            "808017424794512875886459904961710757005754368000000000".parse::<BigUint>().unwrap(),
            GroupFamily::Sporadic(SporadicGroup::Monster),
            "Monster group (largest sporadic group)".to_string(),
        ));

        groups
    }

    fn create_cyclic_group(prime: u128) -> FiniteSimpleGroup {
        FiniteSimpleGroup {
            name: format!("C_{{{}}}", prime),
            order: BigUint::from_u128(prime).unwrap(),
            family: GroupFamily::Cyclic,
            description: format!("Cyclic group of prime order {}", prime),
            properties: GroupProperties {
                simple: true,
                perfect: prime == 2 || prime == 3,
                centerless: true,
                schur_multiplier: vec![],
                outer_automorphism_group: format!("C_{{{}}}", prime - 1),
                conjugacy_classes: prime as u32,
                maximal_subgroups: vec![],
            },
            visualization: GroupVisualization {
                tile_size: (64, 64),
                color_scheme: "monochromatic".to_string(),
                pattern: "circular".to_string(),
                complexity: 1,
                connections: vec![],
            },
        }
    }

    fn create_alternating_group(n: u32) -> FiniteSimpleGroup {
        // Calculate n! / 2 for alternating group order
        let mut order: u128 = 1;
        for i in 1..=n as u128 {
            order *= i;
        }
        order /= 2;
        
        FiniteSimpleGroup {
            name: format!("A_{{{}}}", n),
            order: BigUint::from_u128(order).unwrap(),
            family: GroupFamily::Alternating,
            description: format!("Alternating group A_{{{}}}", n),
            properties: GroupProperties {
                simple: n >= 5,
                perfect: n >= 5,
                centerless: n >= 4,
                schur_multiplier: if n == 6 { vec![2] } else if n == 7 { vec![6] } else { vec![] },
                outer_automorphism_group: if n == 6 { "C_2" } else { "C_2" }.to_string(),
                conjugacy_classes: n as u32 * (n - 1) / 2,
                maximal_subgroups: vec![format!("A_{{{}}}", n - 1)],
            },
            visualization: GroupVisualization {
                tile_size: (80 + n * 4, 80 + n * 4),
                color_scheme: "gradient".to_string(),
                pattern: "triangular".to_string(),
                complexity: 2,
                connections: vec![],
            },
        }
    }

    fn create_lie_type_group(name: String, order: u128, family: GroupFamily, description: String) -> FiniteSimpleGroup {
        FiniteSimpleGroup {
            name,
            order: BigUint::from_u128(order).unwrap(),
            family,
            description,
            properties: GroupProperties {
                simple: true,
                perfect: true,
                centerless: true,
                schur_multiplier: vec![],
                outer_automorphism_group: "C_2".to_string(),
                conjugacy_classes: 0, // Would need specific calculation
                maximal_subgroups: vec![],
            },
            visualization: GroupVisualization {
                tile_size: (96, 96),
                color_scheme: "spectral".to_string(),
                pattern: "geometric".to_string(),
                complexity: 3,
                connections: vec![],
            },
        }
    }

    fn create_sporadic_group(name: String, order: BigUint, family: GroupFamily, description: String) -> FiniteSimpleGroup {
        FiniteSimpleGroup {
            name,
            order,
            family,
            description,
            properties: GroupProperties {
                simple: true,
                perfect: true,
                centerless: true,
                schur_multiplier: vec![],
                outer_automorphism_group: "trivial".to_string(),
                conjugacy_classes: 0, // Would need specific calculation
                maximal_subgroups: vec![],
            },
            visualization: GroupVisualization {
                tile_size: (128, 128),
                color_scheme: "rainbow".to_string(),
                pattern: "complex".to_string(),
                complexity: 4,
                connections: vec![],
            },
        }
    }

    /// Get group by name
    pub fn get_group(&self, name: &str) -> Option<&FiniteSimpleGroup> {
        self.groups.iter().find(|g| g.name == name)
    }

    /// Get all groups sorted by order
    pub fn get_groups_sorted_by_order(&self) -> Vec<FiniteSimpleGroup> {
        let mut groups = self.groups.clone();
        groups.sort_by(|a, b| a.order.cmp(&b.order));
        groups
    }

    /// Get groups by family
    pub fn get_groups_by_family(&self, family: GroupFamily) -> Vec<&FiniteSimpleGroup> {
        self.groups.iter().filter(|g| g.family == family).collect()
    }

    /// Create a composition of groups
    pub fn create_composition(&mut self, name: String, group_names: Vec<String>, composition_type: CompositionType) {
        let groups: Vec<&FiniteSimpleGroup> = group_names.iter()
            .filter_map(|name| self.get_group(name))
            .collect();
        
        if groups.len() != group_names.len() {
            eprintln!("Warning: Some groups not found for composition: {}", name);
            return;
        }

        let order = groups.iter().map(|g| g.order.clone()).product();
        let max_complexity = groups.iter().map(|g| g.get_complexity()).max().unwrap_or(0);
        
        let visualization = GroupVisualization {
            tile_size: (64 * groups.len() as u32, 64 * groups.len() as u32),
            color_scheme: "composite".to_string(),
            pattern: "mosaic".to_string(),
            complexity: max_complexity + 1,
            connections: group_names.clone(),
        };

        let composition = GroupComposition {
            name,
            groups: group_names,
            composition_type,
            order,
            visualization,
        };

        self.compositions.push(composition);
    }

    /// Generate all possible direct products
    pub fn generate_all_direct_products(&mut self) {
        for i in 0..self.groups.len() {
            for j in i + 1..self.groups.len() {
                let g1 = &self.groups[i];
                let g2 = &self.groups[j];
                
                if g1.can_compose_with(g2) {
                    let name = format!("{} × {}", g1.name, g2.name);
                    let group_names = vec![g1.name.clone(), g2.name.clone()];
                    
                    self.create_composition(
                        name,
                        group_names,
                        CompositionType::DirectProduct,
                    );
                }
            }
        }
    }
}

impl Default for GroupAtlas {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cyclic_group_creation() {
        let atlas = GroupAtlas::new();
        // Find a cyclic group (the atlas has groups named C_2, C_3, etc.)
        let group = atlas.groups.iter().find(|g| matches!(g.family, GroupFamily::Cyclic));
        assert!(group.is_some());
        let group = group.unwrap();
        assert!(group.properties.simple);
        assert_eq!(group.get_tile_size(), (64, 64));
    }

    #[test]
    fn test_alternating_group_creation() {
        let atlas = GroupAtlas::new();
        // Find an alternating group (the atlas has groups named A_5, A_6, etc.)
        let group = atlas.groups.iter().find(|g| matches!(g.family, GroupFamily::Alternating));
        assert!(group.is_some());
        let group = group.unwrap();
        assert!(group.properties.simple);
        assert!(group.properties.perfect);
    }

    #[test]
    fn test_group_composition() {
        let mut atlas = GroupAtlas::new();
        atlas.create_composition(
            "C_2 × C_3".to_string(),
            vec!["C_2".to_string(), "C_3".to_string()],
            CompositionType::DirectProduct,
        );
        
        assert_eq!(atlas.compositions.len(), 1);
        assert_eq!(atlas.compositions[0].order, BigUint::from(6u32));
    }

    #[test]
    fn test_group_sorting() {
        let atlas = GroupAtlas::new();
        let sorted = atlas.get_groups_sorted_by_order();
        
        for i in 1..sorted.len() {
            assert!(sorted[i-1].order <= sorted[i].order);
        }
    }

    #[test]
    fn test_family_filtering() {
        let atlas = GroupAtlas::new();
        let cyclic_groups = atlas.get_groups_by_family(GroupFamily::Cyclic);
        
        assert!(!cyclic_groups.is_empty());
        for group in cyclic_groups {
            matches!(group.family, GroupFamily::Cyclic);
        }
    }
}