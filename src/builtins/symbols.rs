use std::io::Result;
use crate::{commands, shell};

pub fn handle_symbol(symbol: &str, args: &[&str]) -> Result<()> {
    match symbol {
        ">" | ">>" | "<" => handle_redirect(symbol, args),
        "|" => handle_pipeline(args),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Unsupported symbol: {}", symbol)
        )),
    }
}

fn handle_redirect(symbol: &str, args: &[&str]) -> Result<()> {
    let mut parts = args.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    parts.insert(0, symbol.to_string());
    
    let parsed = commands::parse_redirects(&parts.join(" "));
    
    if parsed.redirects.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid redirect syntax"
        ));
    }
    
    shell::execute_with_redirect(&parsed.cmd, &parsed.redirects)
}

fn handle_pipeline(args: &[&str]) -> Result<()> {
    let commands = commands::parse_pipeline(&args.join(" "));
    shell::execute_pipeline(commands)
}
