use crate::element::Element;
use crate::ui::Ui;
use std::cmp::{max, min};

pub struct GameWorld {
    board: Vec<Vec<Element>>,
}

impl GameWorld {
    pub fn insert_element_at(&mut self, ui: &Ui, window_x: i32, window_y: i32, element: Element) {
        if let Some((x, y)) = ui.window_to_board_coordinate(window_x, window_y) {
            for drw_y in (max(0, y - ui.cursor_size()))
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
    /// Tick (Calculate the next iteration of this board, cloning the complete board state
    pub fn tick(&self) -> Self {
        let height = self.board[0].len();
        let width = self.board.len();
        let mut new_board = vec![vec![Element::None; height]; width];
        for y in (0..height).rev() {
            for x in 0..width {
                // new_board[x][y] = self.board[x][y];
                // Gravity
                if y == height - 1 {
                    new_board[x][y] = self.board[x][y];
                } else {
                    if self.board[x][y] != Element::None {
                        if new_board[x][y + 1] == Element::None {
                            // Fall down
                            new_board[x][y + 1] = self.board[x][y];
                            new_board[x][y] = Element::None;
                        } else {
                            // Collision
                            new_board[x][y] = self.board[x][y];
                        }
                    }
                }
            }
        }
        Self { board: new_board }
    }
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            board: vec![vec![Element::None; height]; width],
        }
    }
    pub fn board(&self) -> &Vec<Vec<Element>> {
        &self.board
    }
}
