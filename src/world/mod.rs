//   sandbox - World - tick functions and movement logic
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

use crate::element::{AIR_DENSITY, Element, ElementKind};
use crate::ui::{CursorKind, Ui};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{Dimensions, Point, Size};
use embedded_graphics::pixelcolor::raw::RawU32;
use embedded_graphics::prelude::{PixelColor, Primitive};
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, Rectangle};
use embedded_graphics::{Drawable, Pixel};
use rand::{Rng, RngCore};
use std::cmp::{Ordering, max};

mod transmute;
enum Move {
    /// Move the source element to the empty target location
    MoveElement {
        from_x: usize,
        from_y: usize,
        to_x: usize,
        to_y: usize,
    },
    /// Swap the given elements
    SwapElement {
        from_x: usize,
        from_y: usize,
        to_x: usize,
        to_y: usize,
    },
}
impl Move {
    pub fn same_dest_as(&self, other: &Move) -> bool {
        let (old_x, old_y) = match self {
            Move::MoveElement {
                from_x: _,
                from_y: _,
                to_x,
                to_y,
            } => (to_x, to_y),
            Move::SwapElement {
                from_x: _,
                from_y: _,
                to_x,
                to_y,
            } => (to_x, to_y),
        };
        match other {
            Move::MoveElement {
                from_x: _,
                from_y: _,
                to_x,
                to_y,
            } => to_x == old_x && to_y == old_y,
            Move::SwapElement {
                from_x: _,
                from_y: _,
                to_x,
                to_y,
            } => to_x == old_x && to_y == old_y,
        }
    }
}
pub struct GameWorld {
    /// The content of the game board.
    /// Must be at least as large as the viewport size, but may be larger.
    board: Vec<Vec<Element>>,
    /// The viewport width of the game board.
    /// All parts of the board outside the visible area (if e.g., a window is resized to a smaller size)
    /// are being paused until the window is resized again.
    width: usize,
    /// The viewport height of the game board
    /// All parts of the board outside the visible area (if e.g., a window is resized to a smaller size)
    ///  are being paused until the window is resized again.
    height: usize,
    /// All simulated element moves in one tick
    moves: Vec<Move>,
}

impl GameWorld {
    /// The width of the internal board data
    #[allow(dead_code)]
    pub fn board_width(&self) -> usize {
        self.board.len()
    }
    /// The height of the internal board data
    #[allow(dead_code)]
    pub fn board_height(&self) -> usize {
        self.board[0].len()
    }
    pub const fn viewport_height(&self) -> usize {
        self.height
    }
    pub const fn viewport_width(&self) -> usize {
        self.width
    }
    /// Destroy this board and return a resized copy.
    /// Resizes the viewport to the given size.
    /// If the new size is less than the old size, the pruned data will continue to exist,
    /// but not be simulated until the board is resized again.
    #[allow(dead_code)]
    pub(crate) fn resize(mut self, new_width: usize, new_height: usize) -> Self {
        let new_board_width = max(4, new_width);
        let new_board_height = max(4, new_height);
        let oldlen = self.board.len();
        for x in 0..new_board_width {
            if x >= oldlen {
                self.board.push(vec![Element::None; new_board_height]);
            }
            if (new_board_height - 1) >= self.board[x].len() {
                let oldln = new_board_height - self.board[x].len();
                self.board[x].append(&mut vec![Element::None; oldln])
            }
        }
        Self {
            board: self.board,
            width: new_width,
            height: new_height,
            moves: self.moves,
        }
    }
}
impl PixelColor for Element {
    type Raw = RawU32;
}
impl Dimensions for GameWorld {
    fn bounding_box(&self) -> Rectangle {
        Rectangle::new(
            Point::new(0, 0),
            Size::new(self.viewport_width() as u32, self.viewport_height() as u32),
        )
    }
}
impl DrawTarget for GameWorld {
    type Color = Element;
    type Error = String;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for pixel in pixels {
            if pixel.0.x >= 0
                && pixel.0.y >= 0
                && pixel.0.x < self.viewport_width() as i32
                && pixel.0.y < self.viewport_height() as i32
            {
                self.board[pixel.0.x as usize][pixel.0.y as usize] = pixel.1;
            }
        }
        Ok(())
    }
}

