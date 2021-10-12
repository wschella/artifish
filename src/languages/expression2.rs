use decorum::NotNan;

use crate::fish::{Action, Fish};
use crate::vec2::*;
use crate::*;

use std::cmp::Ord;

// THE GREAT BEHAVIOURAL INTERPRETER
#[derive(Clone)]
pub struct Program {
    pub root: BoxedExpr<Action>,
}

impl Program {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Program {
            root: Box::new(ConstExpr {
                value: Action::Pass,
            }),
        }
    }

    pub fn random(rng: &mut ExprRng, max_depth: usize) -> Self {
        Program {
            root: generate_action_expr(rng, max_depth),
        }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        return self.root.size();
    }

    pub fn mutate(&mut self, rng: &mut ExprRng) -> Self {
        Program {
            root: self.root.mutate(rng),
        }
    }
}

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

pub fn run_fish(fishes: &Vec<Fish>, fish_num: usize) -> Action {
    let state = InterpreterState { fishes, fish_num };
    let action = fishes[fish_num].program.root.eval(&state);
    action
}

pub struct InterpreterState<'a> {
    fish_num: usize,
    fishes: &'a Vec<Fish>,
}

impl<'a> InterpreterState<'a> {
    fn get_self(&self) -> &'a Fish {
        &self.fishes[self.fish_num]
    }
}

pub type ExprRng = ChaCha20Rng;
pub type BoxedExpr<T> = Box<dyn Expr<T>>;

const ACTION_MIN: usize = MOVE_MIN;
fn generate_action_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<Action> {
    assert!(max_depth >= ACTION_MIN);
    generate_tree!(max_depth - ACTION_MIN, rng, {
        generate_move_expr(rng, max_depth),
    }, {
        generate_if_expr(generate_action_expr, rng, max_depth),
    })
}

const MOVE_MIN: usize = DIRECTION_MIN + 1;
fn generate_move_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<Action> {
    assert!(max_depth >= MOVE_MIN);
    generate_tree!(max_depth - MOVE_MIN, rng, {
        Box::new(MoveExpr {
            direction: generate_direction_expr(rng, max_depth - 1),
        }),
    }, {
        generate_if_expr(generate_move_expr, rng, max_depth)
    })
}

// TODO: this is not what we want longterm
const DIRECTION_MIN: usize = FISH_REF_MIN + 1;
fn generate_direction_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<Vec2> {
    assert!(max_depth >= DIRECTION_MIN);
    generate_tree!(max_depth - DIRECTION_MIN, rng, {
        Box::new(FishDirectionExpr {
            origin: generate_fish_ref_expr(rng, max_depth - 1),
            target: generate_fish_ref_expr(rng, max_depth - 1),
        }),
     }, {
        generate_if_expr(generate_direction_expr, rng, max_depth)
    })
}

const FISH_REF_MIN: usize = 0;
fn generate_fish_ref_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<FishRef> {
    generate_tree!(max_depth, rng, {
        Box::new(GetSelfExpr),
        Box::new(DichtsteVisExpr),
    }, {
        generate_if_expr(generate_fish_ref_expr, rng, max_depth),
    })
}

fn generate_if_expr<F, T>(generator: F, rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<T>
where
    T: Clone + 'static,
    F: Fn(&mut ExprRng, usize) -> BoxedExpr<T>,
{
    assert!(max_depth >= 1);
    Box::new(IfExpr {
        condition: generate_bool_expr(rng, max_depth - 1),
        consequent: generator(rng, max_depth - 1),
        alternative: generator(rng, max_depth - 1),
    })
}

// macro_rules! expr {
//     ( Const( $value:expr )) => {
//         Box::new(ConstExpr::new($value))
//     };
//     ( $expr:expr ) => {
//         $expr
//     }
// }

fn generate_bool_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<bool> {
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
            left: generate_f64_expr(rng, max_depth - 1),
            right: generate_f64_expr(rng, max_depth - 1)
        }),
    },)
}

const F64_MIN: usize = 1;
fn generate_f64_expr(mut rng: &mut ExprRng, max_depth: usize) -> BoxedExpr<NotNan<f64>> {
    assert!(max_depth > 0);
    generate_tree!(max_depth - F64_MIN, rng, {
        // TODO: Generate random f64
        Box::new(FishEnergyExpr {
            fish: Box::new(GetSelfExpr),
        }),
        Box::new(FishEnergyExpr {
            fish: Box::new(DichtsteVisExpr),
        })
    },
    {
        generate_if_expr(generate_f64_expr, rng, max_depth)
    })
}

fn wrap_in_generic<T: Clone + 'static>(expr: &dyn Expr<T>, mut rng: &mut ExprRng) -> BoxedExpr<T> {
    branch_using!(rng, {
        Box::new(IfExpr {
            condition: generate_bool_expr(rng, 1),
            consequent: expr.clone_box(),
            alternative: expr.mutate(rng),
        }),
        Box::new(IfExpr {
            condition: generate_bool_expr(rng, 1),
            consequent: expr.mutate(rng),
            alternative: expr.clone_box(),
        })
    })
}

