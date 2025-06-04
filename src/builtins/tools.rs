use std::io::Result;
use std::sync::Mutex;
use crate::commands;

static ALIASES: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());
static PREV_DIR: Mutex<Option<String>> = Mutex::new(None);

pub fn set_alias(name: &str, cmd: &str) {
    let mut m = ALIASES.lock().unwrap();
    if let Some((_, v)) = m.iter_mut().find(|(n, _)| n == name) {
        *v = cmd.to_string();
    } else {
        m.push((name.to_string(), cmd.to_string()));
    }
}

pub fn lookup_alias(name: &str) -> Option<String> {
    ALIASES
        .lock().unwrap()
        .iter()
        .find(|(n, _)| n == name)
        .map(|(_, v)| v.clone())
}

pub fn handle_alias_cmd(args: &[&str]) -> Result<()> {
    match args {
        [] => {
            for (n, c) in ALIASES.lock().unwrap().iter() {
                println!("alias {}=\"{}\"", n, c);
            }
            Ok(())
        }
        [pair] if pair.contains('=') => {
            let mut parts = pair.splitn(2, '=');
            if let (Some(name), Some(val)) = (parts.next(), parts.next()) {
                set_alias(name, val.trim_matches('"'));
            }
            Ok(())
        }
        [name, value] => {
            set_alias(name, value.trim_matches('"'));
            Ok(())
        }
        _ => {
            eprintln!("Usage: alias [name=command]");
            Ok(())
        }
    }
}

pub fn change_directory(dir: &str) -> Result<()> {
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
                    "No previous directory",
                ))
            }
        }
    } else {
        commands::expand_tilde(dir)
    };

    if !std::path::Path::new(&target_dir).is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Not a directory: {}", target_dir)
        ));
    }

    std::env::set_current_dir(&target_dir)?;
    *prev_dir = Some(current_dir);
    Ok(())
}

// pub fn handle_export(args: &[&str]) -> Result<()> {
//     if let Some(arg) = args.first() {
//         if let Some((var, value)) = arg.split_once('=') {
//             // Actually set the environment variable
//             std::env::set_var(var, value);
//             return Ok(());
//         }
//     }
//     Err(std::io::Error::new(
//         std::io::ErrorKind::InvalidInput,
//         "Usage: export VAR=value"
//     ))
// }

pub fn handle_export(args: &[&str]) -> Result<()> {
    if let Some(arg) = args.first() {
        if let Some((var, value)) = arg.split_once('=') {
            // Just acknowledge the export command without actually setting the variable
            println!("export {}={}", var, value);
            return Ok(());
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Usage: export VAR=value"
    ))
}
