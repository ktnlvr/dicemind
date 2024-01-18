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
    #[error("Unbalanced left parenthese")]
    UnbalancedLeftParen,
    #[error("Unbalanced right parenthese")]
    UnbalancedRightParen,
    #[error("Undefined symbol")]
    UndefinedSymbol { char: char },
    #[error("No operands")]
    NoOperands { operator: BinaryOperator },
}

fn parse_number(chars: &[char], i: &mut usize) -> Option<PositiveInteger> {
    if *i >= chars.len() {
        return None;
    }

    if !chars[*i].is_digit(10) {
        return None;
    }

    let mut number = PositiveInteger::zero();
    let mut len = 0u32;

    let max_len = chars[*i..].iter().take_while(|c| c.is_digit(10)).count() as u32;
    while let Some(d) = chars[*i].to_digit(10) {
        len += 1;
        number += d * PositiveInteger::from(10u32).pow(max_len - len);

        *i += 1;
        if *i == chars.len() {
            break;
        }
    }

    Some(number)
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

    return Ok(None)
}

fn _parse(mut chars: &[char]) -> Result<Expression, ParsingError> {
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
        }

        if i == chars.len() {
            break;
        }

        let number = parse_number(&chars[..], &mut i);

        if i == chars.len() {
            if let Some(number) = number {
                expression.push(Expression::Constant(number.into()));
            }

            break;
        }

        if chars[i] == 'd' {
            i += 1;

            let dice = if let Some(power) = parse_number(&chars[..], &mut i) {
                Expression::Dice {
                    count: number,
                    power: if power.is_zero() { None } else { Some(power) },
                }
            } else {
                Expression::Dice {
                    count: number,
                    power: None,
                }
            };

            expression.push(dice);
        } else {
            if let Some(number) = number {
                expression.push(Expression::Constant(number.into()));
            }
        }

        if i == chars.len() {
            break;
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
        }

        if begin_i == i {
            // Symbol not captured by any handler
            return Err(ParsingError::UndefinedSymbol { char: chars[i] });
        }
    }

    while let Some(operator) = operators.pop() {
        push_operator(&mut expression, operator)?;
    }

    expression.pop().ok_or(ParsingError::EmptyExpression)
}

#[cfg(test)]
mod tests {
    use crate::parser::BinaryOperator;

    #[test]
    pub fn test_operator_priority() {
        assert!(BinaryOperator::Multiply > BinaryOperator::Subtract);
        assert!(BinaryOperator::Subtract > BinaryOperator::Add);
        assert!(BinaryOperator::Multiply > BinaryOperator::Add);
    }
}
