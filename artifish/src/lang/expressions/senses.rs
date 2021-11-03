use decorum::NotNan;

use crate::color::Color;
use crate::lang::core::*;
use crate::lang::generators::*;
use crate::lang::Fraction;
use crate::vec2::*;

// region: color

#[derive(Clone, ArtifishExpr)]
pub struct FishColorExpr {
    pub fish: ExprSlot<FishRef>,
}

impl Expr<Color> for FishColorExpr {
    fn eval(&self, s: &InterpreterState) -> Color {
        let fish_ref = self.fish.eval(s);
        if let Some(fish_ix) = fish_ref.maybe_fish_num {
            s.fishes[fish_ix].color
        } else {
            Color::BLACK
        }
    }
}

impl Mutable<Color> for FishColorExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Color> {
        branch_using!(rng, {
            generate_color_expr(rng, COLOR_MIN),
            wrap_in_generic(self, rng),
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct ColorSimilarityExpr {
    pub lhs: ExprSlot<Color>,
    pub rhs: ExprSlot<Color>,
}

impl Expr<Fraction> for ColorSimilarityExpr {
    fn eval(&self, s: &InterpreterState) -> Fraction {
        let lhs_color = self.lhs.eval(s).inner;
        let rhs_color = self.rhs.eval(s).inner;

        // todo: maybe move to helper on color
        let mut dot_product = 0.0;
        let mut acc_lhs = 0.0;
        let mut acc_rhs = 0.0;

        for i in 0..4 {
            acc_lhs += lhs_color[i].powi(2);
            acc_rhs += rhs_color[i].powi(2);
            dot_product += lhs_color[i] * rhs_color[i];
        }

        let lhs_len = acc_lhs.sqrt();
        let rhs_len = acc_rhs.sqrt();

        let cos = dot_product / (lhs_len * rhs_len);
        // catch rounding errors
        let cos = cos.clamp(0.0, 1.0);

        return Fraction::from_f64(cos as f64);
    }
}

impl Mutable<Fraction> for ColorSimilarityExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Fraction> {
        branch_using!(rng, {
            generate_fraction_expr(rng, FRACTION_MIN),
            wrap_in_generic(self, rng),
        })
    }
}

// endregion: color

// region: getself
#[derive(Clone, ArtifishExpr)]
pub struct GetSelfExpr;

impl Expr<FishRef> for GetSelfExpr {
    fn eval(&self, state: &InterpreterState) -> FishRef {
        FishRef {
            maybe_fish_num: Some(state.fish_num),
        }
    }
}

impl Mutable<FishRef> for GetSelfExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<FishRef> {
        branch_using!(rng, {
            generate_fish_ref_expr(rng, FISH_REF_MIN),
            wrap_in_generic(self, rng),
        })
    }
}

// endregion: getself

// region: dichtstevis

#[derive(Clone, ArtifishExpr)]
pub struct DichtsteVisExpr;

#[derive(Clone, Debug)]
pub struct FishRef {
    pub maybe_fish_num: Option<usize>,
}

impl Expr<FishRef> for DichtsteVisExpr {
    fn eval(&self, state: &InterpreterState) -> FishRef {
        let maybe_j = state
            .fishes
            .iter()
            .enumerate()
            .filter(|(j, _)| j != &state.fish_num)
            .min_by_key(|(_, fish)| NotNan::from_inner(state.get_self().distance(fish)))
            .map(|(j, _)| j);
        FishRef {
            maybe_fish_num: maybe_j,
        }
    }
}

impl Mutable<FishRef> for DichtsteVisExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<FishRef> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_fish_ref_expr(rng, FISH_REF_MIN),
        })
    }
}

// endregion: dichtstevis

// region: fishenergy

#[derive(Clone, ArtifishExpr)]
pub struct FishEnergyExpr {
    pub fish: ExprSlot<FishRef>,
}

impl Expr<NotNan<f64>> for FishEnergyExpr {
    fn eval(&self, state: &InterpreterState) -> NotNan<f64> {
        let FishRef { maybe_fish_num } = self.fish.eval(state);

        if let Some(fish_num) = maybe_fish_num {
            NotNan::from(state.fishes[fish_num].energy)
        } else {
            NotNan::from(0.0)
        }
    }
}

impl Mutable<NotNan<f64>> for FishEnergyExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_f64_expr(rng, F64_MIN),
        })
    }
}

// endregion: fishenergy

#[derive(Clone, ArtifishExpr)]
pub struct FishDirectionExpr {
    pub origin: ExprSlot<FishRef>,
    pub target: ExprSlot<FishRef>,
}

impl Expr<Vec2> for FishDirectionExpr {
    fn eval(&self, state: &InterpreterState) -> Vec2 {
        let FishRef {
            maybe_fish_num: origin,
        } = self.origin.eval(state);
        let FishRef {
            maybe_fish_num: target,
        } = self.target.eval(state);

        match (origin, target) {
            (Some(o), Some(t)) => state.fishes[o].direction_to(&state.fishes[t]),
            _ => Vec2::zero(),
        }
    }
}

impl Mutable<Vec2> for FishDirectionExpr {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Vec2> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_direction_expr(rng, DIRECTION_MIN),
        })
    }
}
