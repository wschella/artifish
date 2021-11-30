use decorum::N64;

use super::*;
use crate::lang::core::*;
use crate::lang::generators::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fraction(pub N64);

impl Fraction {
    pub fn from_f64(inner_float: f64) -> Self {
        let inner = N64::from_inner(inner_float);
        assert!(inner >= 0.0);
        assert!(inner <= 1.0);
        Fraction(inner)
    }

    pub fn to_f64(self) -> f64 {
        self.0.into_inner()
    }
}

impl From<Fraction> for N64 {
    fn from(f: Fraction) -> Self {
        let Fraction(inner) = f;
        return inner;
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
                right: self.right.mutate(rng),
            }),
            Box::new(AddExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct MulExpr<T1, T2> {
    pub left: ExprSlot<T1>,
    pub right: ExprSlot<T2>,
}

impl<T1, T2> Expr<<T1 as std::ops::Mul<T2>>::Output> for MulExpr<T1, T2>
where
    T1: std::ops::Mul<T2> + Clone + 'static,
    T2: Clone + 'static,
    <T1 as std::ops::Mul<T2>>::Output: Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> <T1 as std::ops::Mul<T2>>::Output {
        return self.left.eval(state) * self.right.eval(state);
    }
}

impl<T1, T2> Mutable<<T1 as std::ops::Mul<T2>>::Output> for MulExpr<T1, T2>
where
    T1: std::ops::Mul<T2> + Clone + 'static,
    T2: Clone + 'static,
    <T1 as std::ops::Mul<T2>>::Output: Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<<T1 as std::ops::Mul<T2>>::Output> {
        branch_using!(rng, {
            wrap_in_generic::<<T1 as std::ops::Mul<T2>>::Output>(self, rng),
            // self.left.inner.clone(),
            // self.right.inner.clone(),
            Box::new(MulExpr {
                left: self.left.clone(),
                right: self.right.mutate(rng),
            }),
            Box::new(MulExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct NegateExpr<T> {
    pub value: ExprSlot<T>,
}

impl<T> Expr<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return -self.value.eval(state);
    }
}

impl<T> Mutable<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.inner.clone(),
            Box::new(NegateExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}
