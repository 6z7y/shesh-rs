mod builtins;
mod commands;
mod config;
mod input;
mod shell;
mod utils;

use std::io::Result;
use commands::CommandSeparator;

fn main() -> Result<()> { 
    let config = config::init();
    config::run_startup(&config);

    loop {
        let input = input::read_line_raw(&config::load_history());
        let input = input.trim().split('#').next().unwrap().trim();
        if input.is_empty() { continue; }

        // Expand aliases
        let expanded_line = builtins::expand_aliases(input);
        config::save_history(&expanded_line);

        // Split commands
        let tokens = commands::split_commands(&expanded_line);
        let mut last_success = true;
        let mut background_next = false;

        for (cmd_str, separator) in tokens {
            let background = background_next;
            background_next = false;

            match separator {
                CommandSeparator::AndAnd if !last_success => {
                    // Skip this command because previous failed
                    continue;
                },
                CommandSeparator::Background => {
                    background_next = true;
                    // Continue to next command without executing this one as foreground
                    continue;
                },
                _ => {}
            }

            // Execute the command
            last_success = commands::process_command(&cmd_str, background);
        }
    }
}
