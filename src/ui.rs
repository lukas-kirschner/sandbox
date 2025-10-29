//   sandbox - UI drawing code
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

use crate::canvas_display::CanvasDisplay;
use crate::colors::BOARD_BORDER_COLOR;
use crate::world::GameWorld;
use embedded_graphics::Drawable;
use embedded_graphics::geometry::Size;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::primitives::{PrimitiveStyle, Rectangle};
use sdl2::pixels::Color;
use sdl2::rect::{Point, Rect};
use sdl2::render::{Canvas, RenderTarget, Texture, WindowCanvas};

pub const CURSOR_PREVIEW_COLOR: Color = Color::RGBA(0xff, 0xff, 0xff, 0x30);
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum CursorKind {
    Square { size: u32 },
}
pub struct Ui {
    pub win_width: usize,
    pub win_height: usize,
    /// Board width in pixels
    pub board_width: usize,
    /// Board height in pixels
    pub board_height: usize,
    cursor: CursorKind,
    pub(crate) scaling_factor: usize,
}

impl Ui {
    pub const fn top_buttonbar_height(&self) -> f32 {
        ((self.win_height - self.board_height) / 2 - 1) as f32
    }
    pub const fn right_buttonbar_width(&self) -> f32 {
        ((self.win_width - self.board_width) / 2 - 2) as f32
    }
    pub const fn left_buttonbar_width(&self) -> f32 {
        ((self.win_width - self.board_width) / 2 - 2) as f32
    }
    pub(crate) fn draw_mouse_preview_at<T: RenderTarget>(
        &self,
        canvas: &mut Canvas<T>,
        x: i32,
        y: i32,
    ) -> Result<(), String> {
        if let Some((x, y)) = self.window_to_board_coordinate(x, y) {
            match self.cursor {
                CursorKind::Square { size } => {
                    Rectangle::with_center(
                        embedded_graphics::prelude::Point::new(
                            x * self.scaling_factor as i32 + self.left_padding() + 1,
                            y * self.scaling_factor as i32 + self.top_padding() + 1,
                        ),
                        Size::new(
                            size * self.scaling_factor as u32,
                            size * self.scaling_factor as u32,
                        ),
                    )
                    .into_styled(PrimitiveStyle::with_fill(CURSOR_PREVIEW_COLOR.into()))
                    .draw(&mut CanvasDisplay {
                        canvas,
                        width: self.win_width,
                        height: self.win_height,
                        left_padding: self.left_padding(),
                        right_padding: self.win_width as i32 - self.right_padding(),
                        top_padding: self.top_padding(),
                        bottom_padding: self.win_height as i32 - self.bottom_padding(),
                    })?;
                },
            }
        }
        Ok(())
    }
}

impl Ui {
    pub(crate) fn set_cursor(&mut self, cursor: CursorKind) {
        self.cursor = cursor
    }
}

