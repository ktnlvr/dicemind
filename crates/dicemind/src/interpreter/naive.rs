use num::ToPrimitive;
use rand::{rngs::StdRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::interpreter::{augmented_roll, simple_roll};

use crate::syntax::{Augmentation, BinaryOperator, Expression, Integer};
use crate::visitor::Visitor;

use super::{RollerConfig, RollerError, RollerResult};

pub type StandardFastRoller = NaiveRoller<StdRng>;

#[derive(Debug, Clone)]
pub struct NaiveRoller<R: Rng = StdRng> {
    config: RollerConfig,
    rng: R,
}

impl<R: Rng> NaiveRoller<R> {
    pub fn roll(&mut self, expr: Expression) -> RollerResult<i64> {
        self.visit(expr)
    }
}

impl<R: SeedableRng + Rng> NaiveRoller<R> {
    pub fn new_seeded(seed: u64) -> Self {
        Self {
            config: Default::default(),
            rng: R::seed_from_u64(seed),
        }
    }
}

impl<R: SeedableRng + Rng> Default for NaiveRoller<R> {
    fn default() -> Self {
        Self {
            config: Default::default(),
            rng: R::from_entropy(),
        }
    }
}

impl<R: Rng> Visitor<RollerResult<i64>> for NaiveRoller<R> {
    fn visit_dice(
        &mut self,
        count: Option<RollerResult<i64>>,
        power: Option<RollerResult<i64>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<i64> {
        let count = count
            .unwrap_or(i64::try_from(self.config.count()).map_err(|_| RollerError::Overflow))?;
        let power = power
            .unwrap_or(i64::try_from(self.config.power()).map_err(|_| RollerError::Overflow))?;

        if augments.is_empty() {
            simple_roll(&mut self.rng, count, power)
        } else {
            // Fallback to using verbose rolling
            Ok(augmented_roll(&mut self.rng, count, power, augments)?
                .into_iter()
                .map(|roll| roll.collapse())
                .sum())
        }
    }

    fn visit_constant(&mut self, c: Integer) -> RollerResult<i64> {
        c.to_i64().ok_or(RollerError::ValueTooLarge { value: c })
    }

    fn visit_binop(
        &mut self,
        op: BinaryOperator,
        lhs: RollerResult<i64>,
        rhs: RollerResult<i64>,
    ) -> RollerResult<i64> {
        use BinaryOperator::*;
        use RollerError::*;

        match op {
            Equals => Ok((lhs? == rhs?) as i64),
            LessThan => Ok((lhs? < rhs?) as i64),
            GreaterThan => Ok((lhs? > rhs?) as i64),
            Add => lhs?.checked_add(rhs?).ok_or(Overflow),
            Subtract => lhs?.checked_sub(rhs?).ok_or(Overflow),
            Multiply => lhs?.checked_mul(rhs?).ok_or(Overflow),
        }
    }

    fn visit_negation(&mut self, value: RollerResult<i64>) -> RollerResult<i64> {
        value.and_then(|n| n.checked_mul(-1).ok_or(RollerError::Overflow))
    }
}
