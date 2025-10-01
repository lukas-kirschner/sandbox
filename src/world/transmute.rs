use crate::element::{Element, ElementKind};
use crate::world::GameWorld;
use rand::{Rng, RngCore};

enum Transmutation {
    None,
    WithProbability {
        probability: f64,
        outcome_a: Option<Element>,
        outcome_b: Option<Element>,
    },
}
/// Given adjacent elements a and b, can a transmute b?
fn can_transmute(a: &Element, b: &Element) -> Transmutation {
    match a {
        Element::None => Transmutation::None,
        Element::BrickWall => Transmutation::None,
        Element::Sand => Transmutation::None,
        Element::Salt => match b {
            // Salt and water transforms into saltwater:
            Element::Water => Transmutation::WithProbability {
                probability: 0.05,
                outcome_a: None,
                outcome_b: Some(Element::SaltWater),
            },
            _ => Transmutation::None,
        },
        Element::Water => Transmutation::None,
        Element::SaltWater => Transmutation::None,
        Element::WaterSource => match b {
            // Water Source spawns water
            Element::None => Transmutation::WithProbability {
                probability: 0.015,
                outcome_a: Some(Element::WaterSource),
                outcome_b: Some(Element::Water),
            },
            _ => Transmutation::None,
        },
        Element::Hydrogen => Transmutation::None,
        Element::Steam => match b {
            // Low probability to condensate in air
            Element::None => Transmutation::WithProbability {
                probability: 0.00005,
                outcome_a: Some(Element::Water),
                outcome_b: None,
            },
            // Condensate when touching walls
            e if matches!(e.kind(), ElementKind::Solid) => Transmutation::WithProbability {
                probability: 0.001,
                outcome_a: Some(Element::Water),
                outcome_b: Some(*e),
            },
            _ => Transmutation::None,
        },
        Element::Dust => match b{
            // Transforms to wet dust in water
            Element::Water => Transmutation::WithProbability {
                probability: 0.005,
                outcome_a: Some(Element::WetDust),
                outcome_b: None,
            },
            // Transforms to salt and wet dust in salt water
            Element::SaltWater => Transmutation::WithProbability {
                probability: 0.005,
                outcome_a: Some(Element::Salt),
                outcome_b: Some(Element::WetDust),
            },
            _ => Transmutation::None,
        },
        Element::WetDust => match b{
            // Has a small chance of 'bleeding' water
            Element::None => Transmutation::WithProbability {
                probability: 0.001,
                outcome_a: Some(Element::Dust),
                outcome_b: Some(Element::Water),
            },
            _ => Transmutation::None,
        },
    }
}

impl GameWorld {
    /// Randomly transmute elements in the world
    pub(in crate::world) fn transmute(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) {
        let height = self.board[0].len();
        let width = self.board.len();
        // Make it most probable to transform the top, then left right, then bottom elements.
        // Important for fire!
        let probes = [(0, -1), (-1, 0), (1, 0), (0, 1)];
        for (x_offs, y_offs) in probes {
            let b_x = x as i32 + x_offs;
            let b_y = y as i32 + y_offs;
            if b_x < 0 || b_x >= width as i32 || b_y < 0 || b_y >= height as i32 {
                continue;
            }
            match can_transmute(&self.board[x][y], &self.board[b_x as usize][b_y as usize]) {
                Transmutation::None => continue,
                Transmutation::WithProbability {
                    probability,
                    outcome_a,
                    outcome_b,
                } => {
                    if rng.random_bool(probability) {
                        // Transmute!
                        self.board[x][y] = outcome_a.unwrap_or(Element::None);
                        self.board[b_x as usize][b_y as usize] = outcome_b.unwrap_or(Element::None);
                    }
                },
            }
        }
    }
}
