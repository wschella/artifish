use rand::Rng;
use rand_chacha::ChaCha20Rng;

#[derive(Clone, Copy, Debug)]
pub struct Color {
    pub inner: [f32; 4],
}

impl Color {
    pub const fn new(color: [f32; 4]) -> Color {
        Color { inner: color }
    }

    pub fn random(rng: &mut ChaCha20Rng) -> Color {
        let color: [f32; 4] = [
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            rng.gen_range(0.0..=1.0),
            1.0,
        ];

        Color { inner: color }
    }

    pub fn mutate(&self, rng: &mut ChaCha20Rng) -> Color {
        let mut color = self.inner;

        for i in 0..3 {
            let delta = 0.1;
            color[i] += rng.gen_range(-delta..delta);
            color[i] = color[i].clamp(0.1, 1.0);
        }

        Color { inner: color }
    }

    pub fn darken(&self, amount: f32) -> Color {
        Color {
            inner: [
                self.inner[0] * amount,
                self.inner[1] * amount,
                self.inner[2] * amount,
                1.0 - amount * (1.0 - self.inner[3]),
            ],
        }
    }

    pub const GREEN: Color = Color::new([0.0, 1.0, 0.0, 1.0]);
    pub const RED: Color = Color::new([1.0, 0.0, 0.0, 1.0]);
    pub const BLUE: Color = Color::new([0.0, 0.0, 1.0, 1.0]);
    pub const BLACK: Color = Color::new([0.0, 0.0, 0.0, 1.0]);
    pub const WHITE: Color = Color::new([1.0, 1.0, 1.0, 1.0]);
}

impl From<[f32; 4]> for Color {
    fn from(color: [f32; 4]) -> Color {
        Color { inner: color }
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> [f32; 4] {
        color.inner
    }
}
