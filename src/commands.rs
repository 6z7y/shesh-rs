use std::{env, fs, path::PathBuf};

use crate::{
    builtins,
    shell
};

#[derive(Debug, PartialEq)]
pub enum CommandSeparator {
    AndAnd,     // &&
    SemiColon,  // ;
    Background, // &
    None,
}

#[derive(Debug, Clone)]
pub enum Redirect {
    Output(String),
    Append(String),
    Input(String),
}

#[derive(Debug)]
pub struct ParsedCommand {
    pub cmd: Vec<String>,
    pub redirects: Vec<Redirect>,
}

pub fn parse_input(input: &str) -> Vec<String> {
    let (mut final_parts, final_buffer, _, _) = input.chars().fold(
        (Vec::new(), String::new(), None, false),
        |(mut parts_acc, mut buffer_acc, in_quote_acc, escape_next_acc), char_val| {
            if escape_next_acc {
                buffer_acc.push(char_val);
                (parts_acc, buffer_acc, in_quote_acc, false)
            } else {
                match char_val {
                    '\\' => (parts_acc, buffer_acc, in_quote_acc, true),
                    q @ ('"' | '\'') => {
                        if in_quote_acc == Some(q) {
                            (parts_acc, buffer_acc, None, false)
                        } else if in_quote_acc.is_none() {
                            (parts_acc, buffer_acc, Some(q), false)
                        } else {
                            buffer_acc.push(q);
                            (parts_acc, buffer_acc, in_quote_acc, false)
                        }
                    }
                    ' ' if in_quote_acc.is_none() => {
                        if !buffer_acc.is_empty() {
                            parts_acc.push(buffer_acc);
                            (parts_acc, String::new(), in_quote_acc, false)
                        } else {
                            (parts_acc, buffer_acc, in_quote_acc, false)
                        }
                    }
                    _ => {
                        buffer_acc.push(char_val);
                        (parts_acc, buffer_acc, in_quote_acc, false)
                    }
                }
            }
        },
    );

    if !final_buffer.is_empty() {
        final_parts.push(final_buffer);
    }
    final_parts
}

pub fn split_commands(input: &str) -> Vec<(String, CommandSeparator)> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = None;
    let mut escape = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if escape {
            current.push(c);
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            q @ ('"' | '\'') => {
                if in_quote == Some(q) {
                    in_quote = None;
                } else if in_quote.is_none() {
                    in_quote = Some(q);
                }
                current.push(q);
            }
            ';' if in_quote.is_none() => {
                tokens.push((current.trim().to_string(), CommandSeparator::SemiColon));
                current.clear();
            }
            '&' if in_quote.is_none() => {
                if let Some('&') = chars.peek() {
                    chars.next();
                    tokens.push((current.trim().to_string(), CommandSeparator::AndAnd));
                } else {
                    tokens.push((current.trim().to_string(), CommandSeparator::Background));
                }
                current.clear();
            }
            _ => current.push(c),
        }
    }

    if !current.trim().is_empty() {
        tokens.push((current.trim().to_string(), CommandSeparator::None));
    }

    tokens
}

pub fn expand_braces(input: &str) -> Vec<String> {
    let (stack, result_strings) = input.chars().fold(
        (Vec::<Vec<String>>::new(), vec![String::new()]),
        |(mut stack_acc, current_strings), c| match c {
            '{' => {
                stack_acc.push(current_strings);
                (stack_acc, vec![String::new()])
            }
            '}' => {
                let items_in_braces: Vec<String> = current_strings
                    .iter()
                    .flat_map(|s_in_brace| s_in_brace.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();

                let prefixes = stack_acc.pop().unwrap_or_else(|| vec![String::new()]);

                let new_strings = prefixes
                    .iter()
                    .flat_map(|prev_part| {
                        if items_in_braces.is_empty() {
                            Vec::new()
                        } else {
                            items_in_braces
                                .iter()
                                .map(move |current_brace_item| {
                                    format!("{}{}", prev_part, current_brace_item)
                                })
                                .collect::<Vec<String>>()
                        }
                    })
                    .collect();
                (stack_acc, new_strings)
            }
            _ => {
                let new_strings = current_strings
                    .iter()
                    .map(|s| format!("{}{}", s, c))
                    .collect();
                (stack_acc, new_strings)
            }
        },
    );

    if !stack.is_empty() {}
    result_strings
}

pub fn expand_vars(input: &str) -> String {
    let (mut final_result_str, final_var_name_segment, in_var_at_end) = input.chars().fold(
        (String::new(), String::new(), false),
        |(mut res_acc, mut var_name_acc, in_var_mode_acc), current_char| {
            if !in_var_mode_acc {
                if current_char == '$' {
                    (res_acc, String::new(), true)
                } else {
                    res_acc.push(current_char);
                    (res_acc, var_name_acc, false)
                }
            } else {
                if current_char.is_alphanumeric() || current_char == '_' {
                    var_name_acc.push(current_char);
                    (res_acc, var_name_acc, true)
                } else {
                    res_acc.push_str(&env::var(&var_name_acc).unwrap_or_default());
                    if current_char == '$' {
                        (res_acc, String::new(), true)
                    } else {
                        res_acc.push(current_char);
                        (res_acc, String::new(), false)
                    }
                }
            }
        },
    );

    if in_var_at_end {
        final_result_str.push_str(&env::var(&final_var_name_segment).unwrap_or_default());
    }
    final_result_str
}
fn wildcard_match(name: &str, pattern: &str) -> bool {
    let pattern = pattern.trim();
    if pattern == "*" {
        return true;
    }
    if let Some(suffix) = pattern.strip_prefix('*') {
        return name.ends_with(suffix);
    }
    if let Some(prefix) = pattern.strip_suffix('*') {
        return name.starts_with(prefix);
    }
    name == pattern
}

pub fn expand_wildcard(pattern: &str) -> Vec<String> {
    let trimmed_pattern = pattern.trim();
    std::path::Path::new(".")
        .read_dir()
        .map(|read_dir_result| {
            read_dir_result
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| wildcard_match(name, trimmed_pattern))
                .collect::<Vec<String>>()
        })
        .unwrap_or_else(|_err| Vec::new())
}

