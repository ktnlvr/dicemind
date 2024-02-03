use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::{
    minmax::MinMax,
    parser::{Augmentation, PositiveInteger, Selector},
};

use super::FastRollerError;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd)]
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

impl DiceRoll {
    pub fn collapse(&self) -> i32 {
        self.value
    }
}

pub fn verbose_roll(
    roller: &mut impl Rng,
    mut count: u32,
    power: u32,
    augments: impl IntoIterator<Item = Augmentation>,
) -> Result<Vec<DiceRoll>, FastRollerError> {
    use Augmentation::*;
    let mut out = MinMax::<DiceRoll>::default();
    let augments: Vec<_> = augments.into_iter().collect();

    // .len() == 0  don't explode
    // None         explode always
    // Some(..)     explode on ..
    let check_explosion = {
        let explode_conditions: Vec<_> = augments
            .iter()
            .filter_map(|augment| {
                if let Explode { n } = augment {
                    Some(n.as_ref())
                } else {
                    None
                }
            })
            .collect();

        move |n: u32, power: u32| -> bool {
            let n1 = PositiveInteger::from(n);
            return explode_conditions.len() != 0
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

    Ok(out.into_inner())
}
