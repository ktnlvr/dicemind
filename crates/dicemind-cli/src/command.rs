use clap::{arg, value_parser, ArgAction, Command};

pub fn command() -> Command {
    Command::new("dicemind")
        .arg(
            arg!([EXPRS] ... "Expressions to evaluate")
                .id("exprs")
                .value_parser(value_parser!(String))
                .action(ArgAction::Append),
        )
        .arg(
            arg!(--seed)
                .value_parser(value_parser!(u64))
                .num_args(1)
                .action(ArgAction::Set),
        )
        .subcommand(
            Command::new("simulate")
                .short_flag('s')
                .long_flag("sim")
                .arg(
                    arg!(-i - -iters)
                        .value_parser(value_parser!(u64))
                        .num_args(1)
                        .action(ArgAction::Set),
                )
                .arg(
                    arg!(-t - -trials)
                        .value_parser(value_parser!(u8))
                        .num_args(1)
                        .action(ArgAction::Set),
                )
                .arg(
                    arg!(-W - -width)
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set)
                        .num_args(1)
                        .action(ArgAction::Set),
                )
                .arg(
                    arg!(-H - -height)
                        .value_parser(value_parser!(u32))
                        .action(ArgAction::Set)
                        .num_args(1)
                        .action(ArgAction::Set),
                ),
        )
}
