use decorum::NotNan;

use crate::*;
use crate::vec2::*;

// THE GREAT BEHAVIOURAL INTERPRETER

#[derive(Clone, Debug)]
pub struct Program {
    commands: Vec<Command>
}

impl Program {
    pub fn empty() -> Self {
        Program { commands: vec![] }
    }
}

pub fn run_away_program() -> Program {
    Program {
        commands: vec![
            Command::PushDirectionToClosestFish,
            Command::InvertVec,
            Command::MoveInDirection, 
        ]
    }
}

pub fn run_towards_program() -> Program {
    Program {
        commands: vec![
            Command::PushDirectionToClosestFish,
            Command::MoveInDirection, 
        ]
    }
}

#[derive(Copy, Clone, Debug)]
enum Command {
    PushDirectionToClosestFish,
    InvertVec,
    MoveInDirection,
}

#[derive(Clone, Debug)]
struct StackSet {
    vec2_stack: Vec<Vec2>,
}

impl StackSet {
    pub fn empty() -> Self {
        StackSet {
            vec2_stack: Vec::new(),
        }
    }
    
    pub fn empty_vec2_stack(&self) -> Vec2 {
        Vec2 { x: 0.0, y: 0.0 }
    }
}

// Currently we return on the first action, ultimately we want to allow multiple actions,
// humans can do that as well, but the cost should increase as the actions (or a singular 
// parametrized one) does more stuff. E.g. if you move further, it should cost more,
// and this should ramp up superlinearly. 
pub fn run_fish(fishes: &Vec<Fish>, fish_num: usize) -> Action {
    let mut stack_set = StackSet::empty();

    for command in fishes[fish_num].program.commands.iter() {
        match command {
            &Command::PushDirectionToClosestFish => {
                let maybe_j = fishes
                    .iter()
                    .enumerate()
                    .filter(|(j, _)| j != &fish_num)
                    .min_by_key(|(_, fish)| NotNan::from_inner(fishes[fish_num].distance(fish)))
                    .map(|(j, _)| j);
            
                if let Some(j) = maybe_j {
                    let direction = fishes[fish_num].direction_to(&fishes[j]);
                    stack_set.vec2_stack.push(direction);
                }
    
            }
            &Command::InvertVec => {
                if let Some(direction) = stack_set.vec2_stack.pop() {
                    stack_set.vec2_stack.push(direction.invert());
                }
            }
            &Command::MoveInDirection => {
                if let Some(direction) = stack_set.vec2_stack.pop() {
                    return Action::Move(direction);
                } else {
                    return Action::Move(stack_set.empty_vec2_stack())
                }
            }
        }
    }

    return Action::Pass;

}