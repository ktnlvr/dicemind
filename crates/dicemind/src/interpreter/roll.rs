use num::{CheckedAdd, CheckedMul, CheckedSub};
use rand::Rng;

use super::{RollerError, RollerResult};
use crate::syntax::{Affix, AugmentKind, Augmentation, Integer, PositiveInteger, Selector};

use std::{
    iter::Sum,
    ops::{Add, AddAssign, Mul, Sub},
};

use serde::{Deserialize, Serialize};

use std::fmt::Debug;

pub(crate) fn try_from_positive_big_int<T: TryFrom<PositiveInteger>>(
    value: PositiveInteger,
) -> RollerResult<T> {
    T::try_from(value.clone()).map_err(|_| RollerError::ValueTooLarge {
        value: value.into(),
    })
}

pub(crate) fn try_from_big_int<T: TryFrom<Integer>>(value: Integer) -> RollerResult<T> {
    T::try_from(value.clone()).map_err(|_| RollerError::ValueTooLarge {
        value: value.into(),
    })
}

pub fn fast_roll_many(rng: &mut impl Rng, quantity: i64, power: i64) -> RollerResult<i64> {
    use RollerError::*;

    // This would happen with the following logic anyway
    // Just an early return
    if quantity == 0 || power == 0 {
        return Ok(0);
    }

    let mut sum = 0i64;

    for _ in 0..quantity.abs() {
        let n = rng.gen_range(1..=power.abs());
        sum = sum.checked_add(n).ok_or(Overflow)?;
    }

    // TODO: Check these
    Ok(sum.mul(quantity.signum() * power.signum()))
}

#[derive(Debug, Default, Serialize, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub struct DiceRoll {
    pub value: i64,
    pub exploded: bool,
    pub critical_fumble: bool,
    pub critical_success: bool,
}

impl From<i64> for DiceRoll {
    fn from(value: i64) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl AddAssign for DiceRoll {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
        self.exploded |= rhs.exploded;
        self.critical_fumble |= rhs.critical_fumble;
        self.critical_success |= rhs.critical_fumble;
    }
}

impl Ord for DiceRoll {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // TODO: bias towards keeping crits by imposing a suborder?
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for DiceRoll {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl DiceRoll {
    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn to_markdown_string(&self) -> String {
        if self.critical_fumble || self.critical_success {
            format!("**{}**", self.value)
        } else {
            self.value.to_string()
        }
    }
}

macro_rules! impl_dice_roll_arithmetic {
    ($t: tt, $name: ident) => {
        impl $t for DiceRoll {
            type Output = Self;

            fn $name(self, rhs: Self) -> Self {
                Self {
                    value: $t::$name(self.value, rhs.value),
                    exploded: self.exploded || rhs.exploded,
                    critical_fumble: self.critical_fumble || rhs.critical_fumble,
                    critical_success: self.critical_success || rhs.critical_success,
                }
            }
        }
    };
}

macro_rules! impl_dice_roll_arithmetic_checked {
    ($t: tt, $name: ident) => {
        impl $t for DiceRoll {
            fn $name(&self, rhs: &Self) -> Option<Self> {
                Some(Self {
                    value: $t::$name(&self.value, &rhs.value)?,
                    exploded: self.exploded || rhs.exploded,
                    critical_fumble: self.critical_fumble || rhs.critical_fumble,
                    critical_success: self.critical_success || rhs.critical_success,
                })
            }
        }
    };
}

impl_dice_roll_arithmetic!(Add, add);
impl_dice_roll_arithmetic!(Sub, sub);
impl_dice_roll_arithmetic!(Mul, mul);

impl_dice_roll_arithmetic_checked!(CheckedAdd, checked_add);
impl_dice_roll_arithmetic_checked!(CheckedSub, checked_sub);
impl_dice_roll_arithmetic_checked!(CheckedMul, checked_mul);

impl Sum for DiceRoll {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(DiceRoll::default(), |acc, n| acc + n)
    }
}

fn apply_augments(
    mut rolls: Vec<DiceRoll>,
    augments: impl Iterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, RollerError> {
    use AugmentKind::*;
    use Augmentation::*;

    for augment in augments {
        match augment {
            Truncate { kind, affix, n } => {
                use Affix::*;

                let n = n.map(|x| try_from_positive_big_int(x)).unwrap_or(Ok(1))?;
                let i = match rolls.binary_search_by(|x| x.value.cmp(&n)) {
                    Ok(i) => i,
                    Err(i) => i,
                } + 1;

                if i > rolls.len() {
                    return Err(RollerError::TruncationFailure {
                        rolled: todo!(),
                        removed: todo!(),
                    });
                }

                match (kind, affix) {
                    // TODO: test this?
                    (Drop, High) => rolls.drain(i..),
                    (Drop, Low) => rolls.drain(..i),
                    (Keep, High) => rolls.drain(..i),
                    (Keep, Low) => rolls.drain(i..),
                };
            }
            Filter { kind, selector } => {
                use std::cmp::Ordering::*;
                let Selector { relation, n } = selector;
                let n = try_from_positive_big_int(n)?;

                let predicate: Box<dyn Fn(&mut DiceRoll) -> bool> = match (kind, relation) {
                    (Drop, Less) => Box::new(|x| x.value() < n),
                    (Drop, Equal) => Box::new(|x| x.value() == n),
                    (Drop, Greater) => Box::new(|x| x.value() > n),
                    (Keep, Less) => Box::new(|x| x.value() >= n),
                    (Keep, Equal) => Box::new(|x| x.value() != n),
                    (Keep, Greater) => Box::new(|x| x.value() <= n),
                };

                let _: Vec<_> = rolls.extract_if(predicate).collect();
            }
            Emphasis { n: _ } => todo!(),
            Explode { selector: _n } => {
                todo!()
            }
        }
    }

    Ok(rolls)
}

pub fn augmented_roll(
    rng: &mut impl Rng,
    quantity: i64,
    power: i64,
    augments: impl IntoIterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, RollerError> {
    let mut out = Vec::<DiceRoll>::new();

    let augments: Vec<_> = augments.into_iter().collect();

    let mut i = 0;
    while i < quantity {
        let value: i64 = rng
            .gen_range(1..=power)
            .try_into()
            .map_err(|_| RollerError::Overflow)?;

        out.push(DiceRoll {
            value,
            exploded: false,
            critical_fumble: false,
            critical_success: false,
        });

        i += 1;
    }

    apply_augments(out, augments.into_iter())
}
