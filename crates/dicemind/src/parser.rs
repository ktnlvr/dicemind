use std::cmp::Ordering;

use num::Zero;
use smallvec::{smallvec, SmallVec};
use thiserror::Error;

pub type Integer = num::bigint::BigInt;
pub type PositiveInteger = num::bigint::BigUint;

#[derive(Debug, PartialEq, Eq, PartialOrd, Clone, Copy, Hash)]
pub enum BinaryOperator {
    Equals,
    LessThan,
    GreaterThan,
    Add,
    Subtract,
    Multiply,
}

impl Into<u8> for BinaryOperator {
    fn into(self) -> u8 {
        use BinaryOperator::*;

        match self {
            Equals | LessThan | GreaterThan => 3,
            Multiply => 2,
            Add | Subtract => 1,
        }
    }
}

impl Ord for BinaryOperator {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let l: u8 = self.clone().into();
        let r: u8 = other.clone().into();

        l.cmp(&r)
    }
}

#[derive(Debug)]
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

#[derive(Debug)]
pub enum DependentAugmentKind {
    Drop,
    Keep,
    Reroll,
}

#[derive(Debug)]
pub enum Augmentation {
    Dependent {
        kind: DependentAugmentKind,
        relation: Ordering,
        n: Option<PositiveInteger>,
    },
    Emphasis {
        e: PositiveInteger,
    },
    Explode {
        n: PositiveInteger,
    },
}

#[derive(Debug, Error)]
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

fn parse_number<'a>(chars: &'a [char]) -> Option<(PositiveInteger, &'a [char])> {
    if chars.len() == 0 {
        return None;
    }

    if !chars[0].is_digit(10) {
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

    if chars.len() == 0 {
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

        return Ok(Some((
            Expression::Dice {
                count,
                power,
                augmentations: smallvec![],
            },
            chars,
        )));
    }

    Ok(None)
}

fn parse_operator(char: char) -> Option<BinaryOperator> {
    match char {
        '+' => Some(BinaryOperator::Add),
        '-' => Some(BinaryOperator::Subtract),
        '*' => Some(BinaryOperator::Multiply),
        '>' => Some(BinaryOperator::GreaterThan),
        '<' => Some(BinaryOperator::LessThan),
        '=' => Some(BinaryOperator::Equals),
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
    if chars.len() == 0 {
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

    return Err(ParsingError::UnbalancedLeftParen);
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

    while chars.len() != 0 {
        while chars[0].is_whitespace() {
            chars = &chars[1..];
        }

        if chars.len() == 0 {
            break;
        }

        // TODO: clean this up?
        let id = |expr: Expression| expr;
        let unary_wrapper = if expressions.len() == 0 && operators.len() == 0 {
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

        if chars.len() == 0 {
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
