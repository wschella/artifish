extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;
#[macro_use]
extern crate random_branch;
extern crate rand_distr;

use decorum::{NotNan};
use glutin_window::GlutinWindow as Window;
use graphics::ellipse::Border;
use opengl_graphics::{Filter, GlGraphics, GlyphCache, OpenGL, TextureSettings};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::Rng;
use rand_chacha::ChaCha20Rng;

mod angels;
mod fish;
mod languages;
mod vec2;
mod state;

use fish::{Fish};
use languages::lang::*;

use vec2::Vec2;
use state::State;

#[allow(dead_code)]
const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];
#[allow(dead_code)]
const WHITE: [f32; 4] = [1.0, 1.0, 1.0, 1.0];

const MAX_X: f64 = 800.0;
const MAX_Y: f64 = 600.0;

const MOVE_SPEED: f64 = 100.0;
const FISH_SPLIT_AT_SIZE: f64 = 90_000.0 * 4.0;
const FISH_GROWTH_FACTOR: f64 = 1.0;
const FISH_GENERATION_RATE: f64 = 2.0 / 1.0;

// TODO: FIXME
const IMPULSE_COST: f64 = 0.1; // / MOVE_SPEED / MOVE_SPEED;

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
        velocity: Vec2::new(0.0, 0.0),
        program,
        color,
        is_man_made: false,
        tag: None,
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

    pub fn update(&mut self, args: &UpdateArgs) {
        self.state.update(args.dt);
    }
}