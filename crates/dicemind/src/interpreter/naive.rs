use std::{collections::HashSet, hash::RandomState};

use num::BigUint;
use rand::{rngs::StdRng, Rng, SeedableRng};
use smallvec::SmallVec;

use crate::{
    interpreter::RollerError,
    prelude::Expression,
    syntax::{Affix, Augmentation, BinaryOperator, Integer, Selector, SelectorOp},
    visitor::Visitor,
};

use super::{RollerOptions, RollerResult};

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

pub fn should_selector_discard(n: i64, selector: Selector, op: SelectorOp) -> bool {
    let matches = selector.matches(n);
    let keep = op == SelectorOp::Keep;

    // TODO: make prettier and more readable
    if keep {
        if !matches {
            return true;
        }
    } else {
        if matches {
            return true;
        }
    }

    return false;
}

fn optional_big_uint_to_usize_or_1(n: Option<BigUint>) -> usize {
    n.map(|n| usize::try_from(n).ok())
        .flatten()
        .unwrap_or(1usize)
}

fn augment(
    mut dice: Vec<TaggedDiceRoll>,
    augments: impl Iterator<Item = Augmentation>,
    power: i64,
) -> RollerResult<Vec<TaggedDiceRoll>> {
    for augment in augments {
        match augment {
            Augmentation::Truncate { op, affix, n } => {
                let n = optional_big_uint_to_usize_or_1(n);

                let mut indices_high_to_low = Vec::<usize>::with_capacity(n);
                for (i, _) in dice.iter().enumerate() {
                    let (Ok(idx) | Err(idx)) = indices_high_to_low.binary_search_by(|j| dice[*j].cmp(&dice[i]));
                    indices_high_to_low.insert(idx, i);
                }

                use SelectorOp::*;
                use Affix::*;

                // Keeping high is the same as dropping low
                // Keeping low is the same as dropping high
                match (op, affix) {
                    (Keep, Low) | (Drop, High) => {},
                    (Keep, High) | (Drop, Low) => indices_high_to_low.reverse(),
                }
                let keep_indices = HashSet::<_, RandomState>::from_iter(indices_high_to_low.into_iter().take(n));

                for (i, d) in dice.iter_mut().enumerate() {
                    if !keep_indices.contains(&i) {
                        d.discard();
                    }
                }
            }
            Augmentation::Filter { op, selector } => {
                for d in &mut dice {
                    if should_selector_discard(d.value, selector.clone(), op) {
                        d.discard();
                    }
                }
            }
            Augmentation::Emphasis { n } => {
                let n = optional_big_uint_to_usize_or_1(n);
            }
            Augmentation::Explode { selector } => {}
        }
    }

    Ok(dice)
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct DiceRollTag: u32 {
        const FAIL = 1 << 0;
        const SUCCESS = 1 << 1;
        const EXPLODES = 1 << 2;
        const EXPLOSION = 1 << 3;
        const DISCARDED = 1 << 4;
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TaggedDiceRoll {
    pub tag: DiceRollTag,
    pub value: i64,
}

impl Eq for TaggedDiceRoll {}

impl PartialEq for TaggedDiceRoll {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for TaggedDiceRoll {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl Ord for TaggedDiceRoll {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
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

    fn with_exploding_on(mut self, explode: i64) -> Self {
        if self.value == explode {
            self.tag |= DiceRollTag::EXPLODES;
        }

        self
    }

    fn exploded(&mut self) {
        self.tag |= DiceRollTag::EXPLOSION;
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
    options: RollerOptions,
    rng: R,
}

impl<R: SeedableRng + Rng> NaiveRoller<R> {
    pub fn new_seeded(seed: u64) -> Self {
        Self {
            options: Default::default(),
            rng: R::seed_from_u64(seed),
        }
    }
}

impl<R: SeedableRng + Rng> Default for NaiveRoller<R> {
    fn default() -> Self {
        Self {
            options: Default::default(),
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
            augment(dice_rolls.into_vec(), augments.into_iter(), power)
                .map(|dice| NaiveValue::Dice(dice.into_iter().collect()))
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
        Ok(NaiveValue::Constant(
            i64::try_from(self.options.power()).unwrap(),
        ))
    }

    fn default_quantity(&self) -> NaiveResult {
        Ok(NaiveValue::Constant(
            i64::try_from(self.options.quantity()).unwrap(),
        ))
    }
}
