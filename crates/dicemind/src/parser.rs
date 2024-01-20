use num::Zero;
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
        count: Option<PositiveInteger>,
        power: Option<PositiveInteger>,
    },
    Binop {
        operator: BinaryOperator,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Constant(Integer),
    Subexpression(Box<Expression>),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("The string did not contain any expressions")]
    EmptyExpression,
    #[error("Unbalanced left parenthis")]
    UnbalancedLeftParen,
    #[error("Unbalanced right parenthis")]
    UnbalancedRightParen,
    #[error("Undefined symbol")]
    UndefinedSymbol { char: char },
    #[error("No operands")]
    NoOperands { operator: BinaryOperator },
    #[error("Missing operator between operands")]
    MissingOperator,
}

fn parse_number<'a>(chars: &'a [char]) -> Option<(PositiveInteger, &'a [char], usize)> {
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

    Some((number, &chars[1..], len))
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

pub fn parse_subexpr(chars: &[char]) -> Result<Option<(Expression, &[char], usize)>, ParsingError> {
    if chars.len() == 0 {
        return Ok(None);
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
                return Ok(Some((_parse(&chars[1..i])?, &chars[i + 1..], i + 1)));
            }
        }
        i += 1;
    }

    return Err(ParsingError::UnbalancedLeftParen);
}

fn _parse(chars: &[char]) -> Result<Expression, ParsingError> {
    let mut expression: Vec<Expression> = vec![];
    let mut operators: Vec<BinaryOperator> = vec![];

    // This is a very old-school parser, boring but works
    let mut i = 0;
    while i < chars.len() {
        let begin_i = i;

        while chars[i].is_whitespace() {
            i += 1;
        }

        if chars[i] == ')' {
            return Err(ParsingError::UnbalancedRightParen);
        }

        if let Some((subexpr, _rest, len)) = parse_subexpr(&chars[i..])? {
            expression.push(Expression::Subexpression(Box::new(subexpr)));
            i += len;
            continue;
        }

        let number = if let Some((n, _rest, len)) = parse_number(&chars[i..]) {
            i += len;
            Some(n)
        } else {
            None
        };

        if i < chars.len() && chars[i] == 'd' {
            i += 1;

            let power = if let Some((power, _rest, len)) = parse_number(&chars[i..]) {
                i += len;

                if power.is_zero() {
                    None
                } else {
                    Some(power)
                }
            } else {
                None
            };

            expression.push(Expression::Dice {
                count: number,
                power,
            });

            continue;
        } else {
            if let Some(number) = number {
                expression.push(Expression::Constant(number.into()));
                continue;
            }
        }

        while chars[i].is_whitespace() {
            i += 1;
        }

        if let Some(operator) = parse_operator(chars[i]) {
            if let Some(top_op) = operators.pop() {
                if operator <= top_op {
                    push_operator(&mut expression, operator)?;
                    operators.push(operator);
                } else {
                    operators.push(top_op);
                    operators.push(operator);
                }
            } else {
                operators.push(operator);
            }

            i += 1;
            continue;
        }

        if begin_i == i {
            // Symbol not captured by any handler
            return Err(ParsingError::UndefinedSymbol { char: chars[i] });
        }
    }

    while let Some(operator) = operators.pop() {
        push_operator(&mut expression, operator)?;
    }

    if expression.len() != operators.len() + 1 {
        return Err(ParsingError::MissingOperator);
    }

    expression.pop().ok_or(ParsingError::EmptyExpression)
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