pub fn parse_redirects(input: &str) -> ParsedCommand {
    let mut parts = input.split_whitespace();
    let mut cmd = Vec::new();
    let mut redirects = Vec::new();

    while let Some(part) = parts.next() {
        match part {
            ">" => {
                if let Some(file) = parts.next() {
                    redirects.push(Redirect::Output(file.to_string()));
                }
            }
            ">>" => {
                if let Some(file) = parts.next() {
                    redirects.push(Redirect::Append(file.to_string()));
                }
            }
            "<" => {
                if let Some(file) = parts.next() {
                    redirects.push(Redirect::Input(file.to_string()));
                }
            }
            _ => cmd.push(part.to_string()),
        }
    }
    ParsedCommand { cmd, redirects }
}

pub fn expand_tilde(path: &str) -> String {
    if path.starts_with("~/") {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        path.replacen('~', &home, 1)
    } else if path == "~" {
        env::var("HOME").unwrap_or_else(|_| ".".to_string())
    } else {
        path.to_string()
    }
}

pub fn parse_pipeline(input: &str) -> Vec<Vec<String>> {
    input
        .split('|')
        .map(|part| part.split_whitespace().map(|s| s.to_string()).collect())
        .collect()
}

/// Complete command names based on input prefix
pub fn complete_command(prefix: &str) -> Vec<String> {
    let path = env::var("PATH").unwrap_or_default();
    let mut completions = Vec::new();

    for dir in path.split(':') {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                if let Ok(file_name) = entry.file_name().into_string() {
                    if file_name.starts_with(prefix) {
                        completions.push(file_name);
                    }
                }
            }
        }
    }

    completions
}

/// Complete path based on input prefix
pub fn complete_path(prefix: &str) -> Vec<String> {
    let expanded_prefix = expand_tilde(prefix);
    let path = PathBuf::from(&expanded_prefix);
    let parent = if path.is_dir() {
        path.clone()
    } else {
        path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
    };

    let mut completions = Vec::new();
    let file_stem = path.file_name().and_then(|s| s.to_str()).unwrap_or("");

    if let Ok(entries) = fs::read_dir(parent) {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if file_name.starts_with(file_stem) {
                    let mut full_path = entry.path();
                    if full_path.is_dir() {
                        full_path.push("");
                    }
                    if let Some(full_str) = full_path.to_str() {
                        completions.push(full_str.to_string());
                    }
                }
            }
        }
    }

    completions
}

// Main command processing
pub fn process_command(cmd_str: &str, background: bool) -> bool {
    // Step 1: Command expansion
    let expanded = {
        let step1 = expand_vars(cmd_str);
        let step2 = expand_tilde(&step1);
        parse_input(&step2)
            .iter()
            .flat_map(|p| expand_braces(p))
            .flat_map(|p| {
                if p.contains('*') {
                    expand_wildcard(&p)
                } else {
                    vec![p]
                }
            })
            .collect::<Vec<String>>()
    };

    let parts: Vec<&str> = expanded.iter().map(|s| s.as_str()).collect();
    let joined_input = expanded.join(" ");

    // Step 2: Handle pipelines
    if parts.contains(&"|") {
        let commands = parse_pipeline(&joined_input);
        return if background {
            shell::execute_background_pipeline(commands).is_ok()
        } else {
            shell::execute_pipeline(commands).is_ok()
        };
    }

    // Step 3: Handle redirections
    let parsed = parse_redirects(&joined_input);
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

    // Step 4: Built-in commands
    if let Some(result) = builtins::handle_command(cmd, args) {
        return if let Err(e) = result {
            eprintln!("{}", e);
            false
        } else {
            true
        };
    }

    // Step 5: External commands
    if background {
        shell::execute_background(cmd, args).is_ok()
    } else {
        shell::execute(cmd, args).is_ok()
    }
}
