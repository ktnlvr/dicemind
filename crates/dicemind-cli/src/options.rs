use std::{
    error::Error,
    io::{stdout, Write},
};

use clap::ArgMatches;
use rustyline::error::ReadlineError;

#[derive(Debug, Default, Hash, Clone, Copy)]
pub struct DisplayOptions {
    pub height: u32,
    pub width: u32,
}

fn stdin_input() -> impl Iterator<Item = Result<String, Box<dyn Error + 'static>>> {
    std::iter::from_coroutine({
        || {
            let mut rl = match rustyline::DefaultEditor::new() {
                Ok(rl) => rl,
                Err(err) => {
                    // TODO: fix this error handling?
                    let err: Box<dyn Error + 'static> = Box::new(err);
                    yield Err(err);
                    return;
                }
            };

            loop {
                if let Err(err) = stdout().flush() {
                    let err: Box<dyn Error + 'static> = Box::new(err);
                    yield Err(err);
                    return;
                };

                let mut buf = match rl.readline("dice? ") {
                    Err(ReadlineError::Interrupted) => return,
                    Err(err) => {
                        yield Err(Box::new(err));
                        return;
                    }
                    Ok(line) => line,
                };

                buf = buf.trim().to_string();
                if buf.is_empty() {
                    return;
                }

                rl.add_history_entry(buf.clone()).unwrap();
                yield Ok(buf);
            }
        }
    })
}

pub fn input_method_from_args(
    args: &ArgMatches,
) -> Box<dyn Iterator<Item = Result<String, Box<dyn Error + 'static>>>> {
    // XXX: A bit of a mess
    let exprs: Option<Vec<_>> = args
        .get_many::<String>("exprs")
        .map(|iter| iter.cloned().collect::<Vec<_>>());

    if let Some(exprs) = exprs {
        Box::new(
            exprs
                .into_iter()
                .map(|s| -> Result<String, Box<dyn Error + 'static>> {
                    println!("dice? {}", s);
                    Ok(s)
                }),
        )
    } else {
        Box::new(stdin_input())
    }
}
