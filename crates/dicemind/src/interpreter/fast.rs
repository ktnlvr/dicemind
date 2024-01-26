use rand::{thread_rng, Rng};
use smallvec::SmallVec;
use thiserror::Error;

use crate::parser::*;
use crate::visitor::Visitor;

use super::RollerConfig;

#[derive(Debug, Default)]
pub struct FastRoller {
    config: RollerConfig,
}

#[derive(Debug, Error)]
pub enum FastRollerError {
    #[error("Value too large and can't fit inside 2^31 - 1")]
    ValueTooLarge,
    #[error("The value has overflown, the result was too large")]
    Overflow,
}

impl FastRoller {
    pub fn roll(&mut self, expr: Expression) -> Result<i32, FastRollerError> {
        self.visit(expr)
    }
}

impl Visitor<Result<i32, FastRollerError>> for FastRoller {
    fn visit_dice(
        &mut self,
        count: Option<Result<i32, FastRollerError>>,
        power: Option<Result<i32, FastRollerError>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> Result<i32, FastRollerError> {
        use FastRollerError::*;

        let count = count
            .unwrap_or(i32::try_from(self.config.count()).map_err(|_| FastRollerError::Overflow));
        let power = power
            .unwrap_or(i32::try_from(self.config.power()).map_err(|_| FastRollerError::Overflow))?;

        let (sign, count) = count.map(|x| (x.signum(), x.abs()))?;

        if count == 0 || power == 0 {
            return Ok(0);
        }

        let mut rng = thread_rng();
        let mut sum = 0i32;

        if augments.is_empty() {
            for _ in 0..count {
                let n = rng.gen_range(1..=power);
                sum = sum
                    .checked_add(n.try_into().map_err(|_| ValueTooLarge)?)
                    .ok_or(Overflow)?;
            }
        } else {
            todo!()
        }

        sum.checked_mul(sign).ok_or(Overflow)
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
        value
            .and_then(|n| n.checked_mul(-1).ok_or(FastRollerError::Overflow))
    }
}
