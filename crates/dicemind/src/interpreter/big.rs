use crate::{
    parser::{Integer, PositiveInteger},
    visitor::Visitor,
};
use memoize::memoize;
use num::{
    bigint::{RandBigInt, Sign},
    One, ToPrimitive, Zero,
};
use rand::thread_rng;

use super::RollerConfig;

#[derive(Debug, Default)]
pub struct BigRoller {
    config: RollerConfig,
}

#[memoize(Capacity: 128)]
pub fn convolve(a: Vec<PositiveInteger>, b: Vec<PositiveInteger>) -> Vec<PositiveInteger> {
    let mut convolved = vec![PositiveInteger::zero(); a.len() + b.len() - 1];

    for (i, a) in a.into_iter().enumerate() {
        for (j, b) in b.iter().cloned().enumerate() {
            convolved[i + j] += a.clone() * b;
        }
    }

    convolved
}

#[memoize(Capacity: 128)]
fn multiconvolve(count: PositiveInteger, power: PositiveInteger) -> Vec<PositiveInteger> {
    let c: usize = count.to_usize().unwrap();
    let power: usize = power.to_usize().unwrap();

    let mut convolved = vec![PositiveInteger::one(); power];

    for _ in 0..(c - 1) {
        convolved = convolve(convolved, vec![PositiveInteger::one(); power]);
    }

    convolved
}

#[memoize(Capacity: 128)]
fn max_convolved(count: PositiveInteger, power: PositiveInteger) -> PositiveInteger {
    let z = multiconvolve(count, power);
    z[z.len() / 2].clone()
}

#[memoize]
fn nth(count: PositiveInteger, power: PositiveInteger, nth: PositiveInteger) -> PositiveInteger {
    multiconvolve(count, power)[nth.to_usize().unwrap() - 1].clone()
}

// FIXMEEEEE
fn ziggurat(count: PositiveInteger, power: PositiveInteger) -> PositiveInteger {
    let lb: PositiveInteger = count.clone().into();
    let rb: PositiveInteger = (count.clone() * power.clone() + PositiveInteger::one()).into();
    let max = max_convolved(count.clone(), power.clone());

    let mut rng = thread_rng();
    loop {
        let u = rng.gen_biguint_range(&lb, &rb);
        let v = rng.gen_biguint_range(
            &PositiveInteger::one(),
            &(max.clone() + PositiveInteger::one()),
        );

        let n = nth(count.clone(), power.clone(), u.clone());
        if v <= n {
            return u;
        }
    }
}

impl Visitor<Integer> for BigRoller {
    fn visit_negation(&mut self, value: Integer) -> Integer {
        -value
    }

    fn visit_dice(&mut self, count: Option<Integer>, power: Option<Integer>) -> Integer {
        let (s1, count) = count
            .map(|x| x.into_parts())
            .unwrap_or((Sign::Plus, self.config.count()));
        let (s2, power) = power
            .map(|x| x.into_parts())
            .unwrap_or((Sign::Plus, self.config.power()));

        Integer::from_biguint(s1 * s2, ziggurat(count, power))
    }

    fn visit_constant(&mut self, c: Integer) -> Integer {
        c
    }

    fn visit_binop(
        &mut self,
        op: crate::parser::BinaryOperator,
        lhs: Integer,
        rhs: Integer,
    ) -> Integer {
        todo!()
    }
}
