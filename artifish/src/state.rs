use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Poisson};

use crate::angels::generate_angel;
use crate::fish::behave_fishes;
use crate::{fish::Fish, FISH_GROWTH_FACTOR, FISH_SPLIT_AT_SIZE, MAX_X, MAX_Y};
use crate::{generate_fish, FISH_GENERATION_RATE, MOVE_SPEED};

#[derive(Clone)]
pub struct State {
    pub fishes: Vec<Fish>,
    pub rng: ChaCha20Rng,
}

impl State {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut fishes: Vec<Fish> = (0..100).map(|_| generate_fish(&mut rng)).collect();

        let angels: Vec<Fish> = (0..40).map(|_| generate_angel(&mut rng)).collect();
        fishes.extend(angels);

        Self { fishes, rng }
    }

    pub fn add_fish(&mut self, fish: Fish) {
        self.fishes.push(fish);
    }

    pub fn update(&mut self, delta_time: f64) {
        for fish in self.fishes.iter_mut() {
            fish.energy += FISH_GROWTH_FACTOR * fish.surface_area() * delta_time;
        }

        for fish in self.fishes.iter_mut() {
            // TODO: Drag @wout

            let displacement = fish.velocity * delta_time * MOVE_SPEED;
            fish.move_by(&displacement);
            fish.move_to(fish.x.clamp(0.0, MAX_X), fish.y.clamp(0.0, MAX_Y));
        }

        for fish in self.fishes.iter_mut() {
            // TODO: Fix accelaration to velocity
        }

        for fish in self.fishes.iter_mut() {
            const FRICTION_COEF: f64 = 25.0; // 1/2 * mass density of fluid * drag coefficient
            let drag_force: f64 =
                FRICTION_COEF * fish.velocity.length().powi(2) * fish.surface_area();
            let drag_force_vec = -drag_force * fish.velocity.normalized();

            // TODO: I am not sure whether this is how physics work
            let drag_impulse_vec = delta_time * drag_force_vec;
            // TODO fix NAns
            // check for unstable feedback loop
            // TODO: can this be solved analytically?
            debug_assert!(
                drag_impulse_vec.length() <= fish.velocity.length() * fish.energy.into_inner()
            );
            fish.apply_impulse(drag_impulse_vec);
        }

        for i in 0..self.fishes.len() {
            if self.fishes[i].energy > FISH_SPLIT_AT_SIZE {
                let new = self.fishes[i].reproduce(&mut self.rng);
                self.fishes.push(new);
            }
        }

        // TODO: Make static some time
        let distr = Poisson::new(FISH_GENERATION_RATE * delta_time).unwrap();
        let n_fishes: u32 = distr.sample(&mut self.rng).floor() as u32;
        for _ in 0..n_fishes {
            self.fishes.push(generate_fish(&mut self.rng))
        }

        // prevent aquarium leaks
        let mut i = 0;
        while i < self.fishes.len() {
            let fish = &self.fishes[i];

            if fish.x > MAX_X || fish.y > MAX_Y || fish.x < 0.0 || fish.y < 0.0 {
                self.fishes.remove(i);
            } else {
                i += 1;
            }
        }

        behave_fishes(self, delta_time);
        // WHEN ANGELS DESERVE TO DIEEEEEEEEEEEEEEE
        self.fishes.retain(|f| f.energy > 0.0);

        let fishes = &mut self.fishes;
        fishes.sort_by_key(|f| -f.energy);

        // Fishes eat other fishes
        let mut i = 0;
        while i < fishes.len() {
            let mut j = i + 1;
            while j < fishes.len() {
                if fishes[i].covers(&fishes[j]) {
                    let eaten = fishes.remove(j);
                    fishes[i].eat(&eaten);
                } else {
                    j += 1;
                }
            }
            i += 1;
        }
    }
}
