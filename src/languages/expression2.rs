use decorum::NotNan;

use crate::vec2::*;
use crate::*;

use std::cmp::Ord;

// THE GREAT BEHAVIOURAL INTERPRETER
#[derive(Clone)]
pub struct Program {
    root: BoxedExpr<Action>,
}

impl Program {
    pub fn empty() -> Self {
        Program {
            root: Box::new(ConstExpr {
                value: Action::Pass,
            }),
        }
    }

    pub fn random(rng: &mut ExprRng) -> Self {
        Program {
            root: generate_action_expr(rng),
        }
    }

    pub fn size(&self) -> u64 {
        return self.root.size();
    }

    pub fn mutate(&mut self, rng: &mut ExprRng) -> Self {
        Program {
            root: self.root.mutate(rng),
        }
    }
}

pub fn run_towards_program() -> Program {
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

pub fn run_fish(fishes: &Vec<Fish>, fish_num: usize) -> Action {
    let state = InterpreterState { fishes, fish_num };
    let action = fishes[fish_num].program.root.eval(&state);
    action
}

struct InterpreterState<'a> {
    fish_num: usize,
    fishes: &'a Vec<Fish>,
}

impl<'a> InterpreterState<'a> {
    fn get_self(&self) -> &'a Fish {
        &self.fishes[self.fish_num]
    }
}

type ExprRng = ChaCha20Rng;
type BoxedExpr<T> = Box<dyn Expr<T>>;

fn generate_action_expr(mut rng: &mut ExprRng) -> BoxedExpr<Action> {
    branch_using!(rng, {
        generate_move_expr(rng),
        // generate_if_expr(generate_action_expr, rng),
    })
}

fn generate_move_expr(mut rng: &mut ExprRng) -> BoxedExpr<Action> {
    branch_using!(rng, {
        Box::new(MoveExpr {
            direction: generate_direction_expr(rng),
        }),
        // generate_if_expr(generate_move_expr, rng)
    })
}

// TODO: this is not what we want longterm
fn generate_direction_expr(mut rng: &mut ExprRng) -> BoxedExpr<Vec2> {
    branch_using!(rng, {
        Box::new(FishDirectionExpr {
            origin: generate_fish_ref_expr(rng),
            target: generate_fish_ref_expr(rng),
        }),
        // generate_if_expr(generate_direction_expr, rng)
    })
}

fn generate_fish_ref_expr(mut rng: &mut ExprRng) -> BoxedExpr<FishRef> {
    branch_using!(rng, {
        Box::new(GetSelfExpr),
        Box::new(DichtsteVisExpr),
        // generate_if_expr(generate_fish_ref_expr, rng)
    })
}

fn generate_if_expr<F, T>(generator: F, rng: &mut ExprRng) -> BoxedExpr<T>
where
    T: Clone + 'static,
    F: Fn(&mut ExprRng) -> BoxedExpr<T>,
{
    dbg!("test");
    Box::new(IfExpr {
        condition: generate_bool_expr(rng),
        consequent: generator(rng),
        alternative: generator(rng),
    })
}

fn generate_bool_expr(mut rng: &mut ExprRng) -> BoxedExpr<bool> {
    branch_using!(rng, {
        Box::new(ConstExpr { value: true }),
        Box::new(ConstExpr { value: false }),
        Box::new(LessThenExpr {
             left: generate_f64_expr(rng),
             right: generate_f64_expr(rng)
        }),
        // generate_if_expr(generate_bool_expr, rng)
    })
}

fn generate_f64_expr(mut rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
    branch_using!(rng, {
        // TODO: Generate random f64
        Box::new(FishEnergyExpr {
            fish: Box::new(GetSelfExpr),
        }),
        Box::new(FishEnergyExpr {
            fish: Box::new(DichtsteVisExpr),
        }),
        // generate_if_expr(generate_f64_expr, rng)
    })
}

fn wrap_in_generic<T: Clone + 'static>(expr: &dyn Expr<T>, mut rng: &mut ExprRng) -> BoxedExpr<T> {
    branch_using!(rng, {
        Box::new(IfExpr {
            condition: generate_bool_expr(rng),
            consequent: expr.clone_box(),
            alternative: expr.mutate(rng),
        }),
        Box::new(IfExpr {
            condition: generate_bool_expr(rng),
            consequent: expr.mutate(rng),
            alternative: expr.clone_box(),
        })
    })
}

trait Mutable<T> {
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<T>;
}

/// Expression that evaluates to T
trait Expr<T>: ExprClone<T> + Mutable<T> {
    fn eval(&self, s: &InterpreterState) -> T;

    fn size(&self) -> u64;
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
trait ExprClone<T> {
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
struct GetSelfExpr;

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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<FishRef> {
        generate_fish_ref_expr(rng)
    }
}

#[derive(Clone)]
struct DichtsteVisExpr;

#[derive(Clone, Debug)]
struct FishRef {
    maybe_fish_num: Option<usize>,
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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<FishRef> {
        generate_fish_ref_expr(rng)
    }
}

#[derive(Clone)]
struct FishEnergyExpr {
    fish: Box<dyn Expr<FishRef>>,
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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
        generate_f64_expr(rng)
    }
}

#[derive(Clone)]
struct FishDirectionExpr {
    origin: Box<dyn Expr<FishRef>>,
    target: Box<dyn Expr<FishRef>>,
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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<Vec2> {
        generate_direction_expr(rng)
    }
}

#[derive(Clone)]
struct ConstExpr<T> {
    value: T,
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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<NotNan<f64>> {
        generate_f64_expr(rng)
    }
}

impl Mutable<Action> for ConstExpr<Action> {
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<Action> {
        generate_action_expr(rng)
    }
}

impl Mutable<bool> for ConstExpr<bool> {
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<bool> {
        generate_bool_expr(rng)
    }
}

#[derive(Clone)]
struct LessThenExpr<T> {
    left: BoxedExpr<T>,
    right: BoxedExpr<T>,
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
            generate_bool_expr(rng),
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
struct AddExpr<T> {
    left: BoxedExpr<T>,
    right: BoxedExpr<T>,
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
struct NotExpr<T> {
    value: BoxedExpr<T>,
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
            self.value.clone(),
            Box::new(NotExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone)]
struct NegateExpr<T> {
    value: BoxedExpr<T>,
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
            self.value.clone(),
            Box::new(NegateExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone)]
struct IfExpr<T> {
    condition: Box<dyn Expr<bool>>,
    consequent: BoxedExpr<T>,
    alternative: BoxedExpr<T>,
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
struct MoveExpr {
    direction: Box<dyn Expr<Vec2>>,
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
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<Action> {
        Box::new(MoveExpr {
            direction: self.direction.mutate(rng),
        })
    }
}
