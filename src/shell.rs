use std::{
    fs::{File, OpenOptions},
    io::Result,
    process::{Command, Stdio},
    thread,
};
use crate::commands::Redirect;

pub fn execute(cmd: &str, args: &[&str]) -> Result<()> {
    let expanded_args: Vec<&str> = args.iter()
        .flat_map(|arg| arg.split(','))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    
    Command::new(cmd)
        .args(expanded_args)
        .status()
        .map_err(|e| {
            // Custom error message for command not found
            if e.kind() == std::io::ErrorKind::NotFound {
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("shesh: command not found: {}", cmd)
                )
            } else {
                e
            }
        })
        .map(|_| ())
}

pub fn execute_background(cmd: &str, args: &[&str]) -> Result<()> {
    let cmd = cmd.to_string();
    let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    
    thread::spawn(move || {
        let _ = Command::new(&cmd)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    });
    
    Ok(())
}

pub fn execute_with_redirect(cmd: &[String], redirects: &[Redirect]) -> Result<()> {
    let (program, args) = cmd.split_first().unwrap();
    let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
    let mut command = Command::new(program);
    
    for redirect in redirects {
        match redirect {
            Redirect::Output(file) => {
                command.stdout(File::create(file)?);
            }
            Redirect::Append(file) => {
                command.stdout(OpenOptions::new().append(true).create(true).open(file)?);
            }
            Redirect::Input(file) => {
                command.stdin(File::open(file)?);
            }
        }
    }
    
    command.args(args).status().map(|_| ())
}

pub fn execute_background_with_redirect(cmd: &[String], redirects: &[Redirect]) -> Result<()> {
    let cmd = cmd.to_vec();
    let redirects = redirects.to_vec();
    
    thread::spawn(move || {
        let _ = execute_with_redirect(&cmd, &redirects);
    });
    
    Ok(())
}

pub fn execute_pipeline(commands: Vec<Vec<String>>) -> Result<()> {
    let mut previous_output = None;
    
    for (i, cmd_parts) in commands.iter().enumerate() {
        let (cmd, args) = cmd_parts.split_first().unwrap();
        let args: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let mut command = Command::new(cmd);
        command.args(args);
        
        if let Some(output) = previous_output.take() {
            command.stdin(output);
        }
        
        if i < commands.len() - 1 {
            let child = command.stdout(Stdio::piped()).spawn()?;
            previous_output = child.stdout;
        } else {
            command.status()?;
        }
    }
    Ok(())
}

pub fn execute_background_pipeline(commands: Vec<Vec<String>>) -> Result<()> {
    thread::spawn(move || {
        let _ = execute_pipeline(commands);
    });
    Ok(())
}
