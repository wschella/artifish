// Blub

use decorum::NotNan;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

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
        }
    }
}
