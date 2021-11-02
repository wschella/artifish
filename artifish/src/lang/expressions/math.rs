use decorum::N64;

use super::*;
use crate::lang::core::*;
use crate::lang::generators::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fraction(pub f64);

impl Fraction {
    pub fn from_f64(inner: f64) -> Self {
        assert!(!inner.is_nan());
        assert!(inner >= 0.0);
        assert!(inner <= 1.0);
        Fraction(inner)
    }
}

impl From<Fraction> for N64 {
    fn from(f: Fraction) -> Self {
        let Fraction(inner) = f;
        N64::from_inner(inner)
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct LessThenExpr<T> {
    pub left: ExprSlot<T>,
    pub right: ExprSlot<T>,
}

impl<T> Expr<bool> for LessThenExpr<T>
where
    T: Ord + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> bool {
        self.left.eval(state) < self.right.eval(state)
    }
}

impl<T> Mutable<bool> for LessThenExpr<T>
where
    T: Ord + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<bool> {
        branch_using!(rng, {
            generate_bool_expr(rng, 0),
            wrap_in_generic::<bool>(self, rng),
            Box::new(LessThenExpr {
                left: self.left.clone(),
                right: self.right.mutate(rng),
            }),
            Box::new(LessThenExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct AddExpr<T> {
    pub left: ExprSlot<T>,
    pub right: ExprSlot<T>,
}

impl<T> Expr<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return self.left.eval(state) + self.right.eval(state);
    }
}

impl<T> Mutable<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.left.inner.clone(),
            self.right.inner.clone(),
            Box::new(AddExpr {
                left: self.left.clone(),
                right: ExprSlot::new(self.mutate(rng)),
            }),
            Box::new(AddExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}
