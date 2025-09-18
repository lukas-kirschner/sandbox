use crate::element::Element;

pub struct GameWorld {
    board: Vec<Vec<Element>>
}
impl GameWorld {
    pub fn new(width: usize, height: usize) -> Self {
        Self {board: vec![vec![Element::None;height];width]}
    }
    pub fn board(&self) -> &Vec<Vec<Element>> {
        &self.board
    }
}
