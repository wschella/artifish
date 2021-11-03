use rand::Rng;
use rand_chacha::ChaCha20Rng;

use super::expressions::ConstExpr;
use super::generators::generate_action_expr;
use crate::fish::{Action, Fish};

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

    pub fn mutated(&self, rng: &mut ExprRng) -> Self {
        let mut new_program = self.clone();
        new_program.mutate(rng);
        new_program
    }
}

pub struct InterpreterState<'a> {
    pub fish_num: usize,
    pub fishes: &'a Vec<Fish>,
}

impl<'a> InterpreterState<'a> {
    pub fn get_self(&self) -> &'a Fish {
        &self.fishes[self.fish_num]
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
    pub inner: BoxedExpr<T>,
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
