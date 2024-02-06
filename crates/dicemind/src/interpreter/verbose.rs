use std::collections::HashMap;

use num::{CheckedAdd, CheckedMul, CheckedSub};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    prelude::{Expression, Visitor},
    syntax::{AnnotationString, Augmentation, BinaryOperator, Integer},
};

use super::{
    augmented_roll, simple_roll, try_from_big_int, try_from_positive_big_int, DiceRoll,
    RollerConfig, RollerError, RollerResult,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerboseRoll {
    sum: DiceRoll,
    annotated_results: HashMap<AnnotationString, DiceRoll>,
}

impl VerboseRoll {
    pub fn total(&self) -> DiceRoll {
        self.sum
    }

    pub fn annotated_results(&self) -> impl Iterator<Item = (&AnnotationString, &DiceRoll)> {
        self.annotated_results.iter()
    }

    pub fn into_inner(self) -> (DiceRoll, HashMap<AnnotationString, DiceRoll>) {
        (self.sum, self.annotated_results)
    }
}

pub type StandardVerboseRoller = VerboseRoller<StdRng>;

pub struct VerboseRoller<R: Rng = StdRng> {
    rng: R,
    config: RollerConfig,
}

impl<R: Rng> VerboseRoller<R> {
    pub fn roll(&mut self, expr: Expression) -> RollerResult<VerboseRoll> {
        self.visit(expr)
    }
}

impl<R: SeedableRng + Rng> Default for VerboseRoller<R> {
    fn default() -> Self {
        Self {
            config: Default::default(),
            rng: R::from_entropy(),
        }
    }
}

impl<R: SeedableRng + Rng> VerboseRoller<R> {
    pub fn new_seeded(seed: u64) -> Self {
        Self {
            config: Default::default(),
            rng: R::seed_from_u64(seed),
        }
    }
}

impl<R: Rng> Visitor<RollerResult<VerboseRoll>> for VerboseRoller<R> {
    fn visit_negation(&mut self, value: RollerResult<VerboseRoll>) -> RollerResult<VerboseRoll> {
        let VerboseRoll {
            sum: total,
            annotated_results,
        } = value?;

        let total = DiceRoll {
            value: total.value.checked_neg().ok_or(RollerError::Overflow)?,
            exploded: total.exploded,
            critical_fumble: total.critical_fumble,
            critical_success: total.critical_success,
        };

        Ok(VerboseRoll {
            sum: total,
            annotated_results,
        })
    }

    fn visit_dice(
        &mut self,
        count: Option<RollerResult<VerboseRoll>>,
        power: Option<RollerResult<VerboseRoll>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<VerboseRoll> {
        let power = power
            .map(|p| p.map(|roll| roll.total().collapse()))
            .unwrap_or(try_from_positive_big_int(self.config.power()))?;
        let count = count
            .map(|c| c.map(|roll| roll.total().collapse()))
            .unwrap_or(try_from_positive_big_int(self.config.count()))?;

        Ok(VerboseRoll {
            sum: if augments.is_empty() {
                simple_roll(&mut self.rng, count, power)?.into()
            } else {
                // Fallback to using verbose rolling
                augmented_roll(&mut self.rng, count, power, augments)?
                    .into_iter()
                    .sum::<DiceRoll>()
            },
            ..Default::default()
        })
    }

    fn visit_constant(&mut self, c: Integer) -> RollerResult<VerboseRoll> {
        let constant = try_from_big_int::<i64>(c)?;
        Ok(VerboseRoll {
            sum: DiceRoll::from(constant),
            ..Default::default()
        })
    }

    fn visit_binop(
        &mut self,
        op: BinaryOperator,
        lhs: RollerResult<VerboseRoll>,
        rhs: RollerResult<VerboseRoll>,
    ) -> RollerResult<VerboseRoll> {
        use BinaryOperator::*;
        let VerboseRoll {
            sum: t_lhs,
            annotated_results: mut annotations_lhs,
        } = lhs?;
        let VerboseRoll {
            sum: t_rhs,
            annotated_results: annotations_rhs,
        } = rhs?;

        let annotated_results = {
            annotations_lhs.extend(annotations_rhs);
            annotations_lhs
        };

        match op {
            Equals => todo!(),
            LessThan => todo!(),
            GreaterThan => todo!(),
            Add => Ok(VerboseRoll {
                sum: t_lhs.checked_add(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
            Subtract => Ok(VerboseRoll {
                sum: t_lhs.checked_sub(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
            Multiply => Ok(VerboseRoll {
                sum: t_lhs.checked_mul(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
        }
    }

    fn visit_annotated(
        &mut self,
        expr: Expression,
        annotation: AnnotationString,
    ) -> RollerResult<VerboseRoll> {
        let mut roll = self.visit(expr)?;
        roll.annotated_results.insert(annotation, roll.sum.clone());
        Ok(roll)
    }
}
