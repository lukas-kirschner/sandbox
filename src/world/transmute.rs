//   sandbox - World - decay and transmute logic
//   Copyright (C) 2025 Lukas Kirschner
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

use crate::element::{Element, ElementKind, Flammability};
use crate::world::GameWorld;
use rand::prelude::IndexedRandom;
use rand::{Rng, RngCore};

enum Transmutation {
    None,
    /// An element a has a chance of transforming into A and B
    WithProbability {
        probability: f64,
        outcome_a: Option<Element>,
        outcome_b: Option<Element>,
    },
    /// An element transforms into one or many of A and exactly one of B
    WithProbabilityOfMultipleA {
        probability: f64,
        outcome_a: Vec<Element>,
        outcome_b: Option<Element>,
    },
    /// An element transforms into one or many of B and exactly one of A
    WithProbabilityOfMultipleB {
        probability: f64,
        outcome_b: Vec<Element>,
        outcome_a: Option<Element>,
    },
}

const LIQUID_SOURCE_SPAWN_PROBABILITY: f64 = 0.015;
const GAS_SOURCE_SPAWN_PROBABILITY: f64 = 0.025;

/// Given adjacent elements a and b, can a transmute b?
fn can_transmute(a: &Element, b: &Element) -> Transmutation {
    match a {
        Element::None => Transmutation::None,
        Element::BrickWall => Transmutation::None,
        Element::Wood => Transmutation::None,
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
        Element::Water => match b {
            // Water extinguishes Flames
            Element::Flame => Transmutation::WithProbability {
                probability: 0.1,
                outcome_a: None,
                outcome_b: Some(Element::Steam),
            },
            // Water extinguishes burning elements
            Element::BurningParticle {
                flame_spawn_prob, ..
            } => Transmutation::WithProbability {
                probability: (1.0 - flame_spawn_prob) * 0.1,
                outcome_a: None,
                outcome_b: Some(Element::Steam),
            },
            // Water boils when touching hot surfaces
            Element::FireSource => Transmutation::WithProbability {
                probability: 0.01,
                outcome_a: Some(Element::Steam),
                outcome_b: Some(Element::FireSource),
            },
            // Water boils immediately when touching very hot surfaces
            Element::Volcano => Transmutation::WithProbability {
                probability: 0.1,
                outcome_a: Some(Element::Steam),
                outcome_b: Some(Element::Volcano),
            },
            _ => Transmutation::None,
        },
        Element::SaltWater => match b {
            // Salt Water extinguishes Flames
            Element::Flame => Transmutation::WithProbability {
                probability: 0.1,
                outcome_a: Some(Element::Salt),
                outcome_b: Some(Element::Steam),
            },
            // Salt Water boils when touching hot surfaces, having a chance of turning into salt or steam
            Element::FireSource => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.01,
                outcome_a: vec![Element::Steam, Element::Salt],
                outcome_b: Some(Element::FireSource),
            },
            // Salt Water boils immediately when touching very hot surfaces
            Element::Volcano => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.1,
                outcome_a: vec![Element::Steam, Element::Salt],
                outcome_b: Some(Element::Volcano),
            },
            _ => Transmutation::None,
        },

        Element::WaterSource => match b {
            // Water Source spawns water
            Element::None => Transmutation::WithProbability {
                probability: LIQUID_SOURCE_SPAWN_PROBABILITY,
                outcome_a: Some(Element::WaterSource),
                outcome_b: Some(Element::Water),
            },
            _ => Transmutation::None,
        },
        Element::GasolineSource => match b {
            // Gasoline Source spawns gasoline
            Element::None => Transmutation::WithProbability {
                probability: LIQUID_SOURCE_SPAWN_PROBABILITY,
                outcome_a: Some(Element::GasolineSource),
                outcome_b: Some(Element::Gasoline),
            },
            _ => Transmutation::None,
        },
        Element::HydrogenBurner => match b {
            // Hydrogen Source spawns hydrogen
            Element::None => Transmutation::WithProbability {
                probability: GAS_SOURCE_SPAWN_PROBABILITY,
                outcome_a: Some(Element::HydrogenBurner),
                outcome_b: Some(Element::Hydrogen),
            },
            _ => Transmutation::None,
        },
        Element::MethaneBurner => match b {
            // Methane Source spawns methane
            Element::None => Transmutation::WithProbability {
                probability: GAS_SOURCE_SPAWN_PROBABILITY,
                outcome_a: Some(Element::MethaneBurner),
                outcome_b: Some(Element::Methane),
            },
            _ => Transmutation::None,
        },
        Element::FireSource => match b {
            // Fire Source spawns flames
            Element::None => Transmutation::WithProbability {
                probability: 0.085,
                outcome_a: Some(Element::FireSource),
                outcome_b: Some(Element::Flame),
            },
            _ => Transmutation::None,
        },
        Element::Volcano => match b {
            // Volcano spawns flames and lava (low probability for lava)
            Element::None | Element::Flame => Transmutation::WithProbabilityOfMultipleB {
                probability: 0.004,
                outcome_a: Some(Element::Volcano),
                outcome_b: vec![
                    Element::Flame,
                    Element::Flame,
                    Element::Flame,
                    Element::Lava,
                ],
            },
            // Volcano immediately ignites all flammable elements
            e => match e.flammability() {
                Flammability::NotFlammable => Transmutation::None,
                Flammability::Flammable {
                    prob,
                    decay_prob,
                    flame_spawn_prob,
                } => Transmutation::WithProbability {
                    probability: (prob * 1.5).min(1.0),
                    outcome_a: Some(*a),
                    outcome_b: Some(Element::BurningParticle {
                        burned_element_kind: b.kind(),
                        decay_prob,
                        flame_spawn_prob,
                        spawns_ash: matches!(e, Element::Wood),
                    }),
                },
            },
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
        Element::Dust => match b {
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
        Element::WetDust => match b {
            // Has a small chance of 'bleeding' water
            Element::None => Transmutation::WithProbability {
                probability: 0.001,
                outcome_a: Some(Element::Dust),
                outcome_b: Some(Element::Water),
            },
            // Is very good at extinguishing fire
            Element::Flame => Transmutation::WithProbability {
                probability: 0.05,
                outcome_a: Some(Element::WetDust),
                outcome_b: None,
            },
            // The water evaporates when touching hot surfaces
            Element::FireSource => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.01,
                outcome_a: vec![Element::Dust, Element::Steam],
                outcome_b: Some(Element::FireSource),
            },
            Element::Volcano => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.04,
                outcome_a: vec![Element::Dust, Element::Steam],
                outcome_b: Some(Element::Volcano),
            },
            _ => Transmutation::None,
        },
        // Handle all flammable elements here:
        Element::Flame => match b.flammability() {
            Flammability::NotFlammable => Transmutation::None,
            Flammability::Flammable {
                prob,
                decay_prob,
                flame_spawn_prob,
            } => Transmutation::WithProbability {
                probability: prob,
                outcome_a: None,
                outcome_b: Some(Element::BurningParticle {
                    burned_element_kind: b.kind(),
                    decay_prob,
                    flame_spawn_prob,
                    spawns_ash: matches!(b, Element::Wood),
                }),
            },
        },
        Element::BurningParticle {
            flame_spawn_prob, ..
        } => match b {
            // Burning Particles spawn flames
            Element::None => Transmutation::WithProbability {
                probability: *flame_spawn_prob,
                outcome_a: Some(*a),
                outcome_b: Some(Element::Flame),
            },
            // Burning particles can light other burning particles on fire, but with a lower probability than flames.
            e => match e.flammability() {
                Flammability::NotFlammable => Transmutation::None,
                Flammability::Flammable {
                    prob,
                    decay_prob,
                    flame_spawn_prob,
                } => Transmutation::WithProbability {
                    probability: prob * 0.5,
                    outcome_a: Some(*a),
                    outcome_b: Some(Element::BurningParticle {
                        burned_element_kind: b.kind(),
                        decay_prob,
                        flame_spawn_prob,
                        spawns_ash: matches!(e, Element::Wood),
                    }),
                },
            },
        },
        Element::Gasoline => Transmutation::None,
        Element::Methane => Transmutation::None,
        Element::Ash => match b {
            // Ash has a small chance of despawning when touching a flame
            Element::Flame => Transmutation::WithProbability {
                probability: 0.1,
                outcome_a: None,
                outcome_b: Some(Element::Flame),
            },
            // Ash has a small chance of dissolving in water and salt water
            Element::Water => Transmutation::WithProbability {
                probability: 0.001,
                outcome_a: None,
                outcome_b: Some(*b),
            },
            // Ash has a small chance of dissolving in water and salt water
            Element::SaltWater => Transmutation::WithProbability {
                probability: 0.001,
                outcome_a: None,
                outcome_b: Some(*b),
            },
            _ => Transmutation::None,
        },
        Element::Sink => {
            // Sink destroys everything except solids
            match b.kind() {
                ElementKind::None => Transmutation::None,
                ElementKind::Solid => Transmutation::None,
                _ => Transmutation::WithProbability {
                    probability: 0.3,
                    outcome_a: Some(Element::Sink),
                    outcome_b: None,
                },
            }
        },
        Element::ColdLava => Transmutation::None,
        Element::Lava => match b {
            // Lava evaporates water and has a high chance of turning into cold lava
            Element::Water => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.5,
                outcome_a: vec![Element::Lava, Element::ColdLava, Element::ColdLava],
                outcome_b: Some(Element::Steam),
            },
            // Lava evaporates Salt Water and might turn into salt in the process
            Element::SaltWater => Transmutation::WithProbabilityOfMultipleA {
                probability: 0.5,
                outcome_a: vec![
                    Element::Lava,
                    Element::ColdLava,
                    Element::ColdLava,
                    Element::Salt,
                ],
                outcome_b: Some(Element::Steam),
            },
            // Lava has a very small chance of emitting flames
            Element::None => Transmutation::WithProbability {
                probability: 0.0005,
                outcome_a: Some(*a),
                outcome_b: Some(Element::Flame),
            },
            // Lava has a (small) chance of lighting flammable elements on fire
            e => match e.flammability() {
                Flammability::NotFlammable => Transmutation::None,
                Flammability::Flammable {
                    prob,
                    decay_prob,
                    flame_spawn_prob,
                } => Transmutation::WithProbability {
                    probability: prob * 0.1,
                    outcome_a: Some(*a),
                    outcome_b: Some(Element::BurningParticle {
                        burned_element_kind: b.kind(),
                        decay_prob,
                        flame_spawn_prob,
                        spawns_ash: matches!(e, Element::Wood),
                    }),
                },
            },
        },
    }
}

