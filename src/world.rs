use crate::element::{Element, ElementKind};
use crate::ui::Ui;
use rand::{Rng, RngCore};
use std::cmp::{Ordering, max, min};

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
    board: Vec<Vec<Element>>,
    moves: Vec<Move>,
}

impl GameWorld {
    pub fn insert_element_at(&mut self, ui: &Ui, window_x: i32, window_y: i32, element: Element) {
        if let Some((x, y)) = ui.window_to_board_coordinate(window_x, window_y) {
            for drw_y in (max(0, y - (ui.cursor_size() - 1)))
                ..=(min(self.board[0].len() as i32 - 1, y + ui.cursor_size()))
            {
                for drw_x in max(0, x - ui.cursor_size())
                    ..=min(self.board.len() as i32 - 1, x + ui.cursor_size())
                {
                    self.board[drw_x as usize][drw_y as usize] = element;
                }
            }
        }
    }
    /// Try to push a 'move down' to the moves vector and return true if that succeeded.
    fn move_down(&mut self, x: usize, y: usize, _rng: &mut dyn RngCore) -> bool {
        if y < (self.board[0].len() - 1) {
            if self.board[x][y + 1] == Element::None {
                self.moves.push(Move::MoveElement {
                    from_x: x,
                    from_y: y,
                    to_x: x,
                    to_y: y + 1,
                });
                return true;
            }
        }
        false
    }
    /// Try to push a 'swap down' to the moves vector and return true if that succeeded.
    fn swap_down(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        if y < (self.board[0].len() - 1) {
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
        if y < (self.board[0].len() - 1) {
            let mut down_left = x > 0 && self.board[x - 1][y + 1] == Element::None;
            let mut down_right =
                x < (self.board.len() - 1) && self.board[x + 1][y + 1] == Element::None;
            if down_left && down_right {
                down_left = rng.random_bool(0.5);
                down_right = !down_left;
            }
            if down_left {
                self.moves.push(Move::MoveElement {
                    from_x: x,
                    from_y: y,
                    to_x: x - 1,
                    to_y: y + 1,
                });
                return true;
            }
            if down_right {
                self.moves.push(Move::MoveElement {
                    from_x: x,
                    from_y: y,
                    to_x: x + 1,
                    to_y: y + 1,
                });
                return true;
            }
        }
        false
    }
    /// Try to push a 'swap down side' to the moves vector and return true if that succeeded.
    fn swap_down_side(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) -> bool {
        // Make down-side swaps a little less probable:
        let prob_quot = 0.75;
        if y < (self.board[0].len() - 1) {
            let my_density = self.board[x][y].density();
            let mut density_down_left = if x == 0 {
                None
            } else {
                self.board[x - 1][y + 1].density()
            };
            let mut density_down_right = if x == (self.board.len() - 1) {
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
        let mut right = x < (self.board.len() - 1) && self.board[x + 1][y] == Element::None;
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
        let height = self.board[0].len();
        let width = self.board.len();
        for y in 0..height {
            for x in 0..width {
                // new_board[x][y] = self.board[x][y];
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
                                    self.move_side(x, y, rng);
                                }
                            }
                        },
                        // ElementKind::Gas { .. } => {},
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
            from_x: self.board.len(),
            from_y: self.board[0].len(),
            to_x: self.board.len(),
            to_y: self.board[0].len(),
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
