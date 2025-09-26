use std::fmt::{Display, Formatter};
use strum_macros::EnumIter;
#[derive(Copy, Clone, Eq, PartialEq, Debug, EnumIter)]
pub enum Element {
    None,
    Sand,
    BrickWall,
}

pub enum ElementKind {
    None,
    Solid,
    Powder { density: f32 },
    Liquid { density: f32 },
    Gas { density: f32 },
}

impl Element {
    pub fn kind(&self) -> ElementKind {
        match self {
            Element::None => ElementKind::None,
            Element::Sand => ElementKind::Powder { density: 1.0 },
            Element::BrickWall => ElementKind::Solid,
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
            }
        )
    }
}
