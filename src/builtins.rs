<<<<<<< HEAD
use {
    std::{
        collections::HashMap,
        env,
        ffi::CString,
        ptr,
        io,
        sync::{Arc, Mutex, OnceLock}
    },
    libc::{dup2, fork, execvp, waitpid},

    crate::{
        utils::expand_tilde
    }
};
=======
use libc::{dup2, execvp, fork, waitpid};
use std::{
    collections::HashMap,
    env,
    ffi::CString,
    io, ptr,
    sync::{Arc, Mutex, OnceLock},
};

use crate::utils::expand_tilde;
>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f


type BuiltinFn = fn(&[String]) -> io::Result<()>;

static BUILTINS: [(&str, BuiltinFn); 6] = [
    ("24!", |args| handle_24_command(args)),
    ("alias", |args| handle_alias(&args.join(" "))),
    ("cd", |args| cd(&args.iter().map(|s| s.as_str()).collect::<Vec<_>>())),
    ("exit", |_| std::process::exit(0)),
    ("export", |args| handle_export_cmd(args)),
    ("help", |_| { println!("{}", help()); Ok(()) }),
];

pub fn get_builtin(cmd: &str) -> Option<BuiltinFn> {
    BUILTINS.iter()
        .find(|(name, _)| *name == cmd)
        .map(|(_, func)| *func)
}

// Alias storage
static ALIASES: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

// Environment variables storage
pub static ENV_VARS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();

static VIM_MODE: OnceLock<Arc<Mutex<bool>>> = OnceLock::new();

pub fn handle_24_command(args: &[String]) -> io::Result<()> {
    if args.is_empty() {
        println!("24! commands:");
        println!("  vim_keys - Toggle Vim keybindings");
        return Ok(());
    }

    match args[0].as_str() {
        "vim_keys" => {
            let enabled = toggle_vim_mode();
            println!("Vim keys {}", if enabled { "enabled" } else { "disabled" });
            Ok(())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Unknown 24! command",
        )),
    }
}

pub fn toggle_vim_mode() -> bool {
    let mode = VIM_MODE.get_or_init(|| Arc::new(Mutex::new(false)));
    let mut enabled = mode.lock().unwrap();
    *enabled = !*enabled;
    *enabled
}

fn get_aliases() -> &'static Mutex<HashMap<String, String>> {
    ALIASES.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn handle_alias(input: &str) -> io::Result<()> {
    let aliases = get_aliases();
    let mut aliases = aliases.lock().unwrap();

    if input.is_empty() {
        for (name, cmd) in &*aliases {
            println!("alias {name}='{cmd}'");
        }
        return Ok(());
    }

    // Support both formats: name=value and name value
    let parts: Vec<&str> = input.splitn(2, ['=', ' ']).collect(); // Using array instead of closure

    match parts.as_slice() {
        [name, value] => {
            aliases.insert(name.trim().to_string(), value.trim().to_string());
            Ok(())
        }
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Usage: alias name=value",
        )),
    }
}

pub fn expand_aliases(input: &str) -> String {
<<<<<<< HEAD
    if let Some(first_word) = input.split_whitespace().next() {
        if let Some(aliases) = ALIASES.get() {
            if let Some(expanded) = aliases.lock().unwrap().get(first_word) {
                return input.replacen(first_word, expanded, 1);
            }
        }
    }
    input.to_string()
=======
    let Some(first_word) = input.split_whitespace().next() else {
        return input.to_string();
    };

    let Some(aliases) = ALIASES.get() else {
        return input.to_string();
    };

    let aliases = aliases.lock().unwrap();
    aliases
        .get(first_word)
        .map(|expanded| input.replacen(first_word, expanded, 1))
        .unwrap_or_else(|| input.to_string())
>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f
}

pub fn cd(args: &[&str]) -> io::Result<()> {
    let dir = args.first().unwrap_or(&"~");
    let path = expand_tilde(dir);

    env::set_current_dir(&path).map_err(|e| {
        let msg = format!("cd: '{}': {e}", path.display());
        io::Error::other(msg)
    })
}

