use std::collections::HashMap;

use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::{
    prelude::{Expression, Visitor},
    syntax::{AnnotationString, Augmentation, BinaryOperator, Integer},
};

use super::{try_from_big_int, DiceRoll, RollerConfig, RollerResult};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VerboseRoll {
    total: DiceRoll,
    annotated_results: HashMap<AnnotationString, DiceRoll>,
}

impl VerboseRoll {
    pub fn total(&self) -> DiceRoll {
        self.total
    }

    pub fn annotated_results(&self) -> impl Iterator<Item = (&AnnotationString, &DiceRoll)> {
        self.annotated_results.iter()
    }

    pub fn into_inner(self) -> (DiceRoll, HashMap<AnnotationString, DiceRoll>) {
        (self.total, self.annotated_results)
    }
}

pub struct VerboseRoller<R: Rng> {
    rng: R,
    config: RollerConfig,
}

impl<R: Rng> VerboseRoller<R> {
    pub fn roll(&mut self, expr: Expression) -> RollerResult<VerboseRoll> {
        self.visit(expr)
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
    fn visit_negation(&mut self, _value: RollerResult<VerboseRoll>) -> RollerResult<VerboseRoll> {
        todo!()
    }

    fn visit_dice(
        &mut self,
        _count: Option<RollerResult<VerboseRoll>>,
        _power: Option<RollerResult<VerboseRoll>>,
        _augments: SmallVec<[Augmentation; 1]>,
    ) -> RollerResult<VerboseRoll> {
        todo!()
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
        _op: BinaryOperator,
        lhs: RollerResult<VerboseRoll>,
        rhs: RollerResult<VerboseRoll>,
    ) -> RollerResult<VerboseRoll> {
        let _lhs = lhs?;
        let _rhs = rhs?;

        todo!()
    }
}
