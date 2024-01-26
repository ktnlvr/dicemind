use clap::{arg, value_parser, ArgAction, Command};
use dicemind::{parser::Expression, prelude::*};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::{
    collections::HashMap,
    io::{stdin, stdout, Result as IOResult, Write},
};
use textplots::{Chart, Plot, Shape};

fn repl(action: impl Fn(Expression) -> IOResult<()>) -> IOResult<()> {
    loop {
        print!("dice? ");
        stdout().flush()?;

        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        buf = buf.trim().to_string();

        if buf.is_empty() {
            break;
        }

        match parse(&buf) {
            Ok(expr) => action(expr)?,
            Err(err) => println!("err. {err}"),
        }
    }

    Ok(())
}

fn roll(expr: Expression) -> IOResult<()> {
    let mut fast_roller = FastRoller::default();

    match fast_roller.roll(expr.clone()) {
        Ok(res) => {
            dbg!(&expr);
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
) -> Box<dyn Fn(Expression) -> IOResult<()>> {
    Box::new(move |expr: Expression| {
        let tables: Vec<(Vec<_>, _, _)> = (0..trials)
            .into_par_iter()
            .map(|_| {
                let mut values: HashMap<i32, u64> = HashMap::new();
                let mut fast_roller = FastRoller::default();

                for _ in 0..iterations {
                    let n = fast_roller.roll(expr.clone()).unwrap();
                    let value = values.entry(n).or_insert_with(|| 0);
                    *value += 1;
                }

                let mut values: Vec<_> = values.into_iter().collect();
                values.sort_unstable_by_key(|(a, _)| *a);

                let max_chance = values.iter().map(|(_, n)| *n).max().unwrap_or(1) as f64;

                let min = values[0].0 as f32;
                let max = values[values.len() - 1].0 as f32;

                let values: Vec<_> = values
                    .into_par_iter()
                    .map(|(n, m)| (n as f32, (m as f64 / max_chance) as f32))
                    .collect();

                (values, min, max)
            })
            .collect();

        for (values, min, max) in tables {
            Chart::new_with_y_range(width, height, min, max, 0., 1.)
                .lineplot(&Shape::Steps(&values[..]))
                .nice();
        }

        Ok(())
    })
}

pub fn main() -> IOResult<()> {
    let m = Command::new("dicemind")
        .subcommand(
            Command::new("simulate")
                .short_flag('s')
                .long_flag("sim")
                .arg(
                    arg!(-i - -iters)
                        .value_parser(value_parser!(u64))
                        .action(ArgAction::Set)
                        .num_args(1),
                )
                .arg(
                    arg!(-t - -trials)
                        .value_parser(value_parser!(u8))
                        .action(ArgAction::Set)
                        .num_args(1),
                )
                .arg(
                    arg!(-W - -width)
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set)
                        .num_args(1),
                )
                .arg(
                    arg!(-H - -height)
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set)
                        .num_args(1),
                ),
        )
        .arg_required_else_help(true)
        .get_matches();

    match m.subcommand() {
        None => repl(roll)?,
        Some(("simulate", c)) => {
            let iters = c.get_one::<u64>("iters").cloned().unwrap_or(10000);
            let trials = c.get_one::<u8>("trials").cloned().unwrap_or(1);

            let h = c.get_one::<u32>("height").cloned().unwrap_or(40);
            let w = c.get_one::<u32>("width").cloned().unwrap_or(120);

            repl(sim(iters, trials, h, w))?;
        }
        _ => {}
    }

    Ok(())
}
