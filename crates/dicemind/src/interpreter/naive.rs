use rand::{rngs::StdRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::{
    interpreter::RollerError,
    prelude::Expression,
    syntax::{Augmentation, BinaryOperator, Integer},
    visitor::Visitor,
};

use super::{RollerConfig, RollerResult};

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
        self.with_fail_on(1)
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

#[derive(Debug)]
pub enum NaiveValue {
    Constant(i64),
    Dice(SmallVec<[TaggedDiceRoll; 1]>),
}

impl Default for NaiveValue {
    fn default() -> Self {
        Self::Constant(0)
    }
}

impl NaiveValue {
    fn total(&self) -> i64 {
        match self {
            NaiveValue::Constant(c) => *c,
            NaiveValue::Dice(dice) => dice
                .iter()
                .fold(0, |acc, TaggedDiceRoll { value, .. }| acc + value),
        }
    }
}

pub type NaiveResult = RollerResult<NaiveValue>;

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
    pub fn roll(&mut self, expr: Expression) -> NaiveResult {
        self.visit(expr)
    }
}

impl<R: Rng> Visitor<NaiveResult> for NaiveRoller<R> {
    fn visit_dice(
        &mut self,
        quantity: NaiveResult,
        power: NaiveResult,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> NaiveResult {
        let power = power?.total();
        let quantity = quantity?.total();

        let dice_rolls = roll_many(&mut self.rng, quantity, power).collect();
        if augments.is_empty() {
            Ok(NaiveValue::Dice(dice_rolls))
        } else {
            todo!()
        }
    }

    fn visit_constant(&mut self, c: Integer) -> NaiveResult {
        Ok(NaiveValue::Constant(i64::try_from(c).unwrap()))
    }

    fn visit_binop(
        &mut self,
        op: BinaryOperator,
        lhs: NaiveResult,
        rhs: NaiveResult,
    ) -> NaiveResult {
        use BinaryOperator::*;
        use RollerError::*;

        let lhs = lhs?;
        let rhs = rhs?;

        let lhs_total = lhs.total();
        let rhs_total = rhs.total();

        let from_int = |x: i64| Ok(NaiveValue::Constant(x));

        match op {
            Equals => from_int((lhs_total == rhs_total) as i64),
            LessThan => from_int((lhs_total < rhs_total) as i64),
            GreaterThan => from_int((lhs_total > rhs_total) as i64),
            Add => from_int(lhs_total.checked_add(rhs_total).ok_or(Overflow)?),
            Subtract => from_int(lhs_total.checked_sub(rhs_total).ok_or(Overflow)?),
            Multiply => from_int(lhs_total.checked_mul(rhs_total).ok_or(Overflow)?),
            Chain => Ok(rhs),
        }
    }

    fn visit_negation(&mut self, value: NaiveResult) -> NaiveResult {
        value.map(|value| NaiveValue::Constant(-value.total()))
    }

    fn default_power(&self) -> NaiveResult {
        Ok(NaiveValue::Constant(i64::try_from(self.config.power()).unwrap()))
    }

    fn default_quantity(&self) -> NaiveResult {
        Ok(NaiveValue::Constant(i64::try_from(self.config.quantity()).unwrap()))
    }
}
