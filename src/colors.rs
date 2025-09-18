use crate::element::Element;
use sdl2::pixels::Color;

pub const BOARD_BORDER_COLOR: Color = Color::RGB(255, 255, 255);
pub const BOARD_BACKGROUND_COLOR: Color = Color::RGB(20, 0, 60);
pub const SAND_COLOR: Color = Color::RGB(200, 200, 150);
impl Element {
    pub fn color(&self) -> Color {
        match self {
            Element::None => BOARD_BACKGROUND_COLOR,
            Element::Sand => SAND_COLOR,
        }
    }
}
