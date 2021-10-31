use decorum::{NotNan, N64};

use crate::fish::{Action, Fish};
use crate::vec2::*;
use crate::*;

use std::cmp::Ord;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Fraction(pub f64);

impl Fraction {
    pub fn from_f64(inner: f64) -> Self {
        assert!(!inner.is_nan());
        assert!(inner >= 0.0);
        assert!(inner <= 1.0);
        Fraction(inner)
    }
}

impl From<Fraction> for N64 {
    fn from(f: Fraction) -> Self {
        let Fraction(inner) = f;
        N64::from_inner(inner)
    }
}

// THE GREAT BEHAVIOURAL INTERPRETER
#[derive(Clone)]
pub struct Program {
    pub root: ExprSlot<Action>,
}

impl Program {
    #[allow(dead_code)]
    pub fn empty() -> Self {
        let root: Box<dyn Expr<Action>> = Box::new(ConstExpr {
            value: Action::Pass,
        });
        Program { root: root.into() }
    }

    pub fn random(rng: &mut ExprRng, max_depth: u64) -> Self {
        Program {
            root: ExprSlot {
                inner: generate_action_expr(rng, max_depth),
            },
        }
    }

    pub fn run(&self, state: &InterpreterState) -> Action {
        self.root.inner.eval(state)
    }

    #[allow(dead_code)]
    pub fn size(&self) -> u64 {
        return self.root.size();
    }

    pub fn mutate(&mut self, rng: &mut ExprRng) {
        let total_size = self.root.size();
        let index: u64 = rng.gen_range(0..total_size);

        let path_to_node = find_node(&mut self.root, index);
        let node = get_node(&mut self.root, path_to_node.as_found());
        node.mutate_expr(rng);
    }
}

pub struct InterpreterState<'a> {
    pub fish_num: usize,
    pub fishes: &'a Vec<Fish>,
}

impl<'a> InterpreterState<'a> {
    fn get_self(&self) -> &'a Fish {
        &self.fishes[self.fish_num]
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

// -------------------------------------------------------------------------

pub type ExprRng = ChaCha20Rng;
pub type BoxedExpr<T> = Box<dyn Expr<T>>;

pub trait Mutable<T> {
    fn mutate(&self, rng: &mut ExprRng) -> BoxedExpr<T>;
}

/// Expression that evaluates to T
pub trait Expr<T>: ExprClone<T> + Mutable<T> + ExprTreeNode {
    fn eval(&self, s: &InterpreterState) -> T;
}

// This split of from the main Expr trait mainly because we generate impl for this one
// via derive proc-macros.
pub trait ExprTreeNode {
    fn borrow_nth_child(&self, n: u64) -> &dyn MutableExprSlot;
    fn borrow_nth_child_mut(&mut self, n: u64) -> &mut dyn MutableExprSlot;
    fn num_children(&self) -> u64;
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

pub struct ExprSlot<T> {
    inner: BoxedExpr<T>,
}

impl<T> Clone for ExprSlot<T> {
    fn clone(&self) -> ExprSlot<T> {
        ExprSlot {
            inner: self.inner.clone_box(),
        }
    }
}

impl<T> ExprSlot<T> {
    pub fn new(expr: BoxedExpr<T>) -> Self {
        ExprSlot { inner: expr }
    }

    pub fn eval(&self, s: &InterpreterState) -> T {
        self.inner.eval(s)
    }

    pub fn mutate(&self, rng: &mut ExprRng) -> ExprSlot<T> {
        Self {
            inner: self.inner.mutate(rng),
        }
    }
}

impl<T> From<BoxedExpr<T>> for ExprSlot<T> {
    fn from(expr: BoxedExpr<T>) -> Self {
        ExprSlot::new(expr)
    }
}

pub trait MutableExprSlot {
    fn mutate_expr(&mut self, rng: &mut ExprRng);
    fn borrow_nth_child_mut(&mut self, n: u64) -> &mut dyn MutableExprSlot;
    fn borrow_nth_child(&self, n: u64) -> &dyn MutableExprSlot;
    fn num_children(&self) -> u64;

    fn size(&self) -> u64 {
        let children_size = (0..self.num_children())
            .map(|child_num| self.borrow_nth_child(child_num).size())
            .sum::<u64>();
        children_size + 1
    }
}

impl<T> MutableExprSlot for ExprSlot<T> {
    fn mutate_expr(&mut self, rng: &mut ExprRng) {
        self.inner = self.inner.mutate(rng)
    }

    fn borrow_nth_child_mut(&mut self, n: u64) -> &mut dyn MutableExprSlot {
        self.inner.borrow_nth_child_mut(n)
    }

    fn borrow_nth_child(&self, n: u64) -> &dyn MutableExprSlot {
        self.inner.borrow_nth_child(n)
    }

    fn num_children(&self) -> u64 {
        self.inner.num_children()
    }
}

// -------------------------------------------------------------------------

enum FindNodeResult {
    NumVisited(u64),
    FoundNode(Vec<u64>),
}

impl FindNodeResult {
    fn as_found(self) -> Vec<u64> {
        match self {
            FindNodeResult::NumVisited(_) => panic!("node not found"),
            FindNodeResult::FoundNode(path) => path,
        }
    }
}

fn get_node<'a>(
    root: &'a mut dyn MutableExprSlot,
    reverse_path: Vec<u64>,
) -> &'a mut dyn MutableExprSlot {
    let mut pos = root;

    for &child_index in reverse_path.iter().rev() {
        pos = pos.borrow_nth_child_mut(child_index);
    }

    return pos;
}

