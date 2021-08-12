use decorum::NotNan;

use crate::*;
use crate::vec2::*;


use std::ops::{Add, Sub, Mul, Div};
use std::cmp::{Ord, Eq};


// THE GREAT BEHAVIOURAL INTERPRETER
#[derive(Clone)]
pub struct Program {
    root: Box<dyn Expr<Action>>,
}

impl Program {
    pub fn empty() -> Self {
        Program { root: Box::new(ConstExpr { value: Action::Pass }) }
    }
}

pub fn run_away_program() -> Program {
    Program { root: todo!()  }
}

pub fn run_towards_program() -> Program {
    Program {
      root: todo!()
    }
}

pub fn run_fish(fishes: &Vec<Fish>, fish_num: usize) -> Action {
  todo!()
}

#[derive(Clone)]
struct State;

/// Expression that evaluates to T
trait Expr<T> : ExprClone<T> {
    fn eval(&self, s: &State) -> T;
}

// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
trait ExprClone<T> {
  fn clone_box(&self) -> Box<dyn Expr<T>>;
}

impl<E, T> ExprClone<T> for E
where
    E: Expr<T> + Clone + 'static,
    T: Clone
{
    fn clone_box(&self) -> Box<dyn Expr<T>> {
        Box::new(self.clone())
    }
}

// We can now implement Clone manually by forwarding to clone_box.
impl <T> Clone for Box<dyn Expr<T>> {
  fn clone(&self) -> Box<dyn Expr<T>> {
      self.clone_box()
  }
}

#[derive(Clone)]
struct ConstExpr<T> {
    value: T
}

impl<T> Expr<T> for ConstExpr<T>
    where T: Clone + 'static
{
    fn eval(&self, _: &State) -> T {
        return self.value.clone();
    }
}

#[derive(Clone)]
struct LessThen<T> {
    left: Box<dyn Expr<T>>,
    right: Box<dyn Expr<T>>,
}

impl<T> Expr<bool> for LessThen<T>
    where T: Ord + Clone + 'static
{
    fn eval(&self, state: &State) -> bool {
        self.left.eval(state) < self.right.eval(state)
    }
}

#[derive(Clone)]
struct AddExpr<T> {
  left: Box<dyn Expr<T>>,
  right: Box<dyn Expr<T>>,
}

impl<T> Expr<T> for AddExpr<T>
    where T: std::ops::Add<Output = T> + Clone + 'static
{
  fn eval(&self, state: &State) -> T {
    return self.left.eval(state) + self.right.eval(state)
  }
}

#[derive(Clone)]
struct NotExpr<T> {
  value: Box<dyn Expr<T>>,
}

impl<T> Expr<T> for NotExpr<T>
    where T: std::ops::Not<Output = T> + Clone + 'static
{
  fn eval(&self, state: &State) -> T {
    return !self.value.eval(state)
  }
}

#[derive(Clone)]
struct NegateExpr<T> {
  value: Box<dyn Expr<T>>,
}

impl<T> Expr<T> for NegateExpr<T>
    where T: std::ops::Neg<Output = T> + Clone + 'static
{
  fn eval(&self, state: &State) -> T {
    return -self.value.eval(state)
  }
}

#[derive(Clone)]
struct IfExpr<T> {
    condition: Box<dyn Expr<bool>>,
    consequent: Box<dyn Expr<T>>,
    alternative: Box<dyn Expr<T>>,
}

impl<T> Expr<T> for IfExpr<T>
    where T: Clone + 'static
{
    fn eval(&self, state: &State) -> T {
        if self.condition.eval(state) {
            self.consequent.eval(state)
        } else {
            self.alternative.eval(state)
        }
    }
}