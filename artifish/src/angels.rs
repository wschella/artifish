use decorum::{NotNan, N64};
use rand::Rng;
use rand_chacha::ChaCha20Rng;

use super::lang::*;
use crate::{
    color::Color,
    fish::{Action, Fish},
    vec2::Vec2,
};

fn node<E, T>(expr: E) -> ExprSlot<T>
where
    E: 'static + Expr<T>,
{
    ExprSlot::new(Box::new(expr))
}

pub fn smartie() -> Program {
    Program {
        root: node(SetVelocityExpr {
            target_velocity: node(MulExpr {
                left: node(ConstExpr::new(Fraction::from_f64(0.2))),
                right: node(IfExpr {
                    // if dichtste_vis.energy < self.energy
                    condition: node(LessThenExpr {
                        left: node(FishEnergyExpr {
                            fish: node(DichtsteVisExpr),
                        }),
                        right: node(FishEnergyExpr {
                            fish: node(GetSelfExpr),
                        }),
                    }),
                    // then move towards
                    consequent: node(FishDirectionExpr {
                        origin: node(GetSelfExpr),
                        target: node(DichtsteVisExpr),
                    }),
                    // else run away
                    alternative: node(FishDirectionExpr {
                        origin: node(DichtsteVisExpr),
                        target: node(GetSelfExpr),
                    }),
                }),
            }),

            max_energy_ratio: node(ConstExpr::new(Fraction::from_f64(0.05))),
        }),
    }
}

pub fn toast_niet_kannibaal() -> Program {
    let smartie = smartie();

    Program {
        root: node(IfExpr {
            condition: node(LessThenExpr {
                left: node(ConstExpr::new(Fraction::from_f64(0.9))),
                right: node(ColorSimilarityExpr {
                    lhs: node(FishColorExpr {
                        fish: node(DichtsteVisExpr),
                    }),
                    rhs: node(FishColorExpr {
                        fish: node(GetSelfExpr),
                    }),
                }),
            }),
            consequent: node(ConstExpr::new(Action::Pass)),
            alternative: smartie.root,
        }),
    }
}

pub fn ass_is_grass() -> Program {
    Program {
        root: node(IfExpr {
            condition: node(LessThenExpr {
                left: node(FishEnergyExpr {
                    fish: node(GetSelfExpr),
                }),
                right: node(ConstExpr::new(N64::from_inner(10_000.0))),
            }),
            consequent: node(ConstExpr::new(Action::Pass)),
            alternative: node(SplitExpr {
                impulse: node(FishDirectionExpr {
                    origin: node(DichtsteVisExpr),
                    target: node(GetSelfExpr),
                }),
                mass_fraction: node(ConstExpr::new(Fraction::from_f64(0.2))),
            }),
        }),
    }
}

fn make_angel(rng: &mut ChaCha20Rng, program: Program, color: Color, tag: &str) -> Fish {
    Fish {
        x: rng.gen_range(0.0..crate::MAX_X),
        y: rng.gen_range(0.0..crate::MAX_Y),
        energy: NotNan::from_inner(500.0),
        velocity: Vec2::zero(),
        program: program,
        color: color,
        is_man_made: true,
        tag: Some(tag.to_owned()),
    }
}

pub fn generate_angel(mut rng: &mut ChaCha20Rng) -> Fish {
    branch_using!(rng, {
        make_angel(rng, smartie(), Color::RED, "SMRT"),
        make_angel(rng, toast_niet_kannibaal(), Color::BLUE, "TNK"),
        make_angel(rng, ass_is_grass(), Color::GREEN, "ASS"),
    })
}

// weten waar de rand is
// movement speed parametriseren
// grootste vis in radius
// grootste vis en kleinste vis
// zwaartekracht
