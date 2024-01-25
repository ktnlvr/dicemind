#![feature(box_patterns)]
#![feature(result_flattening)]

use std::io::{stdin, stdout, Result as IOResult, Write};

use interpreter::FastRoller;
use num::One;

use crate::{
    interpreter::{convolve, BigRoller},
    parser::{parse, PositiveInteger},
    visitor::Visitor,
};

mod interpreter;
mod parser;
mod visitor;

pub fn main() -> IOResult<()> {
    let mut fast_roller = BigRoller::default();
    println!(
        "{:?}",
        convolve(
            vec![PositiveInteger::one(); 6],
            vec![PositiveInteger::one(); 6]
        )
    );

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
            Ok(expr) => {
                dbg!(&expr);
                println!("{}", fast_roller.visit(expr))
            }
            Err(err) => println!("err. {err}"),
        }
    }

    Ok(())
}
