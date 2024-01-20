use num::bigint::{RandBigInt, Sign};
use num::{range, One, Zero};
use rand::thread_rng;
use thiserror::Error;

use crate::parser::*;
use crate::visitor::Visitor;

pub struct FastRoller {
    default_count: u32,
    default_power: u32,
}

impl Default for FastRoller {
    fn default() -> Self {
        Self::new(1, 6)
    }
}

impl FastRoller {
    pub fn new(default_count: u32, default_power: u32) -> Self {
        Self {
            default_count,
            default_power,
        }
    }
}

#[derive(Debug, Error)]
pub enum FastRollerError {
    #[error("Value too large and can't fit inside 2^32!")]
    ValueTooLarge,
    #[error("The value has overflown! Too large!")]
    Overflow,
}

impl Visitor<Result<i32, FastRollerError>> for FastRoller {
    fn visit_dice(
        &mut self,
        count: Option<Integer>,
        power: Option<PositiveInteger>,
    ) -> Result<i32, FastRollerError> {
        use FastRollerError::*;

        let (sign, count) = count
            .unwrap_or(Integer::from(self.default_count))
            .into_parts();
        let power = power.unwrap_or(PositiveInteger::from(self.default_power));

        let mut rng = thread_rng();
        let mut sum = 0i32;

        for _ in range(PositiveInteger::zero(), count) {
            let n = rng.gen_biguint_below(&power) + PositiveInteger::one();

            sum = sum
                .checked_add(n.try_into().map_err(|_| ValueTooLarge)?)
                .ok_or(Overflow)?;
        }

        Ok(if sign == Sign::Minus { -sum } else { sum })
    }

    fn visit_constant(&mut self, c: Integer) -> Result<i32, FastRollerError> {
        c.try_into().map_err(|_| FastRollerError::ValueTooLarge)
    }

    fn visit_binop(
        &mut self,
        op: BinaryOperator,
        lhs: Result<i32, FastRollerError>,
        rhs: Result<i32, FastRollerError>,
    ) -> Result<i32, FastRollerError> {
        use BinaryOperator::*;
        use FastRollerError::*;

        match op {
            Equals => Ok((lhs? == rhs?) as i32),
            LessThan => Ok((lhs? < rhs?) as i32),
            GreaterThan => Ok((lhs? > rhs?) as i32),
            Add => lhs?.checked_add(rhs?).ok_or(Overflow),
            Subtract => lhs?.checked_sub(rhs?).ok_or(Overflow),
            Multiply => lhs?.checked_mul(rhs?).ok_or(Overflow),
        }
    }

    fn visit_negation(
        &mut self,
        value: Result<i32, FastRollerError>,
    ) -> Result<i32, FastRollerError> {
        value.map(|n| -n)
    }
}
