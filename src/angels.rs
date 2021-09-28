use super::languages::lang::*;

pub fn smartie() -> Program {
    Program {
        root: Box::new(MoveExpr {
            direction: Box::new(IfExpr {
                // if dichtste_vis.energy < self.energy
                condition: Box::new(LessThenExpr {
                    left: Box::new(FishEnergyExpr {
                        fish: Box::new(DichtsteVisExpr),
                    }),
                    right: Box::new(FishEnergyExpr {
                        fish: Box::new(GetSelfExpr),
                    }),
                }),
                // then move towards
                consequent: Box::new(FishDirectionExpr {
                    origin: Box::new(GetSelfExpr),
                    target: Box::new(DichtsteVisExpr),
                }),
                // else run away
                alternative: Box::new(FishDirectionExpr {
                    origin: Box::new(DichtsteVisExpr),
                    target: Box::new(GetSelfExpr),
                }),
            }),
        }),
    }
}

// weten waar de rand is
// movement speed parametriseren
// grootste vis in radius
// grootste vis en kleinste vis
// zwaartekracht
// splitactie
