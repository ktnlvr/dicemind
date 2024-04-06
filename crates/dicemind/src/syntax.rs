use std::{
    cmp::Ordering,
    fmt::{Display, Write},
};

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use smol_str::SmolStr;

pub type Integer = num::bigint::BigInt;
pub type PositiveInteger = num::bigint::BigUint;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, Serialize, Deserialize)]
pub enum BinaryOperator {
    Chain,
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
            Chain => 0,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Expression {
    Dice {
        quantity: Option<Box<Expression>>,
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

impl Expression {
    pub fn is_trivial(&self) -> bool {
        use Expression::*;

        match self {
            Constant(_) => true,
            Dice { .. } => true,
            Binop { .. } => false,
            Subexpression(_) => true,
            UnaryNegation(_) => false,
            Annotated { .. } => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Copy, Deserialize)]
pub enum AugmentKind {
    Drop,
    Keep,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub enum Affix {
    High,
    Low,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Ordering")]
enum SerdeOrdering {
    Less = -1,
    Equal = 0,
    Greater = 1,
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

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use BinaryOperator::*;
        use Expression::*;

        match self {
            Dice {
                quantity,
                power,
                augmentations,
            } => {
                if let Some(n) = quantity {
                    if n.is_trivial() {
                        f.write_fmt(format_args!("{}", n))?;
                    } else {
                        f.write_char('(')?;
                        f.write_fmt(format_args!("{}", n))?;
                        f.write_char(')')?;
                    }
                }
                f.write_char('d')?;

                if let Some(p) = power {
                    f.write_fmt(format_args!("{}", p))?;
                }

                if augmentations.len() != 0 {
                    todo!()
                }

                Ok(())
            }
            Binop { operator, lhs, rhs } => {
                match lhs.as_ref() {
                    Subexpression(box Binop {
                        operator: child_operator,
                        ..
                    }) if child_operator < operator => f.write_fmt(format_args!("({lhs}) ")),
                    _ if !lhs.is_trivial() => f.write_fmt(format_args!("({lhs}) ")),
                    _ => f.write_fmt(format_args!("{lhs} ")),
                }?;

                match operator {
                    Equals => f.write_char('='),
                    LessThan => f.write_char('<'),
                    GreaterThan => f.write_char('>'),
                    Add => f.write_char('+'),
                    Subtract => f.write_char('-'),
                    Multiply => f.write_char('*'),
                    Chain => f.write_char(','),
                }?;

                match rhs.as_ref() {
                    Subexpression(box Binop {
                        operator: child_operator,
                        ..
                    }) if child_operator < operator => f.write_fmt(format_args!(" ({rhs})")),
                    _ if !rhs.is_trivial() => f.write_fmt(format_args!(" ({rhs})")),
                    _ => f.write_fmt(format_args!(" {rhs}")),
                }
            }
            Constant(c) => f.write_fmt(format_args!("{c}")),
            Annotated {
                expression,
                annotation,
            } => f.write_fmt(format_args!("{expression} [{annotation}]")),
            Subexpression(expr) => f.write_fmt(format_args!("{expr}")),
            UnaryNegation(expr) => f.write_fmt(format_args!("-{expr}")),
        }
    }
}
