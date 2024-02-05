use num::ToPrimitive;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    minmax::MinMax, syntax::{Affix, AugmentKind, Augmentation, PositiveInteger, Selector},
};

use super::FastRollerError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct DiceRoll {
    value: i32,
    exploded: bool,
    critical_fumble: bool,
    critical_success: bool,
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
    pub fn collapse(&self) -> i32 {
        self.value
    }
}

pub fn apply_augments(
    mut rolls: Vec<DiceRoll>,
    augments: impl Iterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, FastRollerError> {
    use AugmentKind::*;
    use Augmentation::*;

    for augment in augments {
        match augment {
            Truncate { kind, affix, n } => {
                use Affix::*;

                let n = n
                    .map(|x| x.to_i32().ok_or(FastRollerError::ValueTooLarge))
                    .unwrap_or(Ok(1))?;
                let i = match rolls.binary_search_by(|x| x.value.cmp(&n)) {
                    Ok(i) => i,
                    Err(i) => i,
                } + 1;

                if i > rolls.len() {
                    return Err(FastRollerError::TruncationFailure {
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
                let n = n.to_i32().ok_or(FastRollerError::ValueTooLarge)?;

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
                // FIXME: eergh... assumed to be applied in the previous step
            }
        }
    }

    Ok(rolls)
}

pub fn verbose_roll(
    roller: &mut impl Rng,
    mut count: u32,
    power: u32,
    augments: impl IntoIterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, FastRollerError> {
    use Augmentation::*;

    // TODO: optimize the hell out of these
    let mut out = MinMax::<DiceRoll>::default();

    let augments: Vec<_> = augments.into_iter().collect();

    // .len() == 0  don't explode
    // None         explode always
    // Some(..)     explode on ..
    let check_explosion = {
        let explode_conditions: Vec<_> = augments
            .iter()
            .filter_map(|augment| {
                if let Explode { selector: n } = augment {
                    Some(n.as_ref())
                } else {
                    None
                }
            })
            .collect();

        for explode_condition in &explode_conditions {
            if let Some(Selector { relation, n }) = explode_condition {
                let a = n.cmp(&PositiveInteger::from(count));
                let b = n.cmp(&PositiveInteger::from(count * power));
                dbg!(a, b, relation);

                if PositiveInteger::from(count).cmp(n) == *relation
                    && PositiveInteger::from(count * power).cmp(n) == *relation
                {
                    // TODO: cover the case for the = operator
                    return Err(FastRollerError::InfiniteExplosion);
                }
            }
        }

        move |n: u32, power: u32| -> bool {
            let n1 = PositiveInteger::from(n);
            return !explode_conditions.is_empty()
                && explode_conditions.iter().any(|x| {
                    if let Some(Selector { n: n2, relation }) = x {
                        n1.cmp(n2) == *relation
                    } else {
                        n == power
                    }
                });
        }
    };

    let mut i = 0;
    while i < count {
        let value: i32 = roller
            .gen_range(1..=power)
            .try_into()
            .map_err(|_| FastRollerError::Overflow)?;

        let exploded = if check_explosion(value as u32, power) {
            count += 1;
            true
        } else {
            false
        };

        out.insort(DiceRoll {
            value,
            exploded,
            critical_fumble: false,
            critical_success: false,
        });

        i += 1;
    }

    apply_augments(out.into_inner(), augments.into_iter())
}
