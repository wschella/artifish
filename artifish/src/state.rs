use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Poisson};

use crate::angels::generate_angel;
use crate::fish::{Control, FishControl, execute_fish_action};
use crate::lang::InterpreterState;
use crate::vec2::Vec2;
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

    pub fn update(&mut self, delta_time: f64) {
        let mut controls = vec![Control { force: Vec2::zero() }; self.fishes.len()];
        let mut fish_control = FishControl {
            controls: &mut controls,
            fishes: &mut self.fishes,
        };
    
        for fish in fish_control.fishes.iter_mut() {
            fish.energy += FISH_GROWTH_FACTOR * fish.surface_area() * delta_time;
        }

        // behave fishes
        for i in 0..fish_control.fishes.len() {
            let action = {    
                let interpreter_state = InterpreterState{
                    fishes: fish_control.fishes,
                    fish_num: i,
                };
    
                fish_control.fishes[i].program.run(&interpreter_state)
            };
            execute_fish_action(&mut fish_control, i, action, delta_time, &mut self.rng);
        }
        
        // Reproduce
        for i in 0..fish_control.fishes.len() {
            if fish_control.fishes[i].energy > FISH_SPLIT_AT_SIZE {
                fish_control.reproduce(&mut self.rng, i);
            }
        }

        // Move fishes
        for fish in self.fishes.iter_mut() {
            let displacement = fish.velocity * delta_time * MOVE_SPEED;
            fish.move_by(&displacement);
            fish.move_to(fish.x.clamp(0.0, MAX_X), fish.y.clamp(0.0, MAX_Y));
        }

        // TOTO
        for (fish, control) in self.fishes.iter_mut().zip(controls.iter()) {
            // Drag equation
            // F = 1/2 * rho * v^2 * A * Cd
            // rho = mass density of the fluid
            // v = flow velocity relative to the object
            // A = reference area of the object, typically the cross-sectional area of the object
            // Cd = drag coefficient (skin friction and form drag)
            // Wout Schellaert et al. (2010)
            // OUR_DRAG_COEF = 1/2 * rho * Cd

            const OUR_DRAG_COEF: f64 = 0.0; // 1/2 * mass density of fluid * drag coefficient
            let drag_force: Vec2 = OUR_DRAG_COEF 
                * fish.velocity.invert().powi(2)
                * fish.surface_area();

            debug_assert!(drag_force.length() * delta_time <= fish.momentum().length());

            // Apply controls
            let resultant_force = control.force + drag_force;
            let impulse = resultant_force * delta_time;
            let delta_velocity = impulse / fish.mass();
            fish.velocity += delta_velocity;

        }

        // Generate new fishes
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
