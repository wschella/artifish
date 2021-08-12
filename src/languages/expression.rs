use decorum::NotNan;

use crate::*;
use crate::vec2::*;


// THE GREAT BEHAVIOURAL INTERPRETER

#[derive(Clone, Debug)]
pub struct Program {
    root: ActionExpr,
}

impl Program {
    pub fn empty() -> Self {
        Program { root: ActionExpr::Pass }
    }
}

pub fn run_away_program() -> Program {
    Program {
      root: Move(
        If(


        )
      ),
    }
}

pub fn run_towards_program() -> Program {
    Program {
      root: Expr::NeeWoutNopIsEenInstructieGeenExpr
    }
}

#[derive(Clone, Debug)]
enum ActionExpr {
  Pass,
  Move(Vec2Expr),
  NeeWoutNopIsEenInstructieGeenExpr
}

#[derive(Clone, Debug)]
enum Vec2Expr {
    Const(Vec2),
    Add(Box<Vec2Expr>, Box<Vec2Expr>),
    Invert(Box<Vec2Expr>),
    Combined(Box<CombinatorExpr<Vec2Expr>>),
    ClosestFish,
}

enum FloatExpr {
    Const(f64),
    Add(Box<FloatExpr>, Box<FloatExpr>),
    Subtract(Box<FloatExpr>, Box<FloatExpr>),
    Divide(Box<FloatExpr>, Box<FloatExpr>),
    Multiply(Box<FloatExpr>, Box<FloatExpr>),
    Negate(Box<FloatExpr>),
    EnergyOfClosestFish,
}


#[derive(Clone, Debug)]
enum CombinatorExpr<T> {
    If {
        conditional: BooleanExpr,
        consequent: T,
        alternative: T,
    }
}

#[derive(Clone, Debug)]
enum BooleanExpr {
    Const(bool),

    Not(Box<BooleanExpr>),
    And {
        left: Box<BooleanExpr>,
        right: Box<BooleanExpr>,
    },
    Or {
      left: Box<BooleanExpr>,
      right: Box<BooleanExpr>,
    },
    Combined(Box<CombinatorExpr<BooleanExpr>>)
}





// Currently we return on the first action, ultimately we want to allow multiple actions,
// humans can do that as well, but the cost should increase as the actions (or a singular 
// parametrized one) does more stuff. E.g. if you move further, it should cost more,
// and this should ramp up superlinearly. 
pub fn run_fish(fishes: &Vec<Fish>, fish_num: usize) -> Action {
  todo!()
}
