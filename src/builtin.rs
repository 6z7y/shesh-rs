use crate::commands;
use std::sync::Mutex;
use std::io::Result;

static ALIASES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static PREV_DIR: Mutex<Option<String>> = Mutex::new(None);

/// Adds or updates a new alias
fn set_alias(name: &str, cmd: &str) {
    let mut m = ALIASES.lock().unwrap();
    if let Some((_, v)) = m.iter_mut().find(|(n, _)| n == name) {
        *v = cmd.to_string();
    } else {
        m.push((name.to_string(), cmd.to_string()));
    }
}

/// Returns the value if the alias exists
pub fn lookup_alias(name: &str) -> Option<String> {
    ALIASES
        .lock().unwrap()
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
}

pub fn handle_builtin(cmd: &str, args: &[&str]) -> Option<Result<()>> {
    match cmd {
        "alias" => Some(handle_alias_cmd(args)),
        "cd" => Some(cd(args.first().unwrap_or(&"~"))),
        "exit" => { std::process::exit(0); },
        "export" => Some(handle_export(args)),
        _ => None
    }
}

fn handle_alias_cmd(args: &[&str]) -> Result<()> {
    match args {
        // Without arguments: print all aliases
        [] => {
            for (n, c) in ALIASES.lock().unwrap().iter() {
                println!("alias {}=\"{}\"", n, c);
            }
        }
        // Format: VAR="value"
        [pair] if pair.contains('=') => {
            let mut parts = pair.splitn(2, '=');
            let name = parts.next().unwrap();
            let val = parts.next().unwrap().trim_matches('"');
            set_alias(name, val);
        }
        // Format: VAR "value"
        [name, value] => {
            let val = value.trim_matches('"');
            set_alias(name, val);
        }
        _ => eprintln!("Usage: alias [name=command]"),
    }
    Ok(())
}

pub fn cd(dir: &str) -> Result<()> {
    let mut prev_dir = PREV_DIR.lock().unwrap();
    let current_dir = std::env::current_dir()?
        .to_str()
        .unwrap_or("")
        .to_string();

    let target_dir = if dir == "-" {
        match &*prev_dir {
            Some(d) => d.clone(),
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "[E] No previous directory",
                ))
            }
        }
    } else {
        commands::expand_tilde(dir)
    };

    // Check if target is a directory
    if !std::path::Path::new(&target_dir).is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("[E] Not a directory: {}", target_dir)
        ));
    }

    std::env::set_current_dir(&target_dir)?;
    
    // Update previous directory
    *prev_dir = Some(current_dir);
    Ok(())
}

fn handle_export(args: &[&str]) -> Result<()> {
    if let Some(arg) = args.first() {
        if let Some((var, value)) = arg.split_once('=') {
            println!("Exported: {}={}", var, value);
            return Ok(());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Usage: export VAR=value"
    ))
}
