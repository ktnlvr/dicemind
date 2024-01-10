use num::Zero;
use thiserror::Error;

pub type Integer = num::bigint::BigInt;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    GreaterThan,
    LessThan,
    Equals,
}

#[derive(Debug)]
pub enum Expression {
    Dice {
        amount: Option<Integer>,
        power: Option<Integer>,
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
    #[error("Unbalanced left parenthese")]
    UnbalancedLeftParen,
    #[error("Unbalanced right parenthese")]
    UnbalancedRightParen,
    #[error("Undefined symbol")]
    Symbol { char: char },
    #[error("No operands")]
    NoOperands { operator: BinaryOperator },
}

fn parse_number(chars: &[char], i: &mut usize) -> Option<Integer> {
    if !chars[*i].is_digit(10) {
        return None;
    }

    let mut number = Integer::zero();
    let mut len = 0u32;

    let max_len = chars[*i..].iter().take_while(|c| c.is_digit(10)).count() as u32;
    while let Some(d) = chars[*i].to_digit(10) {
        len += 1;
        number += d * Integer::from(10u32).pow(max_len - len);

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

pub fn parse(input: &str) -> Result<Expression, ParsingError> {
    let chars: Vec<char> = input.chars().collect();

    let mut expression: Vec<Expression> = vec![];
    let mut operators: Vec<BinaryOperator> = vec![];

    // This is a very old-school parser, boring but works
    let mut i = 0;
    while i < chars.len() {
        while chars[i].is_whitespace() {
            i += 1;
        }

        if i == chars.len() {
            break;
        }

        let number = parse_number(&chars[..], &mut i);

        if i == chars.len() {
            if let Some(number) = number {
                expression.push(Expression::Constant(number));
            }

            break;
        }

        if chars[i] == 'd' {
            i += 1;

            if i == chars.len() {
                expression.push(Expression::Dice {
                    amount: number,
                    power: None,
                });
                break;
            }

            if let Some(power) = parse_number(&chars[..], &mut i) {
                expression.push(Expression::Dice {
                    amount: number,
                    power: if power.is_zero() { None } else { Some(power) },
                });
            }
        } else {
            if let Some(number) = number {
                expression.push(Expression::Constant(number));
            }
        }

        if i == chars.len() {
            break;
        }

        if let Some(operator) = parse_operator(chars[i]) {
            if let Some(top_op) = operators.pop() {
                if operator <= top_op {
                    // TODO: handle Nones
                    let rhs = expression.pop().unwrap();
                    let lhs = expression.pop().unwrap();

                    expression.push(Expression::Binop {
                        operator: top_op,
                        lhs: Box::new(lhs),
                        rhs: Box::new(rhs),
                    });

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
    }

    assert!(operators.len() + 1 == expression.len());
    while let Some(operator) = operators.pop() {
        let rhs = expression.pop().unwrap();
        let lhs = expression.pop().unwrap();

        expression.push(Expression::Binop {
            operator,
            lhs: Box::new(lhs),
            rhs: Box::new(rhs),
        });
    }

    Ok(expression.pop().unwrap())
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
