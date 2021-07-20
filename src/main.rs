extern crate glutin_window;
extern crate graphics;
extern crate opengl_graphics;
extern crate piston;
extern crate rand;

use glutin_window::GlutinWindow as Window;
use graphics::ellipse::Border;
use opengl_graphics::{GlGraphics, OpenGL};
use piston::event_loop::{EventSettings, Events};
use piston::input::{RenderArgs, RenderEvent, UpdateArgs, UpdateEvent};
use piston::window::WindowSettings;
use rand::Rng;

const MAX_X: f64 = 800.0;
const MAX_Y: f64 = 600.0;

pub struct App {
    gl: GlGraphics, // OpenGL drawing backend.
    state: State,
}

pub struct State {
    fishes: Vec<Fish>,
}

pub struct Fish {
    x: f64,
    y: f64,
    energy: f64,
}

impl Fish {
    pub fn new(x: f64, y: f64, energy: f64) -> Self {
        Fish { x, y, energy }
    }

    pub fn squared_radius(&self) -> f64 {
        self.energy / std::f64::consts::PI
    }

    pub fn radius(&self) -> f64 {
        self.squared_radius().sqrt()
    }

    pub fn covers(&self, other: &Fish) -> bool {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let squared_dist = dx.powi(2) + dy.powi(2);
        self.squared_radius() > squared_dist
    }

    pub fn eat(&mut self, other: &Fish) {
        self.energy += other.energy;
    }

    pub fn move_to(&mut self, x: f64, y: f64) {
        self.x = x;
        self.y = y;
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

        let child_energy = (self.energy - 1500.0) / 2.0;
        self.move_to(x_1, y_1);
        self.energy = child_energy;
        Fish::new(x_2, y_2, child_energy)
    }
}

fn step(state: &mut State) {
    let fishes = &mut state.fishes;
    fishes.sort_by(|a, b| (a.energy).partial_cmp(&b.energy).unwrap().reverse());

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

impl App {
    fn render(&mut self, args: &RenderArgs) {
        use graphics::*;

        const GREEN: [f32; 4] = [0.0, 1.0, 0.0, 1.0];
        const RED: [f32; 4] = [1.0, 0.0, 0.0, 1.0];
        const BLACK: [f32; 4] = [0.0, 0.0, 0.0, 1.0];

        let fishes = &self.state.fishes;
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
            }
        });
    }

    fn update(&mut self, args: &UpdateArgs) {
        step(&mut self.state);

        for fish in self.state.fishes.iter_mut() {
            fish.energy += 300.0 * args.dt;
        }

        for i in 0..self.state.fishes.len() {
            if self.state.fishes[i].energy > 9000.0 {
                let new = self.state.fishes[i].split();
                self.state.fishes.push(new);
            }
        }

        // prevent aquarium leaks
        let mut i = 0;
        while i < self.state.fishes.len() {
            let fish = &self.state.fishes[i];

            if fish.x > MAX_X || fish.y > MAX_Y {
                self.state.fishes.remove(i);
            } else {
                i += 1;
            }
        }
    }
}

fn main() {
    // Change this to OpenGL::V2_1 if not working.
    let opengl = OpenGL::V3_2;

    // Create an Glutin window.
    let mut window: Window = WindowSettings::new("static-circle", [600, 600])
        .graphics_api(opengl)
        .exit_on_esc(true)
        .build()
        .unwrap();
    let mut fishes = Vec::new();
    let mut rng = rand::thread_rng();

    for _ in 0..100 {
        let x = rng.gen_range(0.0..600.0);
        let y = rng.gen_range(0.0..600.0);
        let radius = rng.gen_range(5.0..1000.0);

        let fish = Fish::new(x, y, radius);
        fishes.push(fish);
    }

    // Create a new game and run it.
    let mut app = App {
        gl: GlGraphics::new(opengl),
        state: State { fishes },
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
