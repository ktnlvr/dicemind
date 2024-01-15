use std::io::{stdin, stdout, Result as IOResult, Write};

use interpreter::FastRoller;

use crate::{parser::parse, visitor::Visitor};

mod interpreter;
mod parser;
mod visitor;

pub fn main() -> IOResult<()> {
    let mut fast_roller = FastRoller::default();

    loop {
        print!("+ dice? ");
        stdout().flush()?;

        let mut buf = String::new();
        stdin().read_line(&mut buf)?;
        buf = buf.trim().to_string();

        if buf.is_empty() {
            break;
        }

        let parsed = parse(&buf);
        if let Ok(expr) = parsed {
            match fast_roller.visit(expr) {
                Ok(ok) => println!(" \\ {ok}"),
                Err(err) => println!(" \\ {err}"),
            }
        }
    }

    Ok(())
}
