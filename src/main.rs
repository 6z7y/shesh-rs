mod builtin;
mod commands;
mod config;
mod shell;

use std::{
    path::Path,
    io::{self, Result, Write},
};

use crate::{
    commands::CommandSeparator,
};

fn process_command(cmd_str: &str, background: bool) -> bool {
    // Command expansion
    let expanded = {
        let step1 = commands::expand_vars(cmd_str);
        let step2 = commands::expand_tilde(&step1);
        commands::parse_input(&step2)
            .iter()
            .flat_map(|p| commands::expand_braces(p))
            .flat_map(|p| {
                if p.contains('*') {
                    commands::expand_wildcard(&p)
                } else {
                    vec![p]
                }
            })
            .collect::<Vec<String>>()
    };

    let parts: Vec<&str> = expanded.iter().map(|s| s.as_str()).collect();
    let joined_input = expanded.join(" ");

    // Handle pipelines
    if parts.contains(&"|") {
        let commands = commands::parse_pipeline(&joined_input);
        return if background {
            shell::execute_background_pipeline(commands).is_ok()
        } else {
            shell::execute_pipeline(commands).is_ok()
        };
    }

    // Handle redirections
    let parsed = commands::parse_redirects(&joined_input);
    if !parsed.redirects.is_empty() {
        return if background {
            shell::execute_background_with_redirect(&parsed.cmd, &parsed.redirects).is_ok()
        } else {
            shell::execute_with_redirect(&parsed.cmd, &parsed.redirects).is_ok()
        };
    }

    let (cmd, args) = match parts.split_first() {
        Some((c, a)) => (c, a),
        None => return false,
    };

    // Built-in commands
    if Path::new(cmd).is_dir() {
        return if let Err(e) = builtin::cd(cmd) {
            eprintln!("{}", e);
            false
        } else {
            true
        };
    }

    if let Some(result) = builtin::handle_builtin(cmd, args) {
        return if let Err(e) = result {
            eprintln!("{}", e);
            false
        } else {
            true
        };
    }

    // External commands
    if background {
        shell::execute_background(cmd, args).is_ok()
    } else {
        shell::execute(cmd, args).is_ok()
    }
}

fn main() -> Result<()> { 
    let config = config::init();
    config::run_startup(&config);

    loop {
        print!("{}", config.prompt);
        io::stdout().flush()?;
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        
        let input = input.trim().split('#').next().unwrap().trim();
        if input.is_empty() { continue; };

        // Alias expansion
        let mut line = input.to_string();
        let parts0: Vec<&str> = line.split_whitespace().collect();
        if let Some(alias_cmd) = builtin::lookup_alias(parts0[0]) {
            let rest = if parts0.len() > 1 {
                format!(" {}", parts0[1..].join(" "))
            } else {
                String::new()
            };
            line = format!("{}{}", alias_cmd, rest);
        }

        // Save command history
        config::save_history(&line);

        // Split commands
        let tokens = commands::split_commands(&line);
        let mut last_success = true;
        let mut background_next = false;

        for (cmd_str, separator) in tokens {
            // Handle background flag
            let background = background_next;
            background_next = false;

            match separator {
                CommandSeparator::AndAnd if !last_success => continue,
                CommandSeparator::Background => {
                    background_next = true;
                    continue;
                }
                _ => {}
            }

            last_success = process_command(&cmd_str, background);
        }
    }
}
