use decorum::NotNan;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Poisson};

use crate::vec2::Vec2;
use crate::{FISH_GENERATION_RATE, MOVE_SPEED, angels, generate_fish};
use crate::{FISH_GROWTH_FACTOR, FISH_SPLIT_AT_SIZE, MAX_X, MAX_Y, RED, fish::Fish};
use crate::fish::behave_fishes;

#[derive(Clone)]
pub struct State {
    pub fishes: Vec<Fish>,
    pub rng: ChaCha20Rng,
}

impl State {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut fishes: Vec<Fish> = (0..100).map(|_| generate_fish(&mut rng)).collect();

        for _ in 0..10 {
            let x = rng.gen_range(0.0..MAX_X);
            let y = rng.gen_range(0.0..MAX_Y);

            let smartie = Fish {
                x,
                y,
                energy: NotNan::from_inner(500.0),
                velocity: Vec2::new(0.0, 0.0),
                program: angels::smartie(),
                color: RED,
                is_man_made: true,
                tag: Some("s".to_owned()),
            };
            fishes.push(smartie);
        }

        Self { fishes, rng }
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