fn find_node<'a>(root: &'a mut dyn MutableExprSlot, index: u64) -> FindNodeResult {
    use FindNodeResult::*;

    if index == 0 {
        return FoundNode(vec![]);
    }

    let mut num_visited: u64 = 1; // we visited root

    for i in 0..root.num_children() {
        let child = root.borrow_nth_child_mut(i);
        match find_node(child, index - num_visited) {
            FoundNode(mut reverse_path) => {
                reverse_path.push(i);
                return FoundNode(reverse_path);
            }
            NumVisited(count) => num_visited += count,
        }
    }
    NumVisited(num_visited)
}

// -------------------------------------------------------------------------

const ACTION_MIN: u64 = MOVE_MIN;
fn generate_action_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
    assert!(max_depth >= ACTION_MIN);
    generate_tree!(max_depth - ACTION_MIN, rng, {
        generate_move_expr(rng, max_depth),
        generate_set_velocity_expr(rng, max_depth),
    }, {
        generate_if_expr(generate_action_expr, rng, max_depth),
    })
}

const MOVE_MIN: u64 = DIRECTION_MIN + 1;
fn generate_move_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
    assert!(max_depth >= MOVE_MIN);
    generate_tree!(max_depth - MOVE_MIN, rng, {
        Box::new(MoveExpr {
            direction: ExprSlot::new(generate_direction_expr(rng, max_depth - 1)),
        }),
    }, {
        generate_if_expr(generate_move_expr, rng, max_depth)
    })
}

const SET_VELOCITY_MIN: u64 = DIRECTION_MIN + 1;
fn generate_set_velocity_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Action> {
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
const DIRECTION_MIN: u64 = FISH_REF_MIN + 1;
fn generate_direction_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Vec2> {
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

const FISH_REF_MIN: u64 = 0;
fn generate_fish_ref_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<FishRef> {
    generate_tree!(max_depth, rng, {
        Box::new(GetSelfExpr),
        Box::new(DichtsteVisExpr),
    }, {
        generate_if_expr(generate_fish_ref_expr, rng, max_depth),
    })
}

fn generate_if_expr<F, T>(generator: F, rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<T>
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

fn generate_bool_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<bool> {
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

const F64_MIN: u64 = 1;
fn generate_f64_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<NotNan<f64>> {
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

const FRACTION_MIN: u64 = 1;
fn generate_fraction_expr(mut rng: &mut ExprRng, max_depth: u64) -> BoxedExpr<Fraction> {
    assert!(max_depth > 0);
    generate_tree!(max_depth - FRACTION_MIN, rng, {
        Box::new(ConstExpr::new(Fraction::from_f64(rng.gen_range(0.0..=1.0)))),
    },
    {
        generate_if_expr(generate_fraction_expr, rng, max_depth)
    })
}

fn wrap_in_generic<T: Clone + 'static>(expr: &dyn Expr<T>, mut rng: &mut ExprRng) -> BoxedExpr<T> {
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

// -------------------------------------------------------------------------

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

#[derive(Clone, ArtifishExpr)]
pub struct ConstExpr<T> {
    #[expr_tree_node(not_a_child)]
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

impl Mutable<Fraction> for ConstExpr<Fraction> {
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<Fraction> {
        branch_using!(rng, {
            wrap_in_generic(self, rng),
            generate_fraction_expr(rng, FRACTION_MIN),
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct LessThenExpr<T> {
    pub left: ExprSlot<T>,
    pub right: ExprSlot<T>,
}

impl<T> Expr<bool> for LessThenExpr<T>
where
    T: Ord + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> bool {
        self.left.eval(state) < self.right.eval(state)
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

#[derive(Clone, ArtifishExpr)]
pub struct AddExpr<T> {
    pub left: ExprSlot<T>,
    pub right: ExprSlot<T>,
}

impl<T> Expr<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return self.left.eval(state) + self.right.eval(state);
    }
}

impl<T> Mutable<T> for AddExpr<T>
where
    T: std::ops::Add<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.left.inner.clone(),
            self.right.inner.clone(),
            Box::new(AddExpr {
                left: self.left.clone(),
                right: ExprSlot::new(self.mutate(rng)),
            }),
            Box::new(AddExpr {
                left: self.left.mutate(rng),
                right: self.right.clone(),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct NotExpr<T> {
    pub value: ExprSlot<T>,
}

impl<T> Expr<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return !self.value.eval(state);
    }
}

impl<T> Mutable<T> for NotExpr<T>
where
    T: std::ops::Not<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.inner.clone(),
            Box::new(NotExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct NegateExpr<T> {
    pub value: ExprSlot<T>,
}

impl<T> Expr<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn eval(&self, state: &InterpreterState) -> T {
        return -self.value.eval(state);
    }
}

impl<T> Mutable<T> for NegateExpr<T>
where
    T: std::ops::Neg<Output = T> + Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.value.inner.clone(),
            Box::new(NegateExpr {
                value: self.value.mutate(rng),
            })
        })
    }
}

#[derive(Clone, ArtifishExpr)]
pub struct IfExpr<T> {
    pub condition: ExprSlot<bool>,
    pub consequent: ExprSlot<T>,
    pub alternative: ExprSlot<T>,
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
}

impl<T> Mutable<T> for IfExpr<T>
where
    T: Clone + 'static,
{
    fn mutate(&self, mut rng: &mut ExprRng) -> BoxedExpr<T> {
        branch_using!(rng, {
            wrap_in_generic::<T>(self, rng),
            self.consequent.inner.clone(),
            self.alternative.inner.clone(),
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
