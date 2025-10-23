use std::{env, fs};

// AST (Abstract Syntax Tree) representation of commands
#[derive(Debug, Clone)]
pub enum ParsedCommand {
    Single(Vec<String>),  // Simple command (e.g., "ls -l")
    BinaryOp(Box<ParsedCommand>, Operator, Box<ParsedCommand>),  // Compound command with operator
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RedirectType {
    Stdout,       // >
    StdoutAppend, // >>
    Stderr,       // 2>
    StderrAppend, // 2>>
    Both,         // &>
    BothAppend,   // &>>
    Stdin,        // <
}

// Define supported shell operators
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operator {
    Redirect(RedirectType), // RedirectOut, AppendOut, RedirectIn
    And,         // && (logical AND)
    Or,          // || (logical OR)
    Pipe,        // | (pipe)
    Seq,         // ; (sequential)
    Background,  // & (background process)
}

static OPERATORS: &[(&str, Operator)] = &[
    ("&>>", Operator::Redirect(RedirectType::BothAppend)),
    ("&>", Operator::Redirect(RedirectType::Both)),
    ("2>>", Operator::Redirect(RedirectType::StderrAppend)),
    ("2>", Operator::Redirect(RedirectType::Stderr)),
    (">>", Operator::Redirect(RedirectType::StdoutAppend)),
    (">", Operator::Redirect(RedirectType::Stdout)),
    ("<", Operator::Redirect(RedirectType::Stdin)),
    (";", Operator::Seq),
    ("&&", Operator::And),
    ("||", Operator::Or),
    ("|", Operator::Pipe),
    ("&", Operator::Background),
];

// Main parsing function - entry point
pub fn parse_syntax(input: &str)-> ParsedCommand{
    // If the input is a single operator from OPERATORS
    if OPERATORS.iter().any(|(op, _)| input == *op) {
        return ParsedCommand::Single(vec![]); // Empty list
    }

    // The rest remains the same
    OPERATORS.iter().find_map(|(op_str, op_enum)| {
        find_outside_quotes(input, op_str).map(|index| {
            let (left, right_with_op) = input.split_at(index);
            let right = &right_with_op[op_str.len()..];
            ParsedCommand::BinaryOp(
                Box::new(parse_syntax(left)),
                *op_enum,
                Box::new(parse_syntax(right))
            )
        })
    }).unwrap_or_else(|| ParsedCommand::Single(tokenize(input)))
}

// Finds operator occurrences outside quoted strings
fn find_outside_quotes(input: &str, target: &str) -> Option<usize> {
    let mut in_quotes = None;
    let first_char = target.chars().next()?;
    let chars = input.char_indices();
    
    for (i, c) in chars {
        match c {
            '"' | '\'' => {
                if in_quotes.take() != Some(c) {
                    in_quotes = Some(c);
                }
            },
            _ if in_quotes.is_none() && c == first_char && input[i..].starts_with(target) => {
                return Some(i);
            },
            _ => {}
        }
    }
    None
}

// Splits command into tokens while respecting quotes
fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    let mut found_comment = false;

    while let Some(c) = chars.next() {
        if found_comment {
            continue; // Ignore everything after #
        }
        
        match c {
            '\\' => {
                if let Some(next_char) = chars.next() {
                    current.push(next_char);
                }
            },
            '"' if !in_single => in_double = !in_double,
            '\'' if !in_double => in_single = !in_single,
            '#' if !in_single && !in_double => {
                found_comment = true;
            },
            ' ' if !in_single && !in_double => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            },
            _ => current.push(c),
        }
    }

    if !current.is_empty() && !found_comment {
        tokens.push(current);
    }

    tokens
}

// Processes tokens by expanding variables and wildcards
pub fn process_tokens(cmd: ParsedCommand) -> Vec<String> {
    match cmd {
        ParsedCommand::Single(parts) => {
            let mut result = Vec::with_capacity(parts.len());
            for part in parts {
                match part {
                    _ if part.starts_with('$') => {
                        result.push(env::var(&part[1..]).unwrap_or_default());
                    },
                    _ if part.contains('*') => {
                        // Handle directory/* pattern
                        if let Some(slash_pos) = part.rfind('/') {
                            let (dir, pattern) = part.split_at(slash_pos + 1);
                            if pattern == "*" {
                                if let Ok(entries) = fs::read_dir(dir) {
                                    for entry in entries.flatten() {
                                        let filename = entry.file_name().to_string_lossy().into_owned();
                                        result.push(format!("{dir}{filename}"));
                                    }
                                    continue;
                                }
                            }
                        }
                        // Handle simple * in current directory
                        else if part == "*" {
                            if let Ok(entries) = fs::read_dir(".") {
                                for entry in entries.flatten() {
                                    let filename = entry.file_name().to_string_lossy().into_owned();
                                    result.push(filename);
                                }
                                continue;
                            }
                        }
                        // If we get here, pass the original pattern
                        result.push(part);
                    }
                    _ if part.starts_with('~') => {
                        if let Some(home) = env::var_os("HOME") {
                            result.push(format!("{}{}", home.to_string_lossy(), &part[1..]));
                        } else {
                            result.push(part);
                        }
                    },
                    _ if part.contains('{') && part.contains('}') => {
                        if let Some((start, end)) = part.find('{').zip(part.find('}')) {
                            let expanded = part[start+1..end].split(',')
                                .flat_map(|opt| {
                                    let new = format!("{}{}{}", &part[..start], opt, &part[end+1..]);
                                    process_tokens(ParsedCommand::Single(vec![new]))
                                });
                            result.extend(expanded);
                            continue;
                        }
                        result.push(part);
                    },
                    _ => result.push(part),
                }
            }
            result
        }
        _ => vec!["[complex command not handled yet]".into()],
    }
}
