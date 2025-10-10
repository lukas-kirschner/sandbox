//   sandbox - Colors
//   Copyright (C) 2025  Lukas Kirschner
//
//   This program is free software: you can redistribute it and/or modify
//   it under the terms of the GNU General Public License as published by
//   the Free Software Foundation, either version 3 of the License, or
//   (at your option) any later version.
//
//   This program is distributed in the hope that it will be useful,
//   but WITHOUT ANY WARRANTY; without even the implied warranty of
//   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//   GNU General Public License for more details.
//
//   You should have received a copy of the GNU General Public License
//   along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::element::Element;
use sdl2::pixels::Color;

pub const BOARD_BORDER_COLOR: Color = Color::RGB(255, 255, 255);
pub const BOARD_BACKGROUND_COLOR: Color = Color::RGB(20, 0, 60);
impl Element {
    pub const fn color(&self) -> Color {
        match self {
            Element::None => BOARD_BACKGROUND_COLOR,
            Element::Sand => Color::RGB(0xda, 0xca, 0xb3),
            Element::BrickWall => Color::RGB(0x8c, 0x3d, 0x20),
            Element::Water => Color::RGB(0x05, 0xaf, 0xf2),
            Element::SaltWater => Color::RGB(0x04, 0x9f, 0xc0),
            Element::Salt => Color::RGB(0xd7, 0xd7, 0xd9),
            Element::WaterSource => Color::RGB(0x9c, 0xad, 0xbc),
            Element::Steam => Color::RGB(0xee, 0xee, 0xff),
            Element::Hydrogen => Color::RGB(0x30, 0x00, 0x80),
            Element::HydrogenBurner => Color::RGB(0x25, 0x00, 0x70),
            Element::Methane => Color::RGB(0x15, 0x60, 0x00),
            Element::MethaneBurner => Color::RGB(0x10, 0x50, 0x00),
            Element::Dust => Color::RGB(0xd8, 0xe4, 0xea),
            Element::WetDust => Color::RGB(0xc8, 0xd4, 0xfa),
            Element::Flame => Color::RGB(0xf2, 0x92, 0x1d),
            Element::BurningParticle { .. } => Color::RGB(0xd9, 0x67, 0x04),
            Element::FireSource => Color::RGB(0xd6, 0x9f, 0x7e),
            Element::Gasoline => Color::RGB(0x92, 0x19, 0x09),
            Element::GasolineSource => Color::RGB(0x82, 0x15, 0x06),
        }
    }
}
