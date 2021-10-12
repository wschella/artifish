// Blub

use decorum::NotNan;
use rand_chacha::ChaCha20Rng;
use rand::Rng;

use crate::{GREEN, MAX_X, MAX_Y, MOVE_COST, MOVE_SPEED, languages::lang::{Program, run_fish}, state::State, vec2::Vec2};

use super::languages::lang::*;
use super::vec2::*;

pub type Energy = NotNan<f64>;

#[derive(Clone)]
pub struct Fish {
    pub x: f64,
    pub y: f64,
    pub energy: Energy,
    pub program: Program,
    pub color: [f32; 4],
    pub is_man_made: bool,
    pub tag: Option<String>,
}

pub fn behave_fishes(state: &mut State, delta_time: f64) {
    let fishes = &mut state.fishes;
    for i in 0..fishes.len() {
        let action = run_fish(fishes, i);
        execute_fish_action(&mut fishes[i], action, delta_time);
    }
}

impl Fish {
    pub fn new(x: f64, y: f64, energy: Energy, program: Program) -> Self {
        Fish {
            x,
            y,
            energy,
            program,
            color: GREEN,
            is_man_made: false,
            tag: None,
        }
    }
}

impl Fish {
    pub fn radius(&self) -> f64 {
        (self.energy / std::f64::consts::PI).into_inner().cbrt()
    }

    pub fn surface_area(&self) -> f64 {
        self.radius().powi(2) * std::f64::consts::PI
    }

    pub fn distance(&self, other: &Fish) -> f64 {
        self.displacement_to(other).length()
    }
    // Verplaatsingsvector naar. Ha. Blub. Blub. I'm coming to get you.
    pub fn displacement_to(&self, other: &Fish) -> Vec2 {
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        return Vec2::new(dx, dy);
    }
    pub fn direction_to(&self, other: &Fish) -> Vec2 {
        let displacement_to = self.displacement_to(other);
        // TODO: This is not what we really want, we should return an option or smth
        if displacement_to.length() == 0.0 {
            return Vec2::zero();
        }
        displacement_to / displacement_to.length()
    }

    pub fn covers(&self, other: &Fish) -> bool {
        self.radius() > self.distance(other)
    }

    pub fn eat(&mut self, other: &Fish) {
        self.energy += other.energy;
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
    }

    pub fn move_by(&mut self, direction: &Vec2) {
        self.x += direction.x;
        self.y += direction.y;
    }

    pub fn reproduce(&mut self, rng: &mut ChaCha20Rng) -> Fish {
        let ax = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
        let opposite = ax - std::f64::consts::PI;

        let radius = self.radius();

        let x_1 = self.x + radius * ax.cos();
        let y_1 = self.y + radius * ax.sin();
        let x_2 = self.x + radius * opposite.cos();
        let y_2 = self.y + radius * opposite.sin();

        // self.energy -= SPLIT_COST;
        let child_energy = self.energy / 5.0;
        self.move_to(x_1, y_1);
        self.energy -= child_energy * 2.0;

        let child_program = if self.is_man_made {
            self.program.clone()
        } else {
            self.program.mutate(rng)
        };

        Fish {
            x: x_2,
            y: y_2,
            energy: child_energy,
            program: child_program,
            color: self.color,
            is_man_made: self.is_man_made,
            tag: self.tag.clone(),
        }
    }
}



#[derive(Copy, Clone, Debug)]
pub enum Action {
    Pass,
    Move(Vec2),
}

pub fn execute_fish_action(fish: &mut Fish, action: Action, delta_time: f64) {
    use Action::*;
    match action {
        Move(direction) => {
            let displacement = direction * delta_time * MOVE_SPEED;
            fish.move_by(&displacement);
            fish.move_to(fish.x.clamp(0.0, MAX_X), fish.y.clamp(0.0, MAX_Y));
            // neutral if: energy * distance.powi(2) * move_cost = surface_area * growth_factor
            // with surface = energy.cuberoot().powi(2)
            // -> neutral distance = sqrt(surface_area * growth_factor * 1/move_cost * 1/energy)
            fish.energy -= fish.energy * displacement.length().powi(2) * MOVE_COST;
        }
        Pass => (),
    }
}
