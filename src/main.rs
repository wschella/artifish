extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
#[macro_use]
extern crate random_branch;
extern crate rand_distr;

use decorum::NotNan;
use glutin_window::GlutinWindow as Window;
use graphics::ellipse::Border;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha20Rng;
use rand_distr::{Distribution, Poisson};

mod fish;
mod languages;
mod vec2;

use fish::Fish;
use languages::lang;
use languages::lang::*;

use vec2::Vec2;

#[allow(dead_code)]
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
#[allow(dead_code)]
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const MAX_X: f64 = 800.0;
const MAX_Y: f64 = 600.0;

const MOVE_SPEED: f64 = 25.0;
const FISH_SPLIT_AT_SIZE: f64 = 90_000.0 * 4.0;
const FISH_GROWTH_FACTOR: f64 = 1.0;
const FISH_GENERATION_RATE: f64 = 2.0 / 1.0;

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
    rng: ChaCha20Rng,
}

impl State {
    fn new(seed: u64) -> Self {
        let mut rng = ChaCha20Rng::seed_from_u64(seed);
        let mut fishes: Vec<Fish> = (0..100).map(|_| generate_fish(&mut rng)).collect();

        for _ in 0..10 {
            let x = rng.gen_range(0.0..MAX_X);
            let y = rng.gen_range(0.0..MAX_Y);

            let smartie = Fish {
                x,
                y,
                energy: NotNan::from_inner(500.0),
                program: lang::smartie(),
                color: RED,
                is_man_made: true,
                tag: Some("s".to_owned()),
            };
            fishes.push(smartie);
        }

        Self { fishes, rng }
    }

    fn update(&mut self, delta_time: f64) {
        for fish in self.fishes.iter_mut() {
            fish.energy += FISH_GROWTH_FACTOR * fish.surface_area() * delta_time;
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

fn generate_fish(rng: &mut ChaCha20Rng) -> Fish {
    let x = rng.gen_range(0.0..MAX_X);
    let y = rng.gen_range(0.0..MAX_Y);
    let radius = rng.gen_range(5.0..1000.0);
    let color: [f32; 4] = [
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        rng.gen_range(0.0..=1.0),
        1.0,
    ];
    let program = Program::random(rng, 6);
    Fish {
        x,
        y,
        energy: NotNan::from_inner(radius),
        program,
        color,
        is_man_made: false,
        tag: None,
    }
}

fn behave_fishes(state: &mut State, delta_time: f64) -> () {
    let fishes = &mut state.fishes;
    for i in 0..fishes.len() {
        let action = run_fish(fishes, i);
        execute_fish_action(&mut fishes[i], action, delta_time);
    }
}

fn darken(color: &[f32; 4]) -> [f32; 4] {
    let f = 0.5;

    [
        color[0] * f,
        color[1] * f,
        color[2] * f,
        1.0 - f * (1.0 - color[3]),
    ]
}

impl<'a> App<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let fishes = &self.state.fishes;

        let glyph_cache = &mut self.glyph_cache;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(BLACK, gl);

            let identity = c.transform;

            for fish in fishes.iter().rev() {
                let fish_color_dark = darken(&fish.color);
                let cell = ellipse::circle(fish.x, fish.y, fish.radius());
                let cell_border = Border {
                    color: fish_color_dark,
                    radius: 1.0,
                };
                Ellipse::new(fish.color).border(cell_border).draw(
                    cell,
                    &Default::default(),
                    identity,
                    gl,
                );

                let center = ellipse::circle(fish.x, fish.y, 1.5);
                ellipse(fish_color_dark, center, identity, gl);

                if let Some(ref tag) = fish.tag {
                    text(
                        WHITE,
                        20,
                        tag,
                        glyph_cache,
                        identity.trans(fish.x, fish.y),
                        gl,
                    )
                    .unwrap();
                }

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
    use Action::*;
    match action {
        Move(direction) => {
            fish.move_by(&(direction * delta_time * MOVE_SPEED));
            fish.move_to(fish.x.clamp(0.0, MAX_X), fish.y.clamp(0.0, MAX_Y))
        }
        Pass => (),
    }
}
