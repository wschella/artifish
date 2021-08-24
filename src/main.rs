extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
#[macro_use]
extern crate random_branch;

use decorum::NotNan;
use glutin_window::GlutinWindow as Window;
use graphics::ellipse::Border;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;

mod languages;
mod vec2;

use languages::lang::*;

use vec2::Vec2;

const MAX_X: f64 = 800.0;
const MAX_Y: f64 = 600.0;

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("static-but-internal-circle", [MAX_X, MAX_Y])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();

    let texture_settings = TextureSettings::new().filter(Filter::Nearest);
    let glyphs = GlyphCache::new("assets/ZenLoop-Italic.ttf", (), texture_settings)
        .expect("Could not load font");

    let seed: u64 = 127002;

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        state: State::new(seed),
        glyph_cache: glyphs,
    };

    let mut events = Events::new(EventSettings::new());
    while let Some(e) = events.next(&mut window) {
        if let Some(args) = e.render_args() {
            app.render(&args);
        }

        if let Some(args) = e.update_args() {
            app.update(&args);
        }
    }
}

pub struct App<'a> {
    gl: GlGraphics, // OpenGL drawing backend.
    state: State,
    glyph_cache: GlyphCache<'a>,
}

#[derive(Clone)]
pub struct State {
    fishes: Vec<Fish>,
}

impl State {
    fn new(seed: u64) -> Self {
        let mut fishes = Vec::new();
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        for _ in 0..100 {
            let x = rng.gen_range(0.0..MAX_X);
            let y = rng.gen_range(0.0..MAX_Y);
            let radius = rng.gen_range(5.0..1000.0);

            // let program = run_towards_program();
            let program = Program::random(&mut rng);
            let fish = Fish::new(x, y, NotNan::from_inner(radius), program);
            fishes.push(fish);
        }

        Self { fishes }
    }

    fn update(&mut self, delta_time: f64) {
        for fish in self.fishes.iter_mut() {
            fish.energy += 1.0 * fish.surface_area() * delta_time;
        }

        for i in 0..self.fishes.len() {
            if self.fishes[i].energy > 90000.0 {
                let new = self.fishes[i].split();
                self.fishes.push(new);
            }
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

        // Hard coded run-toward smaller and run away from bigger
        behave_fishes(self, delta_time);

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

pub type Energy = NotNan<f64>;

#[derive(Clone)]
pub struct Fish {
    x: f64,
    y: f64,
    energy: Energy,
    program: Program,
}

fn behave_fishes(state: &mut State, delta_time: f64) {
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
        }
    }

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

    pub fn split(&mut self) -> Fish {
        let mut rng = rand::thread_rng();
        let ax = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;
        let opposite = ax - std::f64::consts::PI;

        let radius = self.radius();

        let x_1 = self.x + radius * ax.cos();
        let y_1 = self.y + radius * ax.sin();
        let x_2 = self.x + radius * opposite.cos();
        let y_2 = self.y + radius * opposite.sin();

        let child_energy = self.energy / 2.5;
        self.move_to(x_1, y_1);
        self.energy = child_energy;
        Fish::new(x_2, y_2, child_energy, self.program.clone())
    }
}

impl<'a> App<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let fishes = &self.state.fishes;

        let glyph_cache = &mut self.glyph_cache;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let identity = c.transform;

            for fish in fishes.iter().rev() {
                let cell = ellipse::circle(fish.x, fish.y, fish.radius());
                let cell_border = Border {
                    color: RED,
                    radius: 0.1,
                };
                Ellipse::new(GREEN).border(cell_border).draw(
                    cell,
                    &Default::default(),
                    identity,
                    gl,
                );

                let center = ellipse::circle(fish.x, fish.y, 2.0);
                ellipse(RED, center, identity, gl);

                let t = identity.trans(100.0, 100.0);
                text(RED, 100, "tetten", glyph_cache, t, gl).unwrap();
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        self.state.update(args.dt);
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Action {
    Pass,
    Move(Vec2),
}

fn execute_fish_action(fish: &mut Fish, action: Action, delta_time: f64) {
    let move_speed: f64 = 10.0;
    use Action::*;
    match action {
        Move(direction) => {
            fish.move_by(&(direction * delta_time * move_speed));
            fish.move_to(fish.x.clamp(0.0, MAX_X), fish.y.clamp(0.0, MAX_Y))
        }
        Pass => (),
    }
}
