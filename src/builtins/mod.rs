mod tools;
mod symbols;

use std::io::Result;

pub fn handle_command(cmd: &str, args: &[&str]) -> Option<Result<()>> {
    match cmd {
        "cd" => Some(tools::change_directory(args.first().unwrap_or(&"~"))),
        "alias" => Some(tools::handle_alias_cmd(args)),
        "export" => Some(tools::handle_export(args)),
        "exit" => std::process::exit(0),
        ">" | ">>" | "<" | "|" => Some(symbols::handle_symbol(cmd, args)),
        _ => None
    }
}

pub fn expand_aliases(input: &str) -> String {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.is_empty() {
        return input.to_string();
    }

    if let Some(alias_cmd) = tools::lookup_alias(parts[0]) {
        let rest = if parts.len() > 1 {
            format!(" {}", parts[1..].join(" "))
        } else {
            String::new()
        };
        format!("{}{}", alias_cmd, rest)
    } else {
        input.to_string()
    }
}
