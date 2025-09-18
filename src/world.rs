use crate::element::Element;
use crate::ui::Ui;

pub struct GameWorld {
    board: Vec<Vec<Element>>
}

impl GameWorld {
    pub(crate) fn insert_element_at(&mut self, ui: &Ui, window_x: i32, window_y: i32, element: Element) {
        if let Some((x,y)) = ui.window_to_board_coordinate(window_x,window_y){
            self.board[x as usize][y as usize] = element;
        }
    }
}

impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {board: vec![vec![Element::None;height];width]}
    }
    pub fn board(&self) -> &Vec<Vec<Element>> {
        &self.board
    }
}
