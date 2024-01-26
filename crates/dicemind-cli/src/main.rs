use std::io::{stdin, stdout, Result as IOResult, Write};

use dicemind::prelude::*;

pub fn main() -> IOResult<()> {
    let mut fast_roller = FastRoller::default();

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
                match fast_roller.roll(expr.clone()) {
                    Ok(res) => { dbg!(&expr); println!("ok. {res}") },
                    Err(err) => println!("err. {err}"),
                }
            }
            Err(err) => println!("err. {err}"),
        }
    }

    Ok(())
}
