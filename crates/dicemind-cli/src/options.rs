use std::io::{stdin, stdout, Write};

use clap::ArgMatches;

#[derive(Debug, Clone, Default, Copy)]
pub struct CliOptions {
    pub seed: Option<u64>,
}

pub fn options_from_args(args: &ArgMatches) -> CliOptions {
    let seed = args.get_one("seed").cloned();
    CliOptions { seed }
}

fn stdin_input() -> impl Iterator<Item = std::io::Result<String>> {
    std::iter::from_coroutine(|| loop {
        print!("dice? ");
        if let Err(err) = stdout().flush() {
            yield Err(err);
            return;
        };

        let mut buf = String::new();
        if let Err(err) = stdin().read_line(&mut buf) {
            yield Err(err);
            return;
        };

        buf = buf.trim().to_string();
        if buf.is_empty() {
            return;
        }

        yield Ok(buf);
    })
}

pub fn input_method_from_args(
    args: &ArgMatches,
) -> Box<dyn Iterator<Item = Result<String, std::io::Error>>> {
    // XXX: A bit of a mess
    let exprs: Option<Vec<_>> = args
        .get_many::<String>("exprs")
        .map(|iter| iter.cloned().collect::<Vec<_>>());

    if let Some(exprs) = exprs {
        Box::new(
            exprs
                .into_iter()
                .map(|s| -> Result<String, std::io::Error> {
                    println!("dice? {}", s);
                    Ok(s)
                }),
        )
    } else {
        Box::new(stdin_input())
    }
}
