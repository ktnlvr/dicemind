use std::cmp::Ordering;

use num::Zero;
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;
use thiserror::Error;

pub type Integer = num::bigint::BigInt;
pub type PositiveInteger = num::bigint::BigUint;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Hash, Serialize, Deserialize)]
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
            Equals | LessThan | GreaterThan => 3,
            Multiply => 2,
            Add | Subtract => 1,
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
    Subexpression(Box<Expression>),
    UnaryNegation(Box<Expression>),
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Copy, Deserialize)]
pub enum DependentAugmentKind {
    Drop,
    Keep,
    Reroll,
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "Ordering")]
enum SerdeOrdering {
    Less = -1,
    Equal = 0,
    Greater = 1,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq, Hash, Deserialize)]
pub enum Augmentation {
    Dependent {
        kind: DependentAugmentKind,
        #[serde(with = "SerdeOrdering")]
        relation: Ordering,
        n: Option<PositiveInteger>,
    },
    Emphasis {
        e: Option<PositiveInteger>,
    },
    Explode {
        n: Option<PositiveInteger>,
    },
}

#[derive(Debug, Error, Clone, Serialize, Copy, Hash, PartialEq, Eq)]
pub enum ParsingError {
    #[error("The string did not contain any expressions")]
    EmptyExpression,
    #[error("Unbalanced left parenthis")]
    UnbalancedLeftParen,
    #[error("Unbalanced right parenthis")]
    UnbalancedRightParen,
    #[error("Undefined symbol `{char}`")]
    UndefinedSymbol { char: char },
    #[error("No operands")]
    NoOperands { operator: BinaryOperator },
    #[error("Missing operator between operands")]
    MissingOperator,
}

fn parse_augments(mut chars: &[char]) -> Option<(impl Iterator<Item = Augmentation>, &[char])> {
    let mut out: Vec<Augmentation> = vec![];
    while !chars.is_empty() {
        if chars[0] == 'e' {
            let e = if let Some((n, rest)) = parse_number(&chars[1..]) {
                chars = rest;
                Some(n)
            } else {
                chars = &chars[1..];
                None
            };

            out.push(Augmentation::Emphasis { e });
        } else if chars[0] == '!' {
            let n = if let Some((n, rest)) = parse_number(&chars[1..]) {
                chars = rest;
                Some(n)
            } else {
                chars = &chars[1..];
                None
            };

            out.push(Augmentation::Explode { n });
        } else {
            break;
        }
    }

    if !out.is_empty() {
        Some((out.into_iter(), chars))
    } else {
        None
    }
}

fn parse_number(chars: &[char]) -> Option<(PositiveInteger, &[char])> {
    if chars.is_empty() {
        return None;
    }

    if !chars[0].is_ascii_digit() {
        return None;
    }

    let digits: Vec<u8> = chars
        .iter()
        .cloned()
        .map_while(|c| c.to_digit(10).map(|n| n as u8))
        .collect();
    let len = digits.len();

    let mut number = PositiveInteger::zero();
    for (i, d) in digits.into_iter().enumerate() {
        number += d * PositiveInteger::from(10u32).pow((len - i - 1) as u32);
    }

    Some((number, &chars[len..]))
}

fn parse_dice(mut chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    let count = parse_term(chars)?.map(|(count, rest)| {
        chars = rest;
        Box::new(count)
    });

    if chars.is_empty() {
        return Ok(None);
    }

    if chars[0] == 'd' {
        let power = if let Some((expr, rest)) = parse_term(&chars[1..])? {
            chars = rest;
            Some(Box::new(expr))
        } else if chars.len() >= 2 && chars[1] == '%' {
            chars = &chars[2..];
            Some(Box::new(Expression::Constant(100.into())))
        } else {
            chars = &chars[1..];
            None
        };

        let augments: Vec<_> = if let Some((augs, rest)) = parse_augments(chars) {
            chars = rest;
            augs.collect()
        } else {
            vec![]
        };

        return Ok(Some((
            Expression::Dice {
                count,
                power,
                augmentations: augments.into_iter().collect(),
            },
            chars,
        )));
    }

    Ok(None)
}

fn parse_operator(char: char) -> Option<BinaryOperator> {
    use BinaryOperator::*;

    match char {
        '+' => Some(Add),
        '-' => Some(Subtract),
        '*' => Some(Multiply),
        '>' => Some(GreaterThan),
        '<' => Some(LessThan),
        '=' => Some(Equals),
        _ => None,
    }
}

