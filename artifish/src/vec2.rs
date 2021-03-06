use rand::Rng;
use rand_chacha::ChaCha20Rng;

use crate::lang::Fraction;

#[derive(Clone, Copy, Debug)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

impl Vec2 {
    pub fn new(x: f64, y: f64) -> Self {
        Vec2 { x, y }
    }

    pub fn zero() -> Self {
        Vec2 { x: 0.0, y: 0.0 }
    }

    pub fn random_normalized(rng: &mut ChaCha20Rng) -> Self {
        let angle = rng.gen::<f64>() * 2.0 * std::f64::consts::PI;

        Vec2 {
            x: angle.cos(),
            y: angle.sin(),
        }
    }

    pub fn powi(self, n: i32) -> Self {
        self.normalized() * self.length().powi(n)
    }

    pub fn length(&self) -> f64 {
        (self.x.powi(2) + self.y.powi(2)).sqrt()
    }

    pub fn normalized(&self) -> Self {
        let length = self.length();
        if length == 0.0 {
            return Vec2::zero();
        } else {
            Vec2 {
                x: self.x / length,
                y: self.y / length,
            }
        }
    }

    pub fn invert(&self) -> Vec2 {
        Vec2 {
            x: -self.x,
            y: -self.y,
        }
    }
}

impl std::ops::Div<f64> for Vec2 {
    type Output = Vec2;

    fn div(self, rhs: f64) -> Self::Output {
        Vec2 {
            x: self.x / rhs,
            y: self.y / rhs,
        }
    }
}

impl std::ops::DivAssign<f64> for Vec2 {
    fn div_assign(&mut self, rhs: f64) {
        *self = *self / rhs;
    }
}

impl std::ops::MulAssign<f64> for Vec2 {
    fn mul_assign(&mut self, rhs: f64) {
        *self = *self * rhs;
    }
}

impl std::ops::Mul<f64> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: f64) -> Self::Output {
        Vec2 {
            x: self.x * rhs,
            y: self.y * rhs,
        }
    }
}

impl std::ops::Mul<Vec2> for f64 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self
    }
}

impl std::ops::Mul<Vec2> for Fraction {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self::Output {
        rhs * self.to_f64()
    }
}

impl std::ops::Mul<Fraction> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Fraction) -> Self::Output {
        self * rhs.to_f64()
    }
}

impl std::ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl std::ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        *self = *self + rhs
    }
}

impl std::ops::Sub<Vec2> for Vec2 {
    type Output = Vec2;

    fn sub(self, rhs: Vec2) -> Self::Output {
        Vec2 {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl std::ops::SubAssign<Vec2> for Vec2 {
    fn sub_assign(&mut self, rhs: Vec2) {
        *self = *self - rhs
    }
}
