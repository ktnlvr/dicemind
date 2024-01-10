#![feature(coroutines)]
#![feature(iter_from_coroutine)]

use std::io::{stdin, Result as IOResult, stdout, Write};

use crate::parser::parse;

mod parser;

pub fn main() -> IOResult<()> {
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
    }

    Ok(())
}
