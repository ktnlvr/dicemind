use std::iter::Sum;

use rand::{rngs::StdRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::{
    interpreter::RollerError,
    prelude::Expression,
    syntax::{Augmentation, BinaryOperator, Integer},
    visitor::Visitor,
};

use super::{try_from_big_int, try_from_positive_big_int, RollerConfig, RollerResult};

fn roll_one(rng: &mut impl Rng, power: i64) -> TaggedDiceRoll {
    if power == 0 {
        return TaggedDiceRoll::zero();
    }

    TaggedDiceRoll::from(rng.gen_range(1..=power.abs()) * power.signum())
        .with_fail_on_1()
        .with_success_on(power)
}

fn roll_many(
    rng: &mut impl Rng,
    quantity: i64,
    power: i64,
) -> impl Iterator<Item = TaggedDiceRoll> {
    // TODO: sanity check this cast
    let mut dice = Vec::with_capacity(power.abs() as usize);

    if quantity == 0 || power == 0 {
        return dice.into_iter();
    }

    for _ in 0..quantity.abs() {
        let rolled = roll_one(rng, power);
        dice.push(TaggedDiceRoll {
            value: rolled.value * quantity.signum(),
            ..rolled
        });
    }

    dice.into_iter()
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct DiceRollTag: u32 {
        const FAIL = 1 << 0;
        const SUCCESS = 1 << 1;
        const EXPLOSION = 1 << 2;
        const DISCARDED = 1 << 3;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TaggedDiceRoll {
    pub tag: DiceRollTag,
    pub value: i64,
}

impl From<i64> for TaggedDiceRoll {
    fn from(n: i64) -> Self {
        Self {
            tag: DiceRollTag::empty(),
            value: n,
        }
    }
}

impl TaggedDiceRoll {
    fn zero() -> Self {
        Self::from(0)
    }

    fn with_success_on(mut self, succ: i64) -> Self {
        if self.value == succ {
            self.tag |= DiceRollTag::SUCCESS;
        }

        self
    }

    fn with_fail_on_1(mut self) -> Self {
        self.with_success_on(1)
    }

    fn with_fail_on(mut self, fail: i64) -> Self {
        if self.value == fail {
            self.tag |= DiceRollTag::FAIL;
        }

        self
    }

    fn with_explosion_on(mut self, explode: i64) -> Self {
        if self.value == explode {
            self.tag |= DiceRollTag::EXPLOSION;
        }

        self
    }

    fn discard(&mut self) {
        self.tag |= DiceRollTag::DISCARDED;
    }
}

pub type StandardNaiveRoller = NaiveRoller;

#[derive(Debug, Clone)]
pub struct NaiveRoller<R: Rng = StdRng> {
    config: RollerConfig,
    rng: R,
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

impl<R: Rng> NaiveRoller<R> {
    pub fn roll(&mut self, expr: Expression) -> RollerResult<Vec<TaggedDiceRoll>> {
        self.visit(expr)
    }
}

impl<R: Rng> Visitor<RollerResult<Vec<TaggedDiceRoll>>> for NaiveRoller<R> {
    fn visit_dice_UPDATED(
        &mut self,
        quantity: RollerResult<Vec<TaggedDiceRoll>>,
        power: RollerResult<Vec<TaggedDiceRoll>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<Vec<TaggedDiceRoll>> {
        let power = power?.into_iter().fold(0, |acc, roll| acc + roll.value);
        let quantity = quantity?.into_iter().fold(0, |acc, roll| acc + roll.value);

        if augments.is_empty() {
            Ok(roll_many(&mut self.rng, quantity, power).collect())
        } else {
            todo!()
        }
    }

    fn visit_constant(&mut self, c: Integer) -> RollerResult<Vec<TaggedDiceRoll>> {
        Ok(vec![TaggedDiceRoll::from(i64::try_from(c).unwrap())])
    }

    fn visit_binop(
        &mut self,
        op: BinaryOperator,
        lhs: RollerResult<Vec<TaggedDiceRoll>>,
        rhs: RollerResult<Vec<TaggedDiceRoll>>,
    ) -> RollerResult<Vec<TaggedDiceRoll>> {
        use BinaryOperator::*;
        use RollerError::*;

        if op == Chain {
            return rhs;
        }

        let lhs = lhs?.into_iter().fold(0, |acc, roll| acc + roll.value);
        let rhs = rhs?.into_iter().fold(0, |acc, roll| acc + roll.value);

        let from_int = |x: i64| Ok(vec![TaggedDiceRoll::from(x)]);

        match op {
            Equals => from_int((lhs == rhs) as i64),
            LessThan => from_int((lhs < rhs) as i64),
            GreaterThan => from_int((lhs > rhs) as i64),
            Add => from_int(lhs.checked_add(rhs).ok_or(Overflow)?),
            Subtract => from_int(lhs.checked_sub(rhs).ok_or(Overflow)?),
            Multiply => from_int(lhs.checked_mul(rhs).ok_or(Overflow)?),
            Chain => unreachable!(),
        }
    }

    fn visit_negation(
        &mut self,
        value: RollerResult<Vec<TaggedDiceRoll>>,
    ) -> RollerResult<Vec<TaggedDiceRoll>> {
        todo!()
    }
}

/*
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
        quantity: Option<RollerResult<i64>>,
        power: Option<RollerResult<i64>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<i64> {
        let power = power.unwrap_or(try_from_positive_big_int(self.config.power()))?;
        let quantity = quantity.unwrap_or(try_from_positive_big_int(self.config.quantity()))?;

        if augments.is_empty() {
            fast_roll_many(&mut self.rng, quantity, power)
        } else {
            // Fallback to using verbose rolling
            Ok(augmented_roll(&mut self.rng, quantity, power, augments)?
                .into_iter()
                .map(|roll| roll.value())
                .sum())
        }
    }

    fn visit_constant(&mut self, c: Integer) -> RollerResult<i64> {
        try_from_big_int(c)
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
            Chain => rhs,
        }
    }

    fn visit_negation(&mut self, value: RollerResult<i64>) -> RollerResult<i64> {
        value.and_then(|n| n.checked_mul(-1).ok_or(RollerError::Overflow))
    }
}
*/
