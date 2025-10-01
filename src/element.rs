use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;

pub const AIR_DENSITY: f32 = 1.2754;
#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter)]
pub enum Element {
    None,
    BrickWall,
    Sand,
    Salt,
    Water,
    SaltWater,
    WaterSource,
    Steam,
    Hydrogen,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ElementKind {
    None,
    Solid,
    Powder {
        /// The density in kg/m³.
        /// Controls the behavior of powders in liquids (i.e., powders more dense than liquids sink down while less dense powders do not sink down).
        /// While falling down, less dense powders get a higher probability of spreading to the side while falling (i.e., skipping a down-fall or down-side-fall).
        density: f32,
    },
    Liquid {
        /// The density in kg/m³.
        /// Controls the displacement of other liquids and the spread of liquids while free-falling down.
        density: f32,
    },
    Gas {
        /// The density in kg/m³.
        /// Controls the displacement of other gases and liquids while rising up.
        /// Lighter gases have less probability of spreading while rising up.
        density: f32,
    },
}

impl Element {
    /// The element kind and associated properties (density, ...)
    pub const fn kind(&self) -> ElementKind {
        match self {
            Element::None => ElementKind::None,
            Element::Sand => ElementKind::Powder { density: 1700.0 },
            Element::BrickWall => ElementKind::Solid,
            Element::Water => ElementKind::Liquid { density: 997.0 },
            Element::SaltWater => ElementKind::Liquid { density: 1027.0 },
            Element::Salt => ElementKind::Powder { density: 2170.0 },
            Element::WaterSource => ElementKind::Solid,
            Element::Steam => ElementKind::Gas { density: 0.6 },
            Element::Hydrogen => ElementKind::Gas { density: 0.08988 },
        }
    }
    pub const fn density(&self) -> Option<f32> {
        match self.kind() {
            ElementKind::None => None,
            ElementKind::Solid => None,
            ElementKind::Powder { density } => Some(density),
            ElementKind::Liquid { density } => Some(density),
            ElementKind::Gas { density } => Some(density),
        }
    }
    /// Whether the given element is a liquid or a gas, i.e., whether the element can swap its position with other elements
    pub const fn is_liquid_or_gas(&self) -> bool {
        match self.kind() {
            ElementKind::None => false,
            ElementKind::Solid => false,
            ElementKind::Powder { .. } => false,
            ElementKind::Liquid { .. } => true,
            ElementKind::Gas { .. } => true,
        }
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Element::None => "Air",
                Element::Sand => "Sand",
                Element::BrickWall => "Wall",
                Element::Water => "Water",
                Element::SaltWater => "Saltwater",
                Element::Salt => "Salt",
                Element::WaterSource => "Water Source",
                Element::Steam => "Steam",
                Element::Hydrogen => "Hydrogen",
            }
        )
    }
}
