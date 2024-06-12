use std::collections::HashMap;

use num::{CheckedAdd, CheckedMul, CheckedSub};
use rand::{rngs::StdRng, Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    prelude::Expression, syntax::{AnnotationString, Augmentation, BinaryOperator, Integer}, visitor::Visitor
};

use super::{
    augmented_roll, fast_roll_many, try_from_big_int, try_from_positive_big_int, DiceRoll,
    RollerConfig, RollerError, RollerResult,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerboseRoll {
    total: DiceRoll,
    annotated_results: HashMap<AnnotationString, (Expression, DiceRoll)>,
}

impl VerboseRoll {
    pub fn total(&self) -> DiceRoll {
        self.total
    }

    pub fn annotated_results(
        &self,
    ) -> impl Iterator<Item = (&AnnotationString, &(Expression, DiceRoll))> {
        self.annotated_results.iter()
    }

    pub fn into_inner(self) -> (DiceRoll, HashMap<AnnotationString, (Expression, DiceRoll)>) {
        (self.total, self.annotated_results)
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

    pub fn new() -> Self {
        Self::default()
    }
}

impl<R: Rng> Visitor<RollerResult<VerboseRoll>> for VerboseRoller<R> {
    fn visit_negation(&mut self, value: RollerResult<VerboseRoll>) -> RollerResult<VerboseRoll> {
        let VerboseRoll {
            total,
            annotated_results,
        } = value?;

        let total = DiceRoll {
            value: total.value.checked_neg().ok_or(RollerError::Overflow)?,
            ..total
        };

        Ok(VerboseRoll {
            total,
            annotated_results,
        })
    }

    fn visit_dice_OLD(
        &mut self,
        quantity: Option<RollerResult<VerboseRoll>>,
        power: Option<RollerResult<VerboseRoll>>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<VerboseRoll> {
        let power = power
            .map(|p| p.map(|roll| roll.total().value()))
            .unwrap_or(try_from_positive_big_int(self.config.power()))?;
        let quantity = quantity
            .map(|c| c.map(|roll| roll.total().value()))
            .unwrap_or(try_from_positive_big_int(self.config.quantity()))?;

        Ok(VerboseRoll {
            total: if augments.is_empty() {
                fast_roll_many(&mut self.rng, quantity, power)?.into()
            } else {
                // Fallback to using verbose rolling
                augmented_roll(&mut self.rng, quantity, power, augments)?
                    .into_iter()
                    .sum::<DiceRoll>()
            },
            ..Default::default()
        })
    }

    fn visit_constant(&mut self, c: Integer) -> RollerResult<VerboseRoll> {
        let constant = try_from_big_int::<i64>(c)?;
        Ok(VerboseRoll {
            total: DiceRoll::from(constant),
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
            total: t_lhs,
            annotated_results: mut annotations_lhs,
        } = lhs?;
        let VerboseRoll {
            total: t_rhs,
            annotated_results: annotations_rhs,
        } = rhs?;

        let annotated_results = {
            annotations_lhs.extend(annotations_rhs.clone());
            annotations_lhs
        };

        match op {
            Equals => todo!(),
            LessThan => todo!(),
            GreaterThan => todo!(),
            Add => Ok(VerboseRoll {
                total: t_lhs.checked_add(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
            Subtract => Ok(VerboseRoll {
                total: t_lhs.checked_sub(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
            Multiply => Ok(VerboseRoll {
                total: t_lhs.checked_mul(&t_rhs).ok_or(RollerError::Overflow)?,
                annotated_results,
            }),
            Chain => Ok(VerboseRoll {
                total: t_rhs,
                annotated_results,
            }),
        }
    }

    fn visit_annotated(
        &mut self,
        expr: Expression,
        annotation: AnnotationString,
    ) -> RollerResult<VerboseRoll> {
        let mut roll = self.visit(expr.clone())?;
        roll.annotated_results
            .insert(annotation, (expr, roll.total.clone()));
        Ok(roll)
    }
    
    fn visit_dice(
        &mut self,
        quantity: RollerResult<VerboseRoll>,
        power: RollerResult<VerboseRoll>,
        augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<VerboseRoll> {
        todo!()
    }
}
