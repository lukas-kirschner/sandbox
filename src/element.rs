use std::cmp::max;
use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;

pub const AIR_DENSITY: f32 = 1.2754;
#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter)]
pub enum Element {
    None,
    BrickWall,
    Sand,
    Salt,
    Dust,
    WetDust,
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
    /// The probability of an element to spread to the side (down-side/up-side for gases) instead of falling down.
    /// The probability of spreading is calculated as follows:
    ///     p_spreading = max(0, (1 - |density1 - density2| / 3000) * 0.3 )
    /// -> i.e., concrete powder (density > 3000) will never spread
    pub const fn spread_prob(&self, environment: &Element) -> f64{
        let displaced_density = match environment.kind(){
            ElementKind::None => AIR_DENSITY,
            ElementKind::Solid => return 0.0, // Cannot replace solid!
            ElementKind::Powder { density } => density,
            ElementKind::Liquid { density } => density,
            ElementKind::Gas { density } => density,
        };
        let density_diff = (displaced_density - match self.kind(){
            ElementKind::None => AIR_DENSITY,
            ElementKind::Solid => return 0.0,
            ElementKind::Powder { density } => density,
            ElementKind::Liquid { density } => density,
            ElementKind::Gas { density } => density,
        }).abs() as f64;
        (1.0 - density_diff / 3000.0).max(0.0) * 0.3
    }
    /// The probability of an element spreading to the side instead of rising up or falling down.
    /// The probability of a side spread is calculated as follows:
    ///     p_side = p_spreading * 0.5
    pub const fn spread_side_prob(&self, environment: &Element)->f64{
        self.spread_prob(environment) * 0.5
    }
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
            Element::Dust => ElementKind::Powder {density: 3.0},
            Element::WetDust => ElementKind::Powder {density: 1000.0}
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
                Element::Dust => "Dust",
                Element::WetDust => "Wet Dust",
            }
        )
    }
}