pub fn help() -> String {
    "
    Available builtins:
    - cd [dir] : Change directory
    - exit     : Exit the shell
    - help     : Show this help"
        .to_string()
}

pub fn execute_external(command: &str, args: &[&str]) -> io::Result<()> {
    // Prepare command and args as C strings
    let cmd_cstr = CString::new(command)?;
    let all_args = std::iter::once(command).chain(args.iter().copied());

    // Convert all arguments to CStrings
    let args_cstr: Vec<CString> = all_args
        .map(|s| {
            CString::new(s).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Invalid argument: {e}"),
                )
            })
        })
        .collect::<Result<_, _>>()?;

    // Build argv array
    let argv: Vec<*const libc::c_char> = args_cstr
        .iter()
        .map(|c| c.as_ptr())
        .chain(std::iter::once(ptr::null()))
        .collect();

    unsafe {
        match fork() {
            0 => {
                // Child process
                libc::signal(libc::SIGINT, libc::SIG_DFL);
                libc::signal(libc::SIGQUIT, libc::SIG_DFL);

                // Redirect stderr to stdout to capture command's own error messages
                dup2(libc::STDOUT_FILENO, libc::STDERR_FILENO);

                execvp(cmd_cstr.as_ptr(), argv.as_ptr());
                // Only reached if execvp fails
                libc::exit(127); // Standard "not found" exit code
            }
            -1 => Err(io::Error::last_os_error()), // Fork failed
            pid => {
                // Parent process
                let mut status = 0;
                waitpid(pid, &mut status, 0);

                if libc::WIFEXITED(status) {
                    match libc::WEXITSTATUS(status) {
                        0 => Ok(()),
                        127 => Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("shesh: '{command}' command not found."),
                        )),
                        _ => Ok(()), // Ignore other exit codes (commands handle their own errors)
                    }
                } else {
                    // Only report signals if they're not part of normal operation
                    if libc::WIFSIGNALED(status) {
                        let sig = libc::WTERMSIG(status);
                        if sig != libc::SIGINT && sig != libc::SIGTERM {
                            return Err(io::Error::new(
                                io::ErrorKind::Interrupted,
                                format!("Command terminated by signal {sig}"),
                            ));
                        }
                    }
                    Ok(())
                }
            }
        }
    }
}

/// Handle export command - condensed version
pub fn handle_export_cmd(args: &[String]) -> io::Result<()> {
    if args.is_empty() {
<<<<<<< HEAD
        use std::collections::BTreeMap;
        
        let mut vars: BTreeMap<String, String> = env::vars().collect();
        if let Some(env_vars) = ENV_VARS.get() {
            vars.extend(env_vars.lock().unwrap().iter().map(|(k,v)| (k.clone(),v.clone())));
        }
        
        for (k, v) in vars {
            println!("{k:<20} {v}");
=======
        // Create compact HashMap to avoid duplication

        let mut vars = HashMap::new();

        // First: System variables
        env::vars().for_each(|(k, v)| {
            vars.insert(k, v);
        });

        // Second: Custom variables (override system variables if they exist)
        if let Some(env_vars) = ENV_VARS.get() {
            env_vars.lock().unwrap().iter().for_each(|(k, v)| {
                vars.insert(k.clone(), v.clone());
            });
        }

        // Conversion and sorting
        let mut sorted_vars: Vec<_> = vars.into_iter().collect();
        sorted_vars.sort_unstable_by_key(|(k, _)| k.clone());

        // Display
        if let Some(max) = sorted_vars.iter().map(|(k, _)| k.len()).max() {
            sorted_vars
                .iter()
                .for_each(|(k, v)| println!("{k:<max$} {v}"));
>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f
        }
    } else {
        args.iter()
            .filter_map(|a| a.split_once('='))
            .for_each(|(k, v)| {
                ENV_VARS
                    .get_or_init(|| Mutex::new(HashMap::new()))
                    .lock()
                    .unwrap()
                    .insert(k.into(), v.into());
                unsafe {
                    env::set_var(k, v);
                }
            });
    }
    Ok(())
}
