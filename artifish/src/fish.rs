// Blub

use decorum::{NotNan, N64};
use rand::Rng;
use rand_chacha::ChaCha20Rng;

use crate::{
    lang::{Fraction, InterpreterState, Program},
    state::State,
    vec2::Vec2,
    GREEN, IMPULSE_COST,
};

pub type Energy = NotNan<f64>;

#[derive(Clone)]
pub struct Fish {
    pub x: f64,
    pub y: f64,
    pub energy: Energy,
    pub velocity: Vec2,
    pub program: Program,
    pub color: [f32; 4],
    pub is_man_made: bool,
    pub tag: Option<String>,
}

pub fn behave_fishes(state: &mut State, delta_time: f64) {
    let fishes = &mut state.fishes;
    for i in 0..fishes.len() {
        let state = InterpreterState {
            fishes,
            fish_num: i,
        };
        let action = fishes[i].program.run(&state);
        execute_fish_action(&mut fishes[i], action, delta_time);
    }
}

impl Fish {
    #[allow(dead_code)]
    pub fn new(x: f64, y: f64, energy: Energy, program: Program) -> Self {
        Fish {
            x,
            y,
            energy,
            program,
            velocity: Vec2::new(0.0, 0.0),
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

    pub fn mass(&self) -> f64 {
        self.energy.into_inner()
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

    pub fn apply_impulse(&mut self, force: Vec2) {
        let acceleration = force / self.mass();
        self.velocity += acceleration;
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

        let mut child_program = self.program.clone();
        if !self.is_man_made {
            child_program.mutate(rng);
        };

        Fish {
            x: x_2,
            y: y_2,
            energy: child_energy,
            velocity: Vec2::new(0.0, 0.0),
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
    SetVelocity(Vec2, Fraction),
}

const FORCE_MULTIPLIER: f64 = 1.0;

pub fn execute_fish_action(fish: &mut Fish, action: Action, delta_time: f64) {
    use Action::*;
    match action {
        Move(force) => {
            // I hope this is impulse, I'm not a physicist
            let impulse = force * delta_time * FORCE_MULTIPLIER * fish.mass();
            fish.apply_impulse(impulse);

            // neutral if: energy * distance.powi(2) * move_cost = surface_area * growth_factor
            // with surface = energy.cuberoot().powi(2)
            // -> neutral distance = sqrt(surface_area * growth_factor * 1/move_cost * 1/energy)
            fish.energy -= impulse.length() * IMPULSE_COST; // we removed powi
        }
        SetVelocity(target_velocity, max_energy_ratio) => {
            let mut impulse = (target_velocity - fish.velocity) * fish.mass();

            let cost: N64 = N64::from_inner(impulse.length() * IMPULSE_COST);
            let cost_max: N64 = fish.energy * N64::from(max_energy_ratio);

            if cost > cost_max {
                // bound impulse by allocated energy
                impulse *= (cost_max / cost).into();
            }

            // without this multiplier, drag force creates an unstable feedback loop
            // and crashes the program
            // impulse *= 0.1;

            fish.apply_impulse(impulse);
            fish.energy -= impulse.length() * IMPULSE_COST;
        }
        Pass => (),
    }
}