impl Ui {
    pub fn cursor(&self) -> &CursorKind {
        &self.cursor
    }
    pub(crate) fn window_to_board_coordinate(
        &self,
        window_x: i32,
        window_y: i32,
    ) -> Option<(i32, i32)> {
        let ret = (
            (window_x - (self.win_width - self.board_width) as i32 / 2)
                / self.scaling_factor as i32,
            (window_y - (self.win_height - self.board_height) as i32 / 2)
                / self.scaling_factor as i32,
        );
        if ret.0 < 0
            || ret.1 < 0
            || ret.0 * self.scaling_factor as i32 >= self.board_width as i32
            || ret.1 * self.scaling_factor as i32 >= self.board_height as i32
        {
            None
        } else {
            // println!("Board {} {} Cursor {} {}", self.board_width, self.board_height, ret.0, ret.1);
            Some(ret)
        }
    }
    fn left_padding(&self) -> i32 {
        ((self.win_width - self.board_width) / 2) as i32 - 1
    }
    fn right_padding(&self) -> i32 {
        (self.win_width as i32 - self.left_padding())
            + ((self.win_width - self.board_width) % 2) as i32
    }
    fn top_padding(&self) -> i32 {
        ((self.win_height - self.board_height) / 2) as i32 - 1
    }
    fn bottom_padding(&self) -> i32 {
        (self.win_height as i32 - self.top_padding())
            + ((self.win_height - self.board_height) % 2) as i32
    }
    pub fn new(width: usize, height: usize, scaling_factor: usize) -> Self {
        Self {
            win_width: width,
            win_height: height,
            board_width: width - 240,
            board_height: height - 80,
            cursor: CursorKind::Square { size: 3 },
            scaling_factor,
        }
    }
    /// Draw the window content
    pub fn draw(
        &self,
        canvas: &mut WindowCanvas,
        texture: &mut Texture,
        world: &GameWorld,
    ) -> Result<(), String> {
        let left_padding: i32 = self.left_padding();
        let top_padding: i32 = self.top_padding();
        let right_padding: i32 = self.right_padding();
        let bottom_padding: i32 = self.bottom_padding();

        // Draw a border around the board
        canvas.set_draw_color(BOARD_BORDER_COLOR);
        canvas.draw_line(
            Point::from((left_padding, top_padding)),
            Point::from((right_padding, top_padding)),
        )?;
        canvas.draw_line(
            Point::from((right_padding, top_padding)),
            Point::from((right_padding, bottom_padding)),
        )?;
        canvas.draw_line(
            Point::from((right_padding, bottom_padding)),
            Point::from((left_padding, bottom_padding)),
        )?;
        canvas.draw_line(
            Point::from((left_padding, bottom_padding)),
            Point::from((left_padding, top_padding)),
        )?;

        // Draw the board
        texture.with_lock(
            Rect::from((0, 0, self.board_width as u32, self.board_height as u32)),
            |pixel_data, _pitch| {
                for board_y in 0..self.board_height / self.scaling_factor {
                    for board_x in 0..self.board_width / self.scaling_factor {
                        for x_scf in 0..self.scaling_factor {
                            for y_scf in 0..self.scaling_factor {
                                let win_x = board_x * self.scaling_factor + x_scf;
                                let win_y = board_y * self.scaling_factor + y_scf;
                                pixel_data[((win_y * self.board_width) + win_x) * 4 + 3] = 0xff;
                                pixel_data[((win_y * self.board_width) + win_x) * 4 + 2] =
                                    world.board()[board_x][board_y].color().r;
                                pixel_data[((win_y * self.board_width) + win_x) * 4 + 1] =
                                    world.board()[board_x][board_y].color().g;
                                pixel_data[((win_y * self.board_width) + win_x) * 4] =
                                    world.board()[board_x][board_y].color().b;
                            }
                        }
                    }
                }
            },
        )?;
        canvas.copy(
            texture,
            None,
            Rect::from((
                left_padding + 1,
                top_padding + 1,
                self.board_width as u32,
                self.board_height as u32,
            )),
        )?;

        Ok(())
    }
}

impl CursorKind {
    pub const fn ui_cursors() -> &'static [Self] {
        &[
            CursorKind::Square { size: 1 },
            CursorKind::Square { size: 2 },
            CursorKind::Square { size: 3 },
            CursorKind::Square { size: 5 },
            CursorKind::Square { size: 10 },
            CursorKind::Square { size: 15 },
            CursorKind::Square { size: 25 },
            CursorKind::Square { size: 50 },
        ]
    }
    /// The text to show on UI buttons for this cursor
    pub fn button_text(&self) -> String {
        match self {
            CursorKind::Square { size } => size.to_string(),
        }
    }
    /// The text to show on UI tooltips for this cursor
    pub fn tooltip_text(&self) -> String {
        match self {
            CursorKind::Square { size } => match size {
                1 => "A single pixel".to_string(),
                x => format!("A {}x{} square", x, x),
            },
        }
    }
    /// The category text to show in the UI for this cursor
    pub const fn category_text(&self) -> &'static str {
        match self {
            CursorKind::Square { .. } => "Square",
        }
    }
}
