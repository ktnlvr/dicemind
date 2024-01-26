use clap::{arg, value_parser, ArgAction, Command};

pub fn command() -> Command {
    Command::new("dicemind")
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
}
