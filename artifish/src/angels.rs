use super::languages::lang::*;

fn node<E, T>(expr: E) -> ExprSlot<T>
where
    E: 'static + Expr<T>,
{
    ExprSlot::new(Box::new(expr))
}

pub fn smartie() -> Program {
    Program {
        root: node(SetVelocityExpr {
            target_velocity: node(IfExpr {
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
            max_energy_ratio: node(ConstExpr::new(Fraction::from_f64(0.01))),
        }),
    }
}

// weten waar de rand is
// movement speed parametriseren
// grootste vis in radius
// grootste vis en kleinste vis
// zwaartekracht
// splitactie
