#![feature(coroutines, coroutine_trait, iter_from_coroutine)]

use defaults::{DEFAULT_HEIGHT, DEFAULT_ITERS, DEFAULT_TRIALS, DEFAULT_WIDTH};
use dicemind::prelude::*;
use human_panic::setup_panic;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{collections::HashMap, io::Result as IOResult};
use textplots::{Chart, Plot, Shape};

mod command;
mod defaults;
mod options;

use command::*;
use options::*;

fn roller_from_opts(opts: CliOptions) -> StandardFastRoller {
    if let Some(seed) = opts.seed {
        StandardFastRoller::new_seeded(seed)
    } else {
        StandardFastRoller::default()
    }
}

fn repl(
    inputs: impl Iterator<Item = IOResult<String>>,
    action: impl Fn(Expression, CliOptions) -> IOResult<()>,
    options: CliOptions,
) -> IOResult<()> {
    for input in inputs {
        match parse(&input?) {
            Ok(expr) => action(expr, options)?,
            Err(err) => println!("err. {err}"),
        }
    }

    Ok(())
}

fn roll(expr: Expression, opts: CliOptions) -> IOResult<()> {
    let mut fast_roller = roller_from_opts(opts);

    match fast_roller.roll(expr.clone()) {
        Ok(res) => {
            println!("ok. {res}")
        }
        Err(err) => println!("err. {err}"),
    };

    Ok(())
}

fn sim(
    iterations: u64,
    trials: u8,
    height: u32,
    width: u32,
) -> Box<dyn Fn(Expression, CliOptions) -> IOResult<()>> {
    Box::new(move |expr: Expression, opts: CliOptions| {
        let tables: Vec<(Vec<_>, _, _)> = (0..trials)
            .into_par_iter()
            .map(|_| {
                let values = (0..iterations)
                    .into_par_iter()
                    .map(|_| roller_from_opts(opts).roll(expr.clone()).unwrap())
                    .fold_with(HashMap::new(), |mut values, n| {
                        *values.entry(n).or_insert_with(|| 0) += 1;
                        values
                    })
                    .reduce_with(|mut a, b| {
                        for (k, u) in b.into_iter() {
                            let v = a.entry(k).or_insert_with(|| 0);
                            *v += u;
                        }
                        a
                    })
                    .unwrap_or_default();

                let mut values: Vec<_> = values.into_iter().collect();
                values.sort_unstable_by_key(|(a, _)| *a);

                let min = values[0].0 as f32;
                let max = values[values.len() - 1].0 as f32;

                (values, min, max)
            })
            .collect();

        for (values, min, max) in tables {
            let mean: f64 = values
                .iter()
                .cloned()
                .map(|(x, freq)| (x as f64 * freq as f64) / (iterations as f64))
                .sum();

            let max_frequency = values.iter().map(|(_, n)| *n).max().unwrap_or(1) as f64;
            let values: Vec<_> = values
                .into_par_iter()
                .map(|(n, m)| (n as f32, (m as f64 / max_frequency) as f32))
                .collect();

            Chart::new_with_y_range(width, height, min, max, 0., 1.)
                .lineplot(&Shape::Steps(&values[..]))
                .display();
            println!("Mean (μ): {mean:.2}")
        }

        Ok(())
    })
}

pub fn main() -> IOResult<()> {
    setup_panic!();
    let m = command().get_matches();

    let options = options_from_args(&m);
    let inputs = input_method_from_args(&m);

    match m.subcommand() {
        None => repl(inputs, roll, options)?,
        Some(("simulate", c)) => {
            let iters = c.get_one::<u64>("iters").cloned().unwrap_or(DEFAULT_ITERS);
            let trials = c.get_one::<u8>("trials").cloned().unwrap_or(DEFAULT_TRIALS);

            let height = c
                .get_one::<u32>("height")
                .cloned()
                .unwrap_or(DEFAULT_HEIGHT);
            let width = c.get_one::<u32>("width").cloned().unwrap_or(DEFAULT_WIDTH);

            repl(inputs, sim(iters, trials, height, width), options)?;
        }
        _ => {}
    }

    Ok(())
}
