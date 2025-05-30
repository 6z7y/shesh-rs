use std::env;

#[derive(Debug, PartialEq)]
pub enum CommandSeparator {
    AndAnd,    // &&
    SemiColon, // ;
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
    let mut parts = Vec::new();
    let mut buffer = String::new();
    let mut in_quote = None;
    let mut escape = false;

    for c in input.chars() {
        if escape {
            buffer.push(c);
            escape = false;
            continue;
        }

        match c {
            '\\' => escape = true,
            '"' | '\'' => {
                if in_quote == Some(c) {
                    in_quote = None;
                } else if in_quote.is_none() {
                    in_quote = Some(c);
                } else {
                    buffer.push(c);
                }
            }
            ' ' if in_quote.is_none() => {
                if !buffer.is_empty() {
                    parts.push(buffer.clone());
                    buffer.clear();
                }
            }
            _ => buffer.push(c),
        }
    }

    if !buffer.is_empty() {
        parts.push(buffer);
    }
    parts
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
            '"' | '\'' => {
                if in_quote == Some(c) {
                    in_quote = None;
                } else if in_quote.is_none() {
                    in_quote = Some(c);
                }
                current.push(c);
            }
            ';' if in_quote.is_none() => {
                tokens.push((current.trim().to_string(), CommandSeparator::SemiColon));
                current.clear();
            }
            '&' if in_quote.is_none() => {
                if let Some('&') = chars.peek() {
                    chars.next();
                    tokens.push((current.trim().to_string(), CommandSeparator::AndAnd));
                    current.clear();
                } else {
                    // إضافة دعم لـ &
                    tokens.push((current.trim().to_string(), CommandSeparator::Background));
                    current.clear();
                }
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
    let mut result = vec![String::new()];
    let mut stack = vec![];

    for c in input.chars() {
        match c {
            '{' => {
                stack.push(result);
                result = vec![String::new()];
            }
            '}' => {
                let parts: Vec<String> = result.iter()
                    .flat_map(|s| s.split(','))
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                
                result = stack.pop().unwrap();
                result = result.iter()
                    .flat_map(|prev| parts.iter().map(move |p| format!("{}{}", prev, p)))
                    .collect();
            }
            _ => {
                for s in &mut result {
                    s.push(c);
                }
            }
        }
    }
    result
}

// fn matches_pattern(name: &str, pattern: &str) -> bool {
//     let pattern = pattern.trim_matches('*');
//     name.starts_with(pattern) || name.ends_with(pattern)
// }
//

pub fn expand_vars(input: &str) -> String {
    let mut result = String::new();
    let mut var_name = String::new();
    let mut in_var = false;

    for c in input.chars() {
        match c {
            '$' if !in_var => in_var = true,
            c if in_var => {
                if c.is_alphanumeric() || c == '_' {
                    var_name.push(c);
                } else {
                    result.push_str(&std::env::var(&var_name).unwrap_or_default());
                    var_name.clear();
                    in_var = false;
                    result.push(c);
                }
            }
            _ => {
                if in_var {
                    result.push_str(&std::env::var(&var_name).unwrap_or_default());
                    var_name.clear();
                    in_var = false;
                }
                result.push(c);
            }
        }
    }

    if in_var {
        result.push_str(&std::env::var(&var_name).unwrap_or_default());
    }

    result
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
    let pattern = pattern.trim();
    let dir = std::path::Path::new(".");
    
    let mut matches = Vec::new();
    if let Ok(entries) = dir.read_dir() {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if wildcard_match(&name, pattern) {
                matches.push(name);
            }
        }
    }
    matches
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
            },
            ">>" => {
                if let Some(file) = parts.next() {
                    redirects.push(Redirect::Append(file.to_string()));
                }
            },
            "<" => {
                if let Some(file) = parts.next() {
                    redirects.push(Redirect::Input(file.to_string()));
                }
            },
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
    input.split('|')
        .map(|part| part.split_whitespace().map(|s| s.to_string()).collect())
        .collect()
}
