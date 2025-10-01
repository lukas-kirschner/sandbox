use crate::element::Element;
use sdl2::pixels::Color;

pub const BOARD_BORDER_COLOR: Color = Color::RGB(255, 255, 255);
pub const BOARD_BACKGROUND_COLOR: Color = Color::RGB(20, 0, 60);
pub const SAND_COLOR: Color = Color::RGB(0xda, 0xca, 0xb3);
pub const DUST_COLOR: Color = Color::RGB(0xd8, 0xe4, 0xea);
pub const WET_DUST_COLOR: Color = Color::RGB(0xc8, 0xd4, 0xfa);
pub const BRICK_WALL_COLOR: Color = Color::RGB(0x8c, 0x3d, 0x20);
pub const WATER_COLOR: Color = Color::RGB(0x05, 0xaf, 0xf2);
pub const SALT_WATER_COLOR: Color = Color::RGB(0x04, 0x9f, 0xc0);
pub const SALT_COLOR: Color = Color::RGB(0xd7, 0xd7, 0xd9);
pub const WATER_SOURCE_COLOR: Color = Color::RGB(0x9c, 0xad, 0xbc);
pub const STEAM_COLOR: Color = Color::RGB(0xee, 0xee, 0xff);
pub const HYDROGEN_COLOR: Color = Color::RGB(0x30, 0x00, 0x80);
pub const FLAME_COLOR: Color = Color::RGB(0xf2, 0x92, 0x1d);
pub const BURNING_COLOR: Color = Color::RGB(0xd9, 0x67, 0x04);
impl Element {
    pub fn color(&self) -> Color {
        match self {
            Element::None => BOARD_BACKGROUND_COLOR,
            Element::Sand => SAND_COLOR,
            Element::BrickWall => BRICK_WALL_COLOR,
            Element::Water => WATER_COLOR,
            Element::SaltWater => SALT_WATER_COLOR,
            Element::Salt => SALT_COLOR,
            Element::WaterSource => WATER_SOURCE_COLOR,
            Element::Steam => STEAM_COLOR,
            Element::Hydrogen => HYDROGEN_COLOR,
            Element::Dust => DUST_COLOR,
            Element::WetDust => WET_DUST_COLOR,
            Element::Flame => FLAME_COLOR,
            Element::BurningParticle { .. } => BURNING_COLOR,
        }
    }
}
