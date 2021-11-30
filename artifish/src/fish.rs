// Blub

use decorum::{NotNan, N64};
use rand::Rng;
use rand_chacha::ChaCha20Rng;

use crate::{
    color::Color,
    lang::{Fraction, Program},
    vec2::Vec2,
    BASE_SPLIT_COST, DIE_ON_AMBITIOUS_BABY, MUTATION_RATE, SPLIT_COST_FACTOR,
};

pub type Energy = NotNan<f64>;

#[derive(Clone)]
pub struct Fish {
    pub x: f64,
    pub y: f64,
    pub velocity: Vec2,

    pub energy: Energy,
    pub program: Program,
    pub color: Color,
    pub is_man_made: bool,
    pub tag: Option<String>,
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
            color: Color::GREEN,
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

    pub fn momentum(&self) -> Vec2 {
        self.velocity * self.mass()
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
}

#[derive(Debug, Clone)]
pub struct Control {
    pub force: Vec2,
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Pass,
    Move(Vec2),
    SetVelocity(Vec2, Fraction),
    Split(Vec2, Fraction),
}

pub struct FishControl<'a> {
    pub fishes: &'a mut Vec<Fish>,
    pub controls: &'a mut Vec<Control>,
}

impl<'a> FishControl<'a> {
    pub fn split_fish(
        &mut self,
        rng: &mut ChaCha20Rng,
        fish_index: usize,
        force_per_kg: Vec2,
        mass_fraction: f64,
    ) {
        let fish = &mut self.fishes[fish_index];

        let direction = force_per_kg.normalized();
        let child = Fish {
            x: fish.x + direction.x * fish.radius() * 1.5,
            y: fish.y + direction.y * fish.radius() * 1.5,
            energy: fish.energy * mass_fraction,
            program: if rng.gen_range(0.0..1.0) < MUTATION_RATE {
                fish.program.mutated(rng)
            } else {
                fish.program.clone()
            },
            velocity: Vec2::zero(),
            color: fish.color.mutate(rng),
            is_man_made: fish.is_man_made,
            tag: fish.tag.clone(),
        };
        let cost: N64 = child.energy * SPLIT_COST_FACTOR + BASE_SPLIT_COST;
        if fish.energy > cost {
            fish.energy -= cost;
            let (x, y) = (force_per_kg.x, force_per_kg.y);
            let child_force = Vec2::new(x * child.mass(), y * child.mass());
            self.fishes.push(child);
            self.controls.push(Control { force: child_force });
        } else {
            if DIE_ON_AMBITIOUS_BABY {
                fish.energy -= cost;
            }
        }
    }

    pub fn reproduce(&mut self, rng: &mut ChaCha20Rng, fish_index: usize) {
        let direction = Vec2::random_normalized(rng);
        let force_per_kg = direction * 10.0;
        self.split_fish(rng, fish_index, force_per_kg, 0.2);
    }
}

pub fn execute_fish_action(
    fish_control: &mut FishControl,
    fish_index: usize,
    action: Action,
    delta_time: f64,
    rng: &mut ChaCha20Rng,
) {
    use Action::*;
    let fish = &mut fish_control.fishes[fish_index];
    match action {
        Move(force_per_kg) => {
            let force = force_per_kg * fish.mass();
            let cost = force.length() * delta_time;
            fish.energy -= cost;

            fish_control.controls[fish_index].force += force;

            // old comments pls ignore
            // neutral if: energy * distance.powi(2) * move_cost = surface_area * growth_factor
            // with surface = energy.cuberoot().powi(2)
            // -> neutral distance = sqrt(surface_area * growth_factor * 1/move_cost * 1/energy)
            // fish.generate_impulse(impulse)
        }

        // Apply a force that would set the velocity to the target velocity
        // if there would be no friction or drag.
        SetVelocity(target_velocity, max_energy_ratio) => {
            // momentum = mass * velocity (newton second)
            // momentum = impulse
            let impulse_needed = (target_velocity - fish.velocity) * fish.mass();

            let mut force_needed = impulse_needed / delta_time;
            let mut cost: N64 = N64::from(force_needed.length() * delta_time);
            let cost_max: N64 = fish.energy * N64::from(max_energy_ratio);
            if cost > cost_max {
                // bound impulse by allocated energy
                force_needed *= (cost_max / cost).into();
                cost = cost_max;
            }
            fish.energy -= cost;
            fish_control.controls[fish_index].force += force_needed;
        }
        Split(force_per_kg, mass_fraction) => {
            // const MIN_SPLIT_ENERGY: f64 = 3.0 * 1000.0;
            // if fish_control.fishes[fish_index].energy > MIN_SPLIT_ENERGY {
            fish_control.split_fish(rng, fish_index, force_per_kg, mass_fraction.to_f64());
            // }
        }
        Pass => (),
    }
}
