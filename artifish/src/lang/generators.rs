use decorum::NotNan;
use rand::Rng;

use super::core::*;
use super::expressions::*;

use crate::fish::Action;
use crate::vec2::*;

#[macro_export]
macro_rules! generate_tree {
    ( $depth:expr, $rng: ident,
        0 => { $( $leaf:expr ),* $(,)? },
        1 => { $( $depth1:expr ),* $(,)? },
        2 => { $( $depth2:expr ),* $(,)? }$(,)?
    ) => {
        match $depth {
            0 => {
                branch_using!($rng, {
                    $( $leaf, )*
                })
            },
            1 => {
                branch_using!($rng, {
                    $( $leaf, )*
                    $( $depth1, )*
                })
            },
            _ => {
                branch_using!($rng, {
                    $( $leaf, )*
                    $( $depth1, )*
                    $( $depth2, )*
                })
            }
        }
    };
    ( $depth:expr, $rng: ident,
        { $( $leaf:expr ),* $(,)? },
        { $( $recursive:expr ),* $(,)? }
    ) => {
       if $depth == 0 {
           branch_using!($rng, {
               $( $leaf, )*
           })
       } else {
           branch_using!($rng, {
               $( $leaf, )*
               $( $recursive, )*
           })
       }
   }

}

// -------------------------------------------------------------------------

pub const ACTION_MIN: u64 = MOVE_MIN;
pub fn generate_action_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
    assert!(max_depth >= ACTION_MIN);
    generate_tree!(max_depth - ACTION_MIN, rng, {
        generate_move_expr(rng, max_depth),
        generate_set_velocity_expr(rng, max_depth),
    }, {
        generate_if_expr(generate_action_expr, rng, max_depth),
    })
}

pub const MOVE_MIN: u64 = DIRECTION_MIN + 1;
pub fn generate_move_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
    assert!(max_depth >= MOVE_MIN);
    generate_tree!(max_depth - MOVE_MIN, rng, {
        Box::new(MoveExpr {
            direction: ExprSlot::new(generate_direction_expr(rng, max_depth - 1)),
        }),
    }, {
        generate_if_expr(generate_move_expr, rng, max_depth)
    })
}

pub const SET_VELOCITY_MIN: u64 = DIRECTION_MIN + 1;
pub fn generate_set_velocity_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
    assert!(max_depth >= SET_VELOCITY_MIN);
    generate_tree!(max_depth - SET_VELOCITY_MIN, rng, {
        Box::new(SetVelocityExpr {
            target_velocity: ExprSlot::new(generate_direction_expr(rng, max_depth - 1)),
            max_energy_ratio: ExprSlot::new(generate_fraction_expr(rng, max_depth - 1)),
        }),
    }, {
        generate_if_expr(generate_set_velocity_expr, rng, max_depth)
    })
}

// TODO: this is not what we want longterm
pub const DIRECTION_MIN: u64 = FISH_REF_MIN + 1;
pub fn generate_direction_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Vec2> {
    assert!(max_depth >= DIRECTION_MIN);
    generate_tree!(max_depth - DIRECTION_MIN, rng, {
        Box::new(FishDirectionExpr {
            origin: ExprSlot::new(generate_fish_ref_expr(rng, max_depth - 1)),
            target: ExprSlot::new(generate_fish_ref_expr(rng, max_depth - 1)),
        }),
     }, {
        generate_if_expr(generate_direction_expr, rng, max_depth)
    })
}

pub const FISH_REF_MIN: u64 = 0;
pub fn generate_fish_ref_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<FishRef> {
    generate_tree!(max_depth, rng, {
        Box::new(GetSelfExpr),
        Box::new(DichtsteVisExpr),
    }, {
        generate_if_expr(generate_fish_ref_expr, rng, max_depth),
    })
}

pub fn generate_if_expr<F, T>(generator: F, rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<T>
where
    T: Clone + 'static,
    F: Fn(&mut ExprRng, u64) -> BoxedExpr<T>,
{
    assert!(max_depth >= 1);
    Box::new(IfExpr {
        condition: generate_bool_expr(rng, max_depth - 1).into(),
        consequent: generator(rng, max_depth - 1).into(),
        alternative: generator(rng, max_depth - 1).into(),
    })
}

pub fn generate_bool_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<bool> {
    generate_tree!(max_depth, rng,
    0 => {
        Box::new(ConstExpr::new(true)),
        Box::new(ConstExpr::new(false))
    }, 
    1 => {
        generate_if_expr(generate_bool_expr, rng, max_depth)
    },
    2 => {
        Box::new(LessThenExpr {
            left: generate_f64_expr(rng, max_depth - 1).into(),
            right: generate_f64_expr(rng, max_depth - 1).into(),
        }),
    },)
}

pub const F64_MIN: u64 = 1;
pub fn generate_f64_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<NotNan<f64>> {
    assert!(max_depth > 0);
    generate_tree!(max_depth - F64_MIN, rng, {
        // TODO: Generate random f64
        Box::new(FishEnergyExpr {
            fish: ExprSlot::new(Box::new(GetSelfExpr)),
        }),
        Box::new(FishEnergyExpr {
            fish: ExprSlot::new(Box::new(DichtsteVisExpr)),
        })
    },
    {
        generate_if_expr(generate_f64_expr, rng, max_depth)
    })
}

pub const FRACTION_MIN: u64 = 1;
pub fn generate_fraction_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Fraction> {
    assert!(max_depth > 0);
    generate_tree!(max_depth - FRACTION_MIN, rng, {
        Box::new(ConstExpr::new(Fraction::from_f64(rng.gen_range(0.0..=1.0)))),
    },
    {
        generate_if_expr(generate_fraction_expr, rng, max_depth)
    })
}

pub const COLOR_MIN: u64 = 1;
pub fn generate_color_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Color> {
    assert!(max_depth > 0);
    generate_tree!(max_depth - COLOR_MIN, rng, {
        {
            let color: [f32; 4] = [
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
                rng.gen_range(0.0..=1.0),
                1.0,
                ];
            Box::new(ConstExpr::new(Color::new(color)))
        },
    },
    {
        generate_if_expr(generate_color_expr, rng, max_depth)
    })
}

pub fn wrap_in_generic<T: Clone + 'static>(
    expr: &dyn Expr<T>,
    mut rng: &mut ExprRng,
) -> BoxedExpr<T> {
    branch_using!(rng, {
        Box::new(IfExpr {
            condition: generate_bool_expr(rng, 1).into(),
            consequent: expr.clone_box().into(),
            alternative: expr.mutate(rng).into(),
        }),
        Box::new(IfExpr {
            condition: generate_bool_expr(rng, 1).into(),
            consequent: expr.mutate(rng).into(),
            alternative: expr.clone_box().into(),
        })
    })
}
