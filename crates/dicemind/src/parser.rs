use std::cmp::Ordering;

use num::{bigint::Sign, Zero};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use thiserror::Error;

use crate::syntax::{
    Affix, AugmentKind, Augmentation, BinaryOperator, Expression, PositiveInteger, Selector,
};

#[derive(Debug, Error, Clone, Serialize, Deserialize, Copy, Hash, PartialEq, Eq)]
pub enum ParsingError {
    #[error("The string did not contain any expressions")]
    EmptyExpression,
    #[error("Unbalanced left parenthis")]
    UnbalancedLeftParen,
    #[error("Unbalanced right parenthis")]
    UnbalancedRightParen,
    #[error("Unabalanced annotation left bracket")]
    UnbalancedLeftBracket,
    #[error("Unabalanced annotation right bracket")]
    UnbalancedRightBracket,
    #[error("Unexpected symbol `{char}`")]
    UnexpectedSymbol { char: char },
    #[error("No operands")]
    NoOperands { operator: BinaryOperator },
    #[error("Missing operator between operands")]
    MissingOperator,
}

pub fn parse(input: &str) -> Result<Expression, ParsingError> {
    let chars: Vec<char> = input.chars().collect();
    _parse(&chars[..])
}

fn parse_augment_explode(mut chars: &[char]) -> Option<(Augmentation, &[char])> {
    chars.first().filter(|c| **c == '!').map(|_| {
        chars = &chars[1..];
        let selector = parse_selector(chars).map(|(selector, rest)| {
            chars = rest;
            selector
        });

        (Augmentation::Explode { selector }, chars)
    })
}

fn parse_augment_emphasis(mut chars: &[char]) -> Option<(Augmentation, &[char])> {
    chars.first().filter(|c| **c == 'e').map(|_| {
        chars = &chars[1..];
        let n = parse_number(chars).map(|(n, rest)| {
            chars = rest;
            n
        });

        (Augmentation::Emphasis { n }, chars)
    })
}

fn parse_truncation(mut chars: &[char]) -> Option<(Augmentation, &[char])> {
    let kind = match chars.first()? {
        'k' => AugmentKind::Keep,
        'd' => AugmentKind::Drop,
        _ => return None,
    };

    let affix = match chars[1..].first()? {
        'l' => Affix::Low,
        'h' => Affix::High,
        _ => return None,
    };

    chars = &chars[2..];

    let n = parse_number(chars).map(|(n, rest)| {
        chars = rest;
        n
    });

    Some((Augmentation::Truncate { kind, affix, n }, chars))
}

pub fn parse_filter(mut chars: &[char]) -> Option<(Augmentation, &[char])> {
    let kind = match chars.first()? {
        'k' => AugmentKind::Keep,
        'd' => AugmentKind::Drop,
        _ => return None,
    };

    chars = &chars[1..];

    let selector = parse_selector(chars).map(|(n, rest)| {
        chars = rest;
        n
    })?;

    Some((Augmentation::Filter { kind, selector }, chars))
}

fn parse_augments(mut chars: &[char]) -> (impl Iterator<Item = Augmentation>, &[char]) {
    let mut augments: Vec<Augmentation> = vec![];
    let parsers = [
        parse_augment_emphasis,
        parse_augment_explode,
        parse_truncation,
        parse_filter,
    ];

    'outer: while !chars.is_empty() {
        for parser in parsers {
            if let Some((augment, rest)) = parser(chars) {
                augments.push(augment);
                chars = rest;
                continue 'outer;
            }
        }
        break;
    }

    (augments.into_iter(), chars)
}

fn parse_number(chars: &[char]) -> Option<(PositiveInteger, &[char])> {
    if !chars.first()?.is_ascii_digit() {
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

fn parse_selector(chars: &[char]) -> Option<(Selector, &[char])> {
    let relation = match chars.first()? {
        '>' => Ordering::Greater,
        '<' => Ordering::Less,
        '=' => Ordering::Equal,
        _ => return None,
    };

    let (n, rest) = parse_number(&chars[1..])?;

    Some((Selector { relation, n }, rest))
}

fn parse_subexpr(chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    if chars.is_empty() || chars[0] != '(' {
        return Ok(None);
    }

    if chars[0] == ')' {
        return Err(ParsingError::UnbalancedRightParen);
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

fn parse_annotation(chars: &[char]) -> Result<Option<(SmolStr, &[char])>, ParsingError> {
    if chars.is_empty() || chars[0] != '[' {
        return Ok(None);
    }

    if chars[0] == ']' {
        return Err(ParsingError::UnbalancedLeftBracket);
    }

    let mut unmatched = 0;
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '[' {
            unmatched += 1;
        } else if chars[i] == ']' {
            unmatched -= 1;
            if unmatched == 0 {
                return Ok(Some((
                    chars[1..i].iter().cloned().collect(),
                    &chars[i + 1..],
                )));
            }
        }
        i += 1;
    }

    Err(ParsingError::UnbalancedRightBracket)
}

fn parse_term(chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    Ok(parse_number(chars)
        .map(|(n, rest)| (Expression::Constant(n.into()), rest))
        .or(parse_subexpr(chars)?
            .map(|(subexpr, rest)| (Expression::Subexpression(Box::new(subexpr)), rest))))
}

fn parse_term_or_dice(mut chars: &[char]) -> Result<Option<(Expression, &[char])>, ParsingError> {
    let term = parse_term(chars)?.map(|(expr, rest)| {
        chars = rest;
        expr
    });

    if chars.is_empty() {
        return Ok(term.map(|term| (term, chars)));
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

        let (augs, rest) = parse_augments(chars);
        chars = rest;

        return Ok(Some((
            Expression::Dice {
                count: term.map(Box::new),
                power,
                augmentations: augs.collect(),
            },
            chars,
        )));
    }

    Ok(term.map(|term| (term, chars)))
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

        let explicit_sign = {
            let sign = match chars.first() {
                Some('+') => Some(Sign::Plus),
                Some('-') => Some(Sign::Minus),
                _ => None,
            };

            if sign.is_some() {
                chars = &chars[1..];
            }

            sign
        };

        if chars.is_empty() {
            return Err(ParsingError::NoOperands {
                operator: BinaryOperator::Add,
            });
        }

        let (term, rest) =
            parse_term_or_dice(chars)?.ok_or(ParsingError::UnexpectedSymbol { char: chars[0] })?;
        chars = rest;

        let mut expr = if explicit_sign == Some(Sign::Minus) {
            Expression::UnaryNegation(Box::new(term))
        } else {
            term
        };

        if chars.is_empty() {
            expressions.push(expr);
            break;
        }

        while chars[0].is_whitespace() {
            chars = &chars[1..];
        }

        if let Some((annotation, rest)) = parse_annotation(chars)? {
            chars = rest;
            expr = Expression::Annotated {
                expression: Box::new(expr),
                annotation,
            }
        }

        expressions.push(expr);

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