impl GameWorld {
    /// Decay all decaying elements in the world
    pub(in crate::world) fn decay(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) {
        if let Some(decay_prob) = self.board[x][y].decay_prob() {
            if rng.random_bool(decay_prob) {
                // Decay the element
                match self.board[x][y].decays_to() {
                    None => self.board[x][y] = Element::None,
                    Some(e) => self.board[x][y] = e,
                }
            }
        }
    }
    /// Randomly transmute elements in the world
    pub(in crate::world) fn transmute(&mut self, x: usize, y: usize, rng: &mut dyn RngCore) {
        let height = self.viewport_height();
        let width = self.viewport_width();
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
                Transmutation::WithProbabilityOfMultipleA {
                    probability,
                    outcome_a,
                    outcome_b,
                } => {
                    if rng.random_bool(probability) {
                        self.board[x][y] = *outcome_a.choose(rng).unwrap_or(&Element::None);
                        self.board[b_x as usize][b_y as usize] = outcome_b.unwrap_or(Element::None);
                    }
                },
                Transmutation::WithProbabilityOfMultipleB {
                    probability,
                    outcome_a,
                    outcome_b,
                } => {
                    if rng.random_bool(probability) {
                        self.board[x][y] = outcome_a.unwrap_or(Element::None);
                        self.board[b_x as usize][b_y as usize] =
                            *outcome_b.choose(rng).unwrap_or(&Element::None);
                    }
                },
            }
        }
    }
}