impl GameWorld {
    pub fn insert_element_at(
        &mut self,
        ui: &Ui,
        window_x: i32,
        window_y: i32,
        element: Element,
        prev_x: i32,
        prev_y: i32,
    ) {
        if let Some((x, y)) = ui.window_to_board_coordinate(window_x, window_y) {
            match ui.cursor() {
                CursorKind::Square { size } => {
                    Rectangle::with_center(Point::new(x, y), Size::new(*size, *size))
                        .into_styled(PrimitiveStyle::with_fill(element))
                        .draw(self)
                        .unwrap();
                },
                CursorKind::Circle { size } => {
                    Circle::with_center(Point::new(x, y), *size)
                        .into_styled(PrimitiveStyle::with_fill(element))
                        .draw(self)
                        .unwrap();
                },
                CursorKind::Pen { size } => {
                    if let Some((px, py)) = ui.window_to_board_coordinate(prev_x, prev_y)
                        && px != x
                        && py != y
                    {
                        Rectangle::with_center(Point::new(x, y), Size::new(*size, *size))
                            .into_styled(PrimitiveStyle::with_fill(element))
                            .draw(self)
                            .unwrap();
                        Rectangle::with_center(Point::new(px, py), Size::new(*size, *size))
                            .into_styled(PrimitiveStyle::with_fill(element))
                            .draw(self)
                            .unwrap();
                        Line::new(Point::new(x, y), Point::new(px, py))
                            .into_styled(PrimitiveStyle::with_stroke(element, *size))
                            .draw(self)
                            .unwrap();
                    } else {
                        Rectangle::with_center(Point::new(x, y), Size::new(*size, *size))
                            .into_styled(PrimitiveStyle::with_fill(element))
                            .draw(self)
                            .unwrap();
                    }
                },
            }
        }
    }
    /// Try to push a 'move down' to the moves vector and return true if that succeeded.
    fn move_down(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        match self.board[x][y].density() {
            None => false,
            Some(density) => {
                if density > AIR_DENSITY {
                    if y < (self.viewport_height() - 1) {
                        // Skip move with side spread probability
                        if rng.random_bool(self.board[x][y].spread_prob(&self.board[x][y + 1])) {
                            return false;
                        }
                        if self.board[x][y + 1] == Element::None {
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x,
                                to_y: y + 1,
                            });
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else if density <= AIR_DENSITY {
                    // The element is a gas or something with less density than air! Try to move up:
                    if y > 0 {
                        // Skip move with side spread probability
                        if rng.random_bool(self.board[x][y].spread_prob(&self.board[x][y - 1])) {
                            return false;
                        }
                        if self.board[x][y - 1] == Element::None {
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x,
                                to_y: y - 1,
                            });
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
        }
    }
    /// Try to push a 'swap down' to the moves vector and return true if that succeeded.
    fn swap_down(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        if y < (self.viewport_height() - 1) {
            // Only enable swaps if the bottom element is a liquid or gas!
            if !self.board[x][y + 1].is_liquid_or_gas() {
                return false;
            }
            // Skip move with side spread probability
            if rng.random_bool(self.board[x][y].spread_prob(&self.board[x][y + 1])) {
                return false;
            }
            let my_density = self.board[x][y].density();
            let other_density = self.board[x][y + 1].density();
            if let Some(a) = my_density {
                if let Some(b) = other_density {
                    let dens_q = b / a; // Density Quotient
                    if dens_q < 1. {
                        if rng.random_bool(1. - dens_q as f64) {
                            self.moves.push(Move::SwapElement {
                                from_x: x,
                                from_y: y,
                                to_x: x,
                                to_y: y + 1,
                            });
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
    /// Try to push a 'move down side' to the moves vector and return true if that succeeded.
    fn move_down_side(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        match self.board[x][y].density() {
            None => false,
            Some(density) => {
                if density > AIR_DENSITY {
                    if y < (self.viewport_height() - 1) {
                        let mut down_left = x > 0
                            && self.board[x - 1][y + 1] == Element::None
                            && (self.board[x - 1][y] == Element::None
                                || self.board[x][y + 1] == Element::None);
                        let mut down_right = x < (self.viewport_width() - 1)
                            && self.board[x + 1][y + 1] == Element::None
                            && (self.board[x + 1][y] == Element::None
                                || self.board[x][y + 1] == Element::None);
                        if down_left && down_right {
                            down_left = rng.random_bool(0.5);
                            down_right = !down_left;
                        }
                        if down_left {
                            // Skip move with side spread probability
                            if rng.random_bool(
                                self.board[x][y].spread_side_prob(&self.board[x - 1][y + 1]),
                            ) {
                                return false;
                            }
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x - 1,
                                to_y: y + 1,
                            });
                            return true;
                        }
                        if down_right {
                            // Skip move with side spread probability
                            if rng.random_bool(
                                self.board[x][y].spread_side_prob(&self.board[x + 1][y + 1]),
                            ) {
                                return false;
                            }
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x + 1,
                                to_y: y + 1,
                            });
                            return true;
                        }
                        false
                    } else {
                        false
                    }
                } else if density < AIR_DENSITY {
                    if y > 0 {
                        let mut up_left = x > 0
                            && self.board[x - 1][y - 1] == Element::None
                            && (self.board[x - 1][y] == Element::None
                                || self.board[x][y - 1] == Element::None);
                        let mut up_right = x < (self.viewport_width() - 1)
                            && self.board[x + 1][y - 1] == Element::None
                            && (self.board[x + 1][y] == Element::None
                                || self.board[x][y - 1] == Element::None);
                        if up_left && up_right {
                            up_left = rng.random_bool(0.5);
                            up_right = !up_left;
                        }
                        if up_left {
                            // Skip move with side spread probability
                            if rng.random_bool(
                                self.board[x][y].spread_side_prob(&self.board[x - 1][y - 1]),
                            ) {
                                return false;
                            }
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x - 1,
                                to_y: y - 1,
                            });
                            return true;
                        }
                        if up_right {
                            // Skip move with side spread probability
                            if rng.random_bool(
                                self.board[x][y].spread_side_prob(&self.board[x + 1][y - 1]),
                            ) {
                                return false;
                            }
                            self.moves.push(Move::MoveElement {
                                from_x: x,
                                from_y: y,
                                to_x: x + 1,
                                to_y: y - 1,
                            });
                            return true;
                        }
                        false
                    } else {
                        false
                    }
                } else {
                    false
                }
            },
        }
    }
    /// Try to push a 'swap down side' to the moves vector and return true if that succeeded.
    fn swap_down_side(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        // Make down-side swaps a little less probable:
        let prob_quot = 0.75;
        if y < (self.viewport_height() - 1) {
            let my_density = self.board[x][y].density();
            let mut density_down_left = if x == 0
                || !self.board[x - 1][y + 1].is_liquid_or_gas()
                || (self.board[x - 1][y] != Element::None && self.board[x][y + 1] != Element::None)
            {
                None
            } else {
                self.board[x - 1][y + 1].density()
            };
            let mut density_down_right = if x == (self.viewport_width() - 1)
                || !self.board[x + 1][y + 1].is_liquid_or_gas()
                || (self.board[x + 1][y] != Element::None && self.board[x][y + 1] != Element::None)
            {
                None
            } else {
                self.board[x + 1][y + 1].density()
            };
            if let Some(a) = my_density {
                if let Some(b) = density_down_right
                    && b >= a
                {
                    density_down_right = None;
                }
                if let Some(b) = density_down_left
                    && b >= a
                {
                    density_down_left = None;
                }
                if density_down_left.is_some() && density_down_right.is_some() {
                    let left = rng.random_bool(0.5);
                    if left {
                        density_down_right = None;
                    } else {
                        density_down_left = None;
                    }
                }
                if let Some(b) = density_down_left {
                    let density_quot = b / a;
                    debug_assert!(density_quot < 1.);
                    if rng.random_bool((1. - density_quot) as f64 * prob_quot) {
                        // Skip move with side spread probability
                        if rng.random_bool(
                            self.board[x][y].spread_side_prob(&self.board[x - 1][y + 1]),
                        ) {
                            return false;
                        }
                        self.moves.push(Move::SwapElement {
                            from_x: x,
                            from_y: y,
                            to_x: x - 1,
                            to_y: y + 1,
                        });
                        return true;
                    }
                }
                if let Some(b) = density_down_right {
                    let density_quot = b / a;
                    debug_assert!(density_quot < 1.);
                    if rng.random_bool((1. - density_quot) as f64 * prob_quot) {
                        // Skip move with side spread probability
                        if rng.random_bool(
                            self.board[x][y].spread_side_prob(&self.board[x + 1][y + 1]),
                        ) {
                            return false;
                        }
                        self.moves.push(Move::SwapElement {
                            from_x: x,
                            from_y: y,
                            to_x: x + 1,
                            to_y: y + 1,
                        });
                        return true;
                    }
                }
            }
        }
        false
    }
    /// Try to push a 'move side' to the moves vector and return true if that succeeded.
    fn move_side(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        let mut left = x > 0 && self.board[x - 1][y] == Element::None;
        let mut right = x < (self.viewport_width() - 1) && self.board[x + 1][y] == Element::None;
        if left && right {
            left = rng.random_bool(0.5);
            right = !left;
        }
        if left {
            self.moves.push(Move::MoveElement {
                from_x: x,
                from_y: y,
                to_x: x - 1,
                to_y: y,
            });
            return true;
        }
        if right {
            self.moves.push(Move::MoveElement {
                from_x: x,
                from_y: y,
                to_x: x + 1,
                to_y: y,
            });
            return true;
        }
        false
    }
    /// Tick (Calculate the next iteration of this board in-place)
    pub fn tick(&mut self, rng: &mut dyn RngCore) {
        self.moves.clear();
        let height = self.viewport_height();
        let width = self.viewport_width();
        // First, perform a 'Decay' pass - This will decay all decaying elements with a certain probability.
        for y in 0..height {
            for x in 0..width {
                self.decay(x, y, rng);
            }
        }

        // Then, perform a 'Transmute' pass! This will transmute all applicable elements in-place
        for y in 0..height {
            for x in 0..width {
                self.transmute(x, y, rng);
            }
        }

        // Then, collect and perform all moves:
        for y in 0..height {
            for x in 0..width {
                // Gravity
                if y != height - 1 {
                    match self.board[x][y].kind() {
                        ElementKind::None => {},
                        ElementKind::Solid => {},
                        ElementKind::Powder { .. } => {
                            if !self.move_down(x, y, rng) {
                                if !self.move_down_side(x, y, rng) {
                                    if !self.swap_down(x, y, rng) {
                                        self.swap_down_side(x, y, rng);
                                    }
                                }
                            }
                        },
                        ElementKind::Liquid { .. } => {
                            if !self.move_down(x, y, rng) {
                                if !self.move_down_side(x, y, rng) {
                                    if !self.swap_down(x, y, rng) {
                                        if !self.swap_down_side(x, y, rng) {
                                            self.move_side(x, y, rng);
                                        }
                                    }
                                }
                            }
                        },
                        ElementKind::Gas { .. } => {
                            if !self.move_down(x, y, rng) {
                                if !self.move_side(x, y, rng) {
                                    if !self.move_down_side(x, y, rng) {
                                        if !self.swap_down_side(x, y, rng) {
                                            self.swap_down(x, y, rng);
                                        }
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
        // Thanks, https://winter.dev/articles/falling-sand , for this algorithm

        // Remove all filled cells from possible moves
        // let mut i = 0;
        // while i < self.moves.len() {
        //     let Move::MoveElement {
        //         from_x,
        //         from_y,
        //         to_x,
        //         to_y,
        //     } = self.moves[i];
        //     debug_assert_ne!(self.board[from_x][from_y], Element::None);
        //     if self.board[to_x][to_y] != Element::None {
        //         self.moves[i] = self.moves.pop().unwrap();
        //     } else {
        //         i += 1;
        //     }
        // }

        // Sort moves by destination
        self.moves.sort_unstable_by(|m1, m2| {
            let (x1, y1) = match m1 {
                Move::MoveElement {
                    from_x: _,
                    from_y: _,
                    to_x,
                    to_y,
                } => (to_x, to_y),
                Move::SwapElement {
                    from_x: _,
                    from_y: _,
                    to_x,
                    to_y,
                } => (to_x, to_y),
            };
            match m2 {
                Move::MoveElement {
                    from_x: _,
                    from_y: _,
                    to_x,
                    to_y,
                } => match to_x.cmp(x1) {
                    Ordering::Equal => to_y.cmp(y1),
                    x => x,
                },
                Move::SwapElement {
                    from_x: _,
                    from_y: _,
                    to_x,
                    to_y,
                } => match to_x.cmp(x1) {
                    Ordering::Equal => to_y.cmp(y1),
                    x => x,
                },
            }
        });

        // Commit moves. If multiple moves into one single destination are possible, select a random one
        self.moves.push(Move::MoveElement {
            from_x: self.viewport_width(),
            from_y: self.viewport_height(),
            to_x: self.viewport_width(),
            to_y: self.viewport_height(),
        });
        let mut prev_i = 0usize;
        for i in 0..(self.moves.len() - 1) {
            if !self.moves[i].same_dest_as(&self.moves[i + 1]) {
                let random_choice = rng.random_range(prev_i..=i);
                // Execute the randomly chosen move
                match self.moves[random_choice] {
                    Move::MoveElement {
                        from_x,
                        from_y,
                        to_x,
                        to_y,
                    } => {
                        debug_assert_ne!(self.board[from_x][from_y], Element::None);
                        debug_assert_eq!(self.board[to_x][to_y], Element::None);
                        self.board[to_x][to_y] = self.board[from_x][from_y];
                        self.board[from_x][from_y] = Element::None;
                    },
                    Move::SwapElement {
                        from_x,
                        from_y,
                        to_x,
                        to_y,
                    } => {
                        debug_assert_ne!(self.board[from_x][from_y], Element::None);
                        debug_assert_ne!(self.board[to_x][to_y], Element::None);
                        let b = self.board[to_x][to_y];
                        self.board[to_x][to_y] = self.board[from_x][from_y];
                        self.board[from_x][from_y] = b;
                    },
                }
                prev_i = i + 1;
            }
        }
    }
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            board: vec![vec![Element::None; height]; width],
            moves: Vec::new(),
            width,
            height,
        }
    }
    pub fn board(&self) -> &Vec<Vec<Element>> {
        &self.board
    }
}

#[cfg(test)]
mod tests {
    use crate::element::Element;
    use crate::world::GameWorld;
    use rand::SeedableRng;
    use rand_xorshift::XorShiftRng;

    #[test]
    fn test_fall_tick_simple() {
        let mut board = GameWorld::new(3, 3);
        let mut rng = XorShiftRng::seed_from_u64(0);
        board.board[1][1] = Element::Sand;
        board.tick(&mut rng);
        assert_eq!(board.board[1][1], Element::None);
        assert_eq!(board.board[1][2], Element::Sand);
    }
    #[test]
    fn test_fall_tick_stacked() {
        let mut board = GameWorld::new(3, 3);
        let mut rng = XorShiftRng::seed_from_u64(0);
        board.board[1][0] = Element::Sand;
        board.board[1][1] = Element::Sand;
        board.tick(&mut rng);
        assert_eq!(board.board[1][0], Element::None);
        assert_eq!(board.board[1][1], Element::None);
        assert_eq!(board.board[1][2], Element::Sand);
    }
}