pub trait Mutable<T> {
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<T>;
}

/// Expression that evaluates to T
pub trait Expr<T>: ExprClone<T> + Mutable<T> {
    fn eval(&self, s: &InterpreterState) -> T;

    fn size(&self) -> u64;
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait ExprClone<T> {
    fn clone_box(&self) -> BoxedExpr<T>;
}

impl<E, T> ExprClone<T> for E
where
    E: Expr<T> + Clone + 'static,
    T: Clone,
{
    fn clone_box(&self) -> BoxedExpr<T> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl<T> Clone for BoxedExpr<T> {
    fn clone(&self) -> BoxedExpr<T> {
        self.clone_box()
    }
}

#[derive(Clone)]
pub struct GetSelfExpr;

impl Expr<FishRef> for GetSelfExpr {
    fn eval(&self, state: &InterpreterState) -> FishRef {
        FishRef {
            maybe_fish_num: Some(state.fish_num),
        }
    }

    fn size(&self) -> u64 {
        1
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

#[derive(Clone)]
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

    fn size(&self) -> u64 {
        1
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

#[derive(Clone)]
pub struct FishEnergyExpr {
    pub fish: Box<dyn Expr<FishRef>>,
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

    fn size(&self) -> u64 {
        1 + self.fish.size()
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

#[derive(Clone)]
pub struct FishDirectionExpr {
    pub origin: Box<dyn Expr<FishRef>>,
    pub target: Box<dyn Expr<FishRef>>,
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

    fn size(&self) -> u64 {
        1 + self.origin.size() + self.target.size()
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

#[derive(Clone)]
pub struct ConstExpr<T> {
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

    fn size(&self) -> u64 {
        1
    }
}

impl Mutable<NotNan<f64>> for ConstExpr<NotNan<f64>> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
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

#[derive(Clone)]
pub struct LessThenExpr<T> {
    pub left: BoxedExpr<T>,
    pub right: BoxedExpr<T>,
}

impl<T> Expr<bool> for LessThenExpr<T>
where
    T: Ord + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> bool {
        self.left.eval(state) < self.right.eval(state)
    }

    fn size(&self) -> u64 {
        1 + self.left.size() + self.right.size()
    }
}

impl<T> Mutable<bool> for LessThenExpr<T>
where
    T: Ord + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<bool> {
        branch_using!(rng, {
            generate_bool_expr(rng, 0),
            wrap_in_generic::<bool>(self, rng),
            Box::new(LessThenExpr {
                left: self.left.clone(),
                right: self.right.mutate(rng),
            }),
            Box::new(LessThenExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone)]
pub struct AddExpr<T> {
    pub left: BoxedExpr<T>,
    pub right: BoxedExpr<T>,
}

impl<T> Expr<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return self.left.eval(state) + self.right.eval(state);
    }

    fn size(&self) -> u64 {
        1 + self.left.size() + self.right.size()
    }
}
// impl Mutable<NotNan<f64>> for AddExpr<NotNan<f64>> {
//     fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
//         todo!()
//     }
// }

impl<T> Mutable<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.left.clone(),
            self.right.clone(),
            Box::new(AddExpr {
                left: self.left.clone(),
                right: self.mutate(rng),
            }),
            Box::new(AddExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone)]
pub struct NotExpr<T> {
    pub value: BoxedExpr<T>,
}

impl<T> Expr<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return !self.value.eval(state);
    }

    fn size(&self) -> u64 {
        1 + self.value.size()
    }
}
impl<T> Mutable<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.clone(),
            Box::new(NotExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone)]
pub struct NegateExpr<T> {
    pub value: BoxedExpr<T>,
}

impl<T> Expr<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return -self.value.eval(state);
    }

    fn size(&self) -> u64 {
        1 + self.value.size()
    }
}
impl<T> Mutable<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.clone(),
            Box::new(NegateExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone)]
pub struct IfExpr<T> {
    pub condition: Box<dyn Expr<bool>>,
    pub consequent: BoxedExpr<T>,
    pub alternative: BoxedExpr<T>,
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

    fn size(&self) -> u64 {
        1 + self.condition.size() + self.consequent.size() + self.alternative.size()
    }
}
impl<T> Mutable<T> for IfExpr<T>
where
    T: Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.consequent.clone(),
            self.alternative.clone(),
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

#[derive(Clone)]
pub struct MoveExpr {
    pub direction: Box<dyn Expr<Vec2>>,
}

impl Expr<Action> for MoveExpr {
    fn eval(&self, state: &InterpreterState) -> Action {
        let dir_vec = self.direction.eval(state);
        Action::Move(dir_vec)
    }

    fn size(&self) -> u64 {
        1 + self.direction.size()
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
