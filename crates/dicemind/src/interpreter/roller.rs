use rand::{distributions::Uniform, Rng};

use crate::{parser::BinaryOperator, parser::Integer, visitor::Visitor};

pub struct SimpleRoller {
    default_dice_size: Integer,
}

impl SimpleRoller {
    pub fn new(default_dice: impl Into<Integer>) -> Self {
        Self {
            default_dice_size: default_dice.into(),
        }
    }
}

impl Default for SimpleRoller {
    fn default() -> Self {
        SimpleRoller::new(6)
    }
}

impl Visitor<Integer> for SimpleRoller {
    fn visit_dice(&mut self, amount: Option<Integer>, power: Option<Integer>) -> Integer {
        let amount = amount.unwrap_or(1.into());
        let power = power.unwrap_or(self.default_dice_size.clone());

        rand::thread_rng().sample(Uniform::new_inclusive(amount.clone(), amount * power))
    }

    fn visit_constant(&mut self, c: Integer) -> Integer {
        c
    }

    fn visit_binop(&mut self, op: BinaryOperator, lhs: Integer, rhs: Integer) -> Integer {
        use crate::parser::BinaryOperator::*;

        match op {
            Equals => (lhs == rhs).into(),
            LessThan => (lhs < rhs).into(),
            GreaterThan => (lhs > rhs).into(),
            Add => lhs + rhs,
            Subtract => lhs - rhs,
            Multiply => lhs * rhs,
        }
    }
}
