use std::io::{stdin, stdout, Result as IOResult, Write};

use crate::interpreter::SimpleRoller;
use crate::parser::parse;
use crate::visitor::Visitor;

mod interpreter;
mod parser;
mod visitor;

pub fn main() -> IOResult<()> {
    let mut simple_roller = SimpleRoller::default();

    loop {
        print!("dice> ");
        stdout().flush()?;

        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        buf = buf.trim().to_string();

        if buf.is_empty() {
            break;
        }

        let parsed = parse(&buf);
        println!("{:?}", parsed);
        if let Ok(expr) = parsed {
            println!("{}", simple_roller.visit(expr));
        }
    }

    Ok(())
}
