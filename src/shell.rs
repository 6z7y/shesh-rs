use {
    std::io,
    crate::{
        builtins::{execute_external, expand_aliases, get_builtin},
        parse::{parse_syntax, process_tokens, Operator, ParsedCommand},
        process_exec::{flatten_pipes, run_background, run_pipe, handle_redirect}}
};

pub fn exec(cmd: &str) -> io::Result<()> {
    let expanded_cmd = expand_aliases(cmd);
    let command = parse_syntax(&expanded_cmd);
    run(command)
}

pub fn run(cmd: ParsedCommand) -> io::Result<()> {
    match cmd {
        ParsedCommand::Single(args) => {
            // if args.is_empty() {
            //     return Ok(());
            // }

            let str_args = process_tokens(ParsedCommand::Single(args));
            let cmd = str_args[0].as_str();
            get_builtin(cmd).map_or_else(||
                execute_external(cmd, &str_args[1..].iter().map(|s| s.as_str()).collect::<Vec<_>>()),
                |handler| handler(&str_args[1..])
            )
        },
        ParsedCommand::BinaryOp(left, op, right) => {
            match op {
                Operator::Seq => {
                    run(*left)?;
                    run(*right)
                }
                Operator::And => {
                    if run(*left).is_ok() {
                        run(*right)
                    } else {
                        Ok(())
                    }
                }
                Operator::Or => {
                    if run(*left).is_err() {
                        run(*right)
                    } else {
                        Ok(())
                    }
                }
                Operator::Pipe => {
                    let commands = flatten_pipes(vec![*left, *right]);
                    run_pipe(commands)
                }
                Operator::Background => run_background(*left),
                Operator::Redirect(redirect_type) => handle_redirect(*left, redirect_type, *right),
            }
        }
    }
}
