use crate::lang::core::*;
use crate::lang::generators::*;

#[derive(Clone, ArtifishExpr)]
pub struct NotExpr<T> {
    pub value: ExprSlot<T>,
}

impl<T> Expr<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return !self.value.eval(state);
    }
}

impl<T> Mutable<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.inner.clone(),
            Box::new(NotExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}
