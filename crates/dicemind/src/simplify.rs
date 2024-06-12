use crate::{
    options::RollerOptions,
    syntax::{Expression, Integer},
};

bitflags::bitflags! {
    #[derive(Clone, Copy, Default, Debug)]
    pub struct Steps: u32 {
        // Emplace values from the Roller Options into the expression directly
        const INLINE_IMPLICIT_OPTIONS = 1 << 0;
        // a = 1, b = 2, a + b => 1 + 3
        const INLINE_CHAINED_VARIABLES = 1 << 1;
        // 0d9 = 0, 8d1 = 8 and others
        const REPLACE_CONSTANT_VALUED_DICE = 1 << 2;
        // 2 * (1 + d) => 2 + 2 * d
        const DISTRIBUTE_OPERATIONS = 1 << 3;
        // 2 + 4 => 6
        const COLLAPE_CONSTANTS = 1 << 4;
    }
}

pub fn simplify(expr: Expression) -> Expression {
    advanced_simplify(expr, &RollerOptions::default(), Steps::all())
}

pub fn advanced_simplify(expr: Expression, options: &RollerOptions, steps: Steps) -> Expression {
    use Expression as E;

    match expr {
        E::Dice {
            quantity,
            power,
            augmentations,
        } => {
            let mut q = quantity.map(|expr| advanced_simplify(*expr, options, steps));
            let mut p = power.map(|expr| advanced_simplify(*expr, options, steps));

            if steps.contains(Steps::INLINE_IMPLICIT_OPTIONS) {
                q = q.or_else(|| Some(E::Constant(options.quantity().into())));
                p = p.or_else(|| Some(E::Constant(options.power().into())));
            }

            if steps.contains(Steps::REPLACE_CONSTANT_VALUED_DICE) {
                use num::One;
                fn is_one(expr: &E) -> bool {
                    matches!(expr, E::Constant(c) if c.is_one())
                }

                if p.as_ref().is_some_and(is_one) {
                    if let Some(expr) = q {
                        return expr;
                    }
                }

                use num::Zero;
                fn is_zero(expr: &E) -> bool {
                    matches!(expr, E::Constant(c) if c.is_zero())
                }

                if p.as_ref().is_some_and(is_zero) || q.as_ref().is_some_and(is_zero) {
                    return E::Constant(Integer::zero());
                }
            }

            E::Dice {
                quantity: q.map(Box::new),
                power: p.map(Box::new),
                augmentations,
            }
        }
        _ => expr,
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        prelude::{advanced_simplify, parse, Expression, RollerOptions},
        simplify::Steps,
        syntax::Integer,
    };

    use Expression as E;

    #[test]
    fn test_inlining_implicit_options() {
        let parsed = parse("d").unwrap();
        let default_options = RollerOptions::default();

        let simplified =
            advanced_simplify(parsed, &default_options, Steps::INLINE_IMPLICIT_OPTIONS);

        matches!(
            simplified,
            E::Dice {
                quantity: Some(box q),
                power: Some(box p),
                ..
            } if q == E::Constant(default_options.quantity().into()) && p == E::Constant(default_options.power().into())
        );
    }

    #[test]
    fn test_constant_dice() {
        use num::Zero;

        let simplify = |expr| {
            advanced_simplify(
                expr,
                &RollerOptions::default(),
                Steps::REPLACE_CONSTANT_VALUED_DICE,
            )
        };

        assert_eq!(simplify(parse("0d4").unwrap()), E::Constant(Integer::zero()));
        assert_eq!(simplify(parse("4d0").unwrap()), E::Constant(Integer::zero()));
        assert_eq!(simplify(parse("8d1").unwrap()), E::Constant(Integer::from(8)));
    }
}
