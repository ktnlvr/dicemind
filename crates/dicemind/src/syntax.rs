use std::cmp::Ordering;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smol_str::SmolStr;

pub type Integer = num::bigint::BigInt;
pub type PositiveInteger = num::bigint::BigUint;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    Equals,
    LessThan,
    GreaterThan,
    Add,
    Subtract,
    Multiply,
}

impl From<BinaryOperator> for u8 {
    fn from(val: BinaryOperator) -> Self {
        use BinaryOperator::*;

        match val {
            Multiply => 3,
            Add | Subtract => 2,
            Equals | LessThan | GreaterThan => 1,
        }
    }
}

impl Ord for BinaryOperator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let l: u8 = (*self).into();
        let r: u8 = (*other).into();

        l.cmp(&r)
    }
}

impl PartialOrd for BinaryOperator {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub type AnnotationString = SmolStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Dice {
        count: Option<Box<Expression>>,
        power: Option<Box<Expression>>,
        augmentations: SmallVec<[Augmentation; 1]>,
    },
    Binop {
        operator: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Constant(Integer),
    Annotated {
        expression: Box<Expression>,
        annotation: AnnotationString,
    },
    Subexpression(Box<Expression>),
    UnaryNegation(Box<Expression>),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Copy, Deserialize)]
pub enum AugmentKind {
    Drop,
    Keep,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Ordering")]
enum SerdeOrdering {
    Less = -1,
    Equal = 0,
    Greater = 1,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub enum Affix {
    High,
    Low,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub struct Selector {
    #[serde(with = "SerdeOrdering")]
    pub relation: Ordering,
    pub n: PositiveInteger,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub enum Augmentation {
    // kh4 kl2
    Truncate {
        kind: AugmentKind,
        affix: Affix,
        n: Option<PositiveInteger>,
    },
    // d<2 k=3
    Filter {
        kind: AugmentKind,
        selector: Selector,
    },
    // e
    Emphasis {
        // How many dice to emphasise
        n: Option<PositiveInteger>,
    },
    // !
    // TODO: allow exploding n-times on different values
    Explode {
        // On what values to explode
        selector: Option<Selector>,
    },
}