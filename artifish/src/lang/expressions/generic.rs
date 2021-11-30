use decorum::NotNan;

use super::*;
use crate::color::Color;
use crate::fish::*;
use crate::lang::core::*;
use crate::lang::generators::*;

#[derive(Clone, ArtifishExpr)]
pub struct IfExpr<T> {
    pub condition: ExprSlot<bool>,
    pub consequent: ExprSlot<T>,
    pub alternative: ExprSlot<T>,
}

impl<T> Expr<T> for IfExpr<T>
where
    T: Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        if self.condition.eval(state) {
            self.consequent.eval(state)
        } else {
            self.alternative.eval(state)
        }
    }
}

impl<T> Mutable<T> for IfExpr<T>
where
    T: Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.consequent.inner.clone(),
            self.alternative.inner.clone(),
            Box::new(IfExpr {
                condition: self.condition.mutate(rng),
                consequent: self.consequent.clone(),
                alternative: self.alternative.clone(),
            }),
            Box::new(IfExpr {
                condition: self.condition.clone(),
                consequent: self.consequent.mutate(rng),
                alternative: self.alternative.clone(),
            }),
            Box::new(IfExpr {
                condition: self.condition.clone(),
                consequent: self.consequent.clone(),
                alternative: self.alternative.mutate(rng),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct ConstExpr<T> {
    #[expr_tree_node(not_a_child)]
    pub value: T,
}

impl<T> ConstExpr<T> {
    pub fn new(value: T) -> Self {
        Self { value: value }
    }
}

impl<T> Expr<T> for ConstExpr<T>
where
    Self: Mutable<T>,
    T: Clone + 'static,
{
    fn eval(&self, _: &InterpreterState) -> T {
        return self.value.clone();
    }
}

impl Mutable<NotNan<f64>> for ConstExpr<NotNan<f64>> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            Box::new(NegateExpr {
                value: ExprSlot { inner: Box::new(self.clone())},
            }),
            generate_f64_expr(rng, F64_MIN),
        })
    }
}

impl Mutable<Action> for ConstExpr<Action> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Action> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_action_expr(rng, ACTION_MIN),
        })
    }
}

impl Mutable<bool> for ConstExpr<bool> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<bool> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_bool_expr(rng, 0),
        })
    }
}

impl Mutable<Fraction> for ConstExpr<Fraction> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Fraction> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_fraction_expr(rng, FRACTION_MIN),
        })
    }
}

impl Mutable<Color> for ConstExpr<Color> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Color> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_color_expr(rng, COLOR_MIN),
        })
    }
}
