use num::ToPrimitive;
use rand::Rng;

use super::{RollerError, RollerResult};
use crate::{
    minmax::MinMax,
    syntax::{Affix, AugmentKind, Augmentation, Selector},
};

use std::ops::{Add, AddAssign, Mul};

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Clone, Copy, Deserialize, PartialEq, Eq, Hash)]
pub struct DiceRoll {
    value: i64,
    exploded: bool,
    critical_fumble: bool,
    critical_success: bool,
}

impl From<i64> for DiceRoll {
    fn from(value: i64) -> Self {
        Self {
            value,
            ..Default::default()
        }
    }
}

impl Add for DiceRoll {
    type Output = DiceRoll;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            value: self.value + rhs.value,
            exploded: self.exploded || rhs.exploded,
            critical_fumble: self.critical_fumble || rhs.critical_fumble,
            critical_success: self.critical_success || rhs.critical_success,
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
    pub fn collapse(&self) -> i64 {
        self.value
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

                let n = n
                    .map(|x| {
                        x.to_i64()
                            .ok_or(RollerError::ValueTooLarge { value: x.into() })
                    })
                    .unwrap_or(Ok(1))?;
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
                let n = n
                    .to_i64()
                    .ok_or(RollerError::ValueTooLarge { value: n.into() })?;

                let predicate: Box<dyn Fn(&mut DiceRoll) -> bool> = match (kind, relation) {
                    (Drop, Less) => Box::new(|x| x.collapse() < n),
                    (Drop, Equal) => Box::new(|x| x.collapse() == n),
                    (Drop, Greater) => Box::new(|x| x.collapse() > n),
                    (Keep, Less) => Box::new(|x| x.collapse() >= n),
                    (Keep, Equal) => Box::new(|x| x.collapse() != n),
                    (Keep, Greater) => Box::new(|x| x.collapse() <= n),
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
    roller: &mut impl Rng,
    count: i64,
    power: i64,
    augments: impl IntoIterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, RollerError> {
    // TODO: optimize the hell out of these
    let mut out = MinMax::<DiceRoll>::default();

    let augments: Vec<_> = augments.into_iter().collect();

    let mut i = 0;
    while i < count {
        let value: i64 = roller
            .gen_range(1..=power)
            .try_into()
            .map_err(|_| RollerError::Overflow)?;

        out.insort(DiceRoll {
            value,
            exploded: false,
            critical_fumble: false,
            critical_success: false,
        });

        i += 1;
    }

    apply_augments(out.into_inner(), augments.into_iter())
}

pub fn simple_roll(roller: &mut impl Rng, count: i64, power: i64) -> RollerResult<i64> {
    use RollerError::*;

    if count == 0 || power == 0 {
        return Ok(0);
    }

    let mut sum = 0i64;

    for _ in 0..count.abs() {
        let n = roller.gen_range(1..=power.abs());
        sum = sum.checked_add(n).ok_or(Overflow)?;
    }

    Ok(sum.mul(count.signum() * power.signum()))
}
