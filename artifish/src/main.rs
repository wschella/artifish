extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
#[macro_use]
extern crate random_branch;
extern crate rand_distr;
#[macro_use]
extern crate artifish_derive;

use decorum::NotNan;
use glutin_window::GlutinWindow as Window;
use graphics::ellipse::Border;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

mod angels;
mod color;
mod fish;
mod lang;
mod state;
mod vec2;

use color::Color;
use fish::Fish;
use lang::Program;
use state::State;
use vec2::Vec2;

const N_TICKS: u8 = 20;

const MAX_X: f64 = 800.0;
const MAX_Y: f64 = 600.0;

const MOVE_SPEED: f64 = 100.0;
const FISH_SPLIT_AT_SIZE: f64 = 90_000.0 * 1.0;
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
        elapsed_time: 0.0,
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
    elapsed_time: f64,
}

fn generate_fish(rng: &mut ChaCha20Rng) -> Fish {
    let x = rng.gen_range(0.0..MAX_X);
    let y = rng.gen_range(0.0..MAX_Y);
    let radius = rng.gen_range(5.0..1000.0);
    let program = Program::random(rng, 6);
    Fish {
        x,
        y,
        energy: NotNan::from_inner(radius),
        velocity: Vec2::zero(),
        program,
        color: Color::random(rng),
        is_man_made: false,
        tag: None,
    }
}

impl<'a> App<'a> {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        let fishes = &self.state.fishes;

        let glyph_cache = &mut self.glyph_cache;

        self.gl.draw(args.viewport(), |c, gl| {
            // Clear the screen.
            clear(Color::BLACK.into(), gl);

            let identity = c.transform;

            for fish in fishes.iter().rev() {
                let fish_color_dark = fish.color.darken(0.5);
                let cell = ellipse::circle(fish.x, fish.y, fish.radius());
                let cell_border = Border {
                    color: fish_color_dark.into(),
                    radius: 1.0,
                };
                Ellipse::new(fish.color.into()).border(cell_border).draw(
                    cell,
                    &Default::default(),
                    identity,
                    gl,
                );

                let center = ellipse::circle(fish.x, fish.y, 1.5);
                ellipse(fish_color_dark.into(), center, identity, gl);

                if let Some(ref tag) = fish.tag {
                    text(
                        Color::WHITE.into(),
                        20,
                        tag,
                        glyph_cache,
                        identity.trans(fish.x, fish.y),
                        gl,
                    )
                    .unwrap();
                }

                // let t = identity.trans(100.0, 100.0);
                // text(Color::RED.into(), 100, "tetten", glyph_cache, t, gl).unwrap();

                let t = identity.trans(MAX_X - 100.0, MAX_Y - 100.0);
                text(
                    Color::RED.into(),
                    30,
                    &fishes.len().to_string(),
                    glyph_cache,
                    t,
                    gl,
                )
                .unwrap();
            }
        });
    }

    pub fn update(&mut self, args: &UpdateArgs) {
        let time_step = 1.0 / (N_TICKS as f64);
        self.elapsed_time += args.dt;

        if self.elapsed_time > time_step {
            // easy mode
            self.elapsed_time = 0.0;
            self.state.update(time_step);
        }
    }
}
