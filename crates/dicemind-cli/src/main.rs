#![feature(coroutines, coroutine_trait, iter_from_coroutine)]

use defaults::{DEFAULT_HEIGHT, DEFAULT_TRIALS, DEFAULT_WIDTH};
use dicemind::{
    interpreter::{DiceRoll, StandardVerboseRoller, VerboseRoll},
    prelude::*,
};
use human_panic::setup_panic;
use simulate::{print_chart, simulate, SimulationOptions};
use std::error::Error;

mod command;
mod defaults;
mod options;
mod simulate;

use command::*;
use options::*;

fn repl(
    inputs: impl Iterator<Item = Result<String, Box<dyn Error + 'static>>>,
    action: impl Fn(Expression, CliOptions) -> Result<(), Box<dyn Error + 'static>>,
    options: CliOptions,
) -> Result<(), Box<dyn Error + 'static>> {
    for input in inputs {
        let input = input?;
        match parse(&input) {
            Ok(expr) => action(expr, options)?,
            Err(err) => println!("err. {err}"),
        }
    }

    Ok(())
}

fn roll(expr: Expression, opts: CliOptions) -> Result<(), Box<dyn Error + 'static>> {
    let mut fast_roller = StandardVerboseRoller::default();

    match fast_roller.roll(expr.clone()).map(VerboseRoll::into_inner) {
        Ok((sum, annotations)) => {
            let DiceRoll { value, .. } = sum;
            println!("ok. {value}");
            annotations
                .into_iter()
                .for_each(|(note, (expr, DiceRoll { value, .. }))| {
                    println!("{expr} [{note}] = {value}")
                });
        }
        Err(err) => println!("err. {err}"),
    };

    Ok(())
}

fn sim(
    options: SimulationOptions,
    display: DisplayOptions,
) -> Box<dyn Fn(Expression, CliOptions) -> Result<(), Box<dyn Error + 'static>>> {
    Box::new(move |expr: Expression, _: CliOptions| {
        let ft = simulate(expr, options)?;
        print_chart(display, [((0, 0, 0), &ft)].into_iter());

        Ok(())
    })
}

pub fn main() -> Result<(), Box<dyn Error + 'static>> {
    setup_panic!();
    let m = command().get_matches();

    let options = options_from_args(&m);
    let inputs = input_method_from_args(&m);

    match m.subcommand() {
        None => repl(inputs, roll, options)?,
        Some(("simulate", c)) => {
            let trials = c
                .get_one::<u64>("trials")
                .cloned()
                .unwrap_or(DEFAULT_TRIALS);

            let height = c
                .get_one::<u32>("height")
                .cloned()
                .unwrap_or(DEFAULT_HEIGHT);
            let width = c.get_one::<u32>("width").cloned().unwrap_or(DEFAULT_WIDTH);

            repl(
                inputs,
                sim(
                    SimulationOptions { trials },
                    DisplayOptions { height, width },
                ),
                options,
            )?;
        }
        _ => {}
    }

    Ok(())
}
