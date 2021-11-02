use super::*;
use crate::fish::*;
use crate::lang::core::*;
use crate::lang::generators::*;
use crate::vec2::*;

#[derive(Clone, ArtifishExpr)]
pub struct MoveExpr {
    pub direction: ExprSlot<Vec2>,
}

impl Expr<Action> for MoveExpr {
    fn eval(&self, state: &InterpreterState) -> Action {
        let dir_vec = self.direction.eval(state);
        Action::Move(dir_vec)
    }
}

impl Mutable<Action> for MoveExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Action> {
        branch_using!(rng, {
            wrap_in_generic::<Action>(self, rng),
            Box::new(MoveExpr {
                direction: self.direction.mutate(rng),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct SetVelocityExpr {
    pub target_velocity: ExprSlot<Vec2>,
    pub max_energy_ratio: ExprSlot<Fraction>,
}

impl Expr<Action> for SetVelocityExpr {
    fn eval(&self, state: &InterpreterState) -> Action {
        let velocity_vec = self.target_velocity.eval(state);
        let max_energy_ratio = self.max_energy_ratio.eval(state);
        Action::SetVelocity(velocity_vec, max_energy_ratio)
    }
}

impl Mutable<Action> for SetVelocityExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Action> {
        branch_using!(rng, {
            wrap_in_generic::<Action>(self, rng),
            Box::new(SetVelocityExpr {
                target_velocity: self.target_velocity.mutate(rng),
                max_energy_ratio: self.max_energy_ratio.clone(),
            }),
            Box::new(SetVelocityExpr {
                target_velocity: self.target_velocity.clone(),
                max_energy_ratio: self.max_energy_ratio.mutate(rng),
            })
        })
    }
}