pub fn push_operator(
    exprs: &mut Vec<Expression>,
    operator: BinaryOperator,
) -> Result<(), ParsingError> {
    let rhs = exprs.pop().ok_or(ParsingError::NoOperands { operator })?;
    let lhs = exprs.pop().ok_or(ParsingError::NoOperands { operator })?;

    exprs.push(Expression::Binop {
        operator,
        lhs: Box::new(lhs),
        rhs: Box::new(rhs),
    });

    Ok(())
}

pub fn parse(input: &str) -> Result<Expression, ParsingError> {
    let chars: Vec<char> = input.chars().collect();
    _parse(&chars[..])
}

pub fn parse_subexpr(chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    if chars.is_empty() {
        return Ok(None);
    }

    if chars[0] == ')' {
        return Err(ParsingError::UnbalancedRightParen);
    }

    if chars[0] != '(' {
        return Ok(None);
    }

    let mut unmatched = 0;
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '(' {
            unmatched += 1;
        } else if chars[i] == ')' {
            unmatched -= 1;
            if unmatched == 0 {
                return Ok(Some((_parse(&chars[1..i])?, &chars[i + 1..])));
            }
        }
        i += 1;
    }

    Err(ParsingError::UnbalancedLeftParen)
}

pub fn parse_term(chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    Ok(if let Some((n, rest)) = parse_number(chars) {
        Some((Expression::Constant(n.into()), rest))
    } else if let Some((subexpr, rest)) = parse_subexpr(chars)? {
        Some((Expression::Subexpression(Box::new(subexpr)), rest))
    } else {
        None
    })
}

pub fn parse_term_or_dice(chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    Ok(if let Some((dice, rest)) = parse_dice(chars)? {
        Some((dice, rest))
    } else if let Some((term, rest)) = parse_term(chars)? {
        Some((term, rest))
    } else {
        None
    })
}

fn _parse(mut chars: &[char]) -> Result<Expression, ParsingError> {
    let mut expressions: Vec<Expression> = vec![];
    let mut operators: Vec<BinaryOperator> = vec![];

    while !chars.is_empty() {
        while chars[0].is_whitespace() {
            chars = &chars[1..];
        }

        if chars.is_empty() {
            break;
        }

        // TODO: clean this up?
        let id = |expr: Expression| expr;
        let unary_wrapper = if expressions.is_empty() && operators.is_empty() {
            match chars[0] {
                '-' => {
                    chars = &chars[1..];
                    |expr: Expression| Expression::UnaryNegation(Box::new(expr))
                }
                '+' => {
                    chars = &chars[1..];
                    id
                }
                _ => id,
            }
        } else {
            id
        };

        if let Some((term, rest)) = parse_term_or_dice(chars)? {
            expressions.push(unary_wrapper(term));
            chars = rest;
        } else {
            return Err(ParsingError::UndefinedSymbol { char: chars[0] });
        }

        if chars.is_empty() {
            break;
        }

        while chars[0].is_whitespace() {
            chars = &chars[1..];
        }

        if let Some(operator) = parse_operator(chars[0]) {
            if let Some(top_op) = operators.pop() {
                if operator <= top_op {
                    push_operator(&mut expressions, top_op)?;
                    operators.push(operator);
                } else {
                    operators.push(top_op);
                    operators.push(operator);
                }
            } else {
                operators.push(operator);
            }

            chars = &chars[1..];
            continue;
        }
    }

    while let Some(operator) = operators.pop() {
        push_operator(&mut expressions, operator)?;
    }

    if expressions.len() != operators.len() + 1 {
        return Err(ParsingError::MissingOperator);
    }

    expressions.pop().ok_or(ParsingError::EmptyExpression)
}

#[cfg(test)]
mod tests {
    use crate::parser::{parse, BinaryOperator, ParsingError};

    #[test]
    pub fn test_operator_priority() {
        assert!(BinaryOperator::Multiply > BinaryOperator::Subtract);
        assert!(BinaryOperator::Subtract > BinaryOperator::Add);
        assert!(BinaryOperator::Multiply > BinaryOperator::Add);
    }

    #[test]
    pub fn test_missing_operator() {
        assert!(matches!(
            parse("2 + 3 4 + 5"),
            Err(ParsingError::MissingOperator)
        ));

        assert!(matches!(
            parse("(1 2) * 3"),
            Err(ParsingError::MissingOperator)
        ));
    }
}
