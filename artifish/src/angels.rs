use super::languages::lang::*;

pub fn smartie() -> Program {
    Program::empty()
    // Program {
    // root: todo!()
    // root: Box::new(MoveExpr {
    //     direction: Box::new(IfExpr {
    //         // if dichtste_vis.energy < self.energy
    //         condition: Box::new(LessThenExpr {
    //             left: Box::new(FishEnergyExpr {
    //                 fish: Box::new(DichtsteVisExpr).into(),
    //             }),
    //             right: Box::new(FishEnergyExpr {
    //                 fish: Box::new(GetSelfExpr).into(),
    //             }),
    //         }),
    //         // then move towards
    //         consequent: Box::new(FishDirectionExpr {
    //             origin: Box::new(GetSelfExpr).into(),
    //             target: Box::new(DichtsteVisExpr).into(),
    //         }),
    //         // else run away
    //         alternative: Box::new(FishDirectionExpr {
    //             origin: Box::new(DichtsteVisExpr).into(),
    //             target: Box::new(GetSelfExpr).into(),
    //         }),
    //     }),
    // }).into(),
    // }
}

// weten waar de rand is
// movement speed parametriseren
// grootste vis in radius
// grootste vis en kleinste vis
// zwaartekracht
// splitactie
