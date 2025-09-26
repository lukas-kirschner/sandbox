use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;
#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter)]
pub enum Element {
    None,
    BrickWall,
    Sand,
    Water,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ElementKind {
    None,
    Solid,
    Powder {
        /// The density in kg/m³
        density: f32,
    },
    Liquid {
        // The density in kg/m³
        density: f32,
    },
    Gas {
        density: f32,
    },
}

impl Element {
    pub fn kind(&self) -> ElementKind {
        match self {
            Element::None => ElementKind::None,
            Element::Sand => ElementKind::Powder { density: 1700.0 },
            Element::BrickWall => ElementKind::Solid,
            Element::Water => ElementKind::Liquid { density: 1000.0 },
        }
    }
    pub fn density(&self) -> Option<f32> {
        match self.kind() {
            ElementKind::None => None,
            ElementKind::Solid => None,
            ElementKind::Powder { density } => Some(density),
            ElementKind::Liquid { density } => Some(density),
            ElementKind::Gas { density } => Some(density),
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
            }
        )
    }
}
