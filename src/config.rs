use std::{env, fs, io::Write, path::PathBuf};

pub struct Config {
    pub prompt: String,
    pub startup: Vec<String>,
}

impl Config {
    fn default() -> Self {
        Self {
            prompt: "shesh> ".to_string(),
            startup: Vec::new(),
        }
    }
}

pub fn init() -> Config {
    let config_path = get_config_path();
    ensure_config_dirs(&config_path);
    
    if !config_path.exists() {
        create_default_config(&config_path);
    }
    
    load_config(&config_path)
}

fn get_config_path() -> PathBuf {
    get_home_dir().join(".config/shesh/shesh.24")
}

fn get_home_dir() -> PathBuf {
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            eprintln!("Warning: HOME not set, using current directory");
            env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        })
}

fn ensure_config_dirs(config_path: &std::path::Path) {
    if let Some(parent) = config_path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
}

fn create_default_config(config_path: &PathBuf) {
    let default_content = "prompt = \"shesh> \"\n#startup\necho \"shesh ready!\"";
    let _ = fs::write(config_path, default_content);
}

fn load_config(path: &PathBuf) -> Config {
    let mut config = Config::default();
    
    let Ok(content) = fs::read_to_string(path) else {
        return config;
    };

    let mut in_startup = false;
    
    for line in content.lines() {
        let trimmed = line.trim();
        
        if trimmed.is_empty() {
            continue;
        }
        
        if let Some(comment) = trimmed.strip_prefix('#') {
            if comment.trim().eq_ignore_ascii_case("startup") {
                in_startup = true;
            }
            continue;
        }
        
        if in_startup {
            config.startup.push(trimmed.to_string());
        } else if let Some((key, value)) = trimmed.split_once('=') {
            if key.trim() == "prompt" {
                config.prompt = value.trim().trim_matches('"').to_string();
            }
        }
    }
    config
}

pub fn save_history(cmd: &str) {
    let history_path = get_home_dir().join(".local/share/shesh/history");
    if let Some(parent) = history_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    if let Ok(mut file) = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(history_path) 
    {
        let _ = writeln!(file, "{}", cmd);
    }
}

pub fn load_history() -> Vec<String> {
    let history_path = get_home_dir().join(".local/share/shesh/history");
    if let Ok(content) = fs::read_to_string(history_path) {
        content.lines().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

pub fn run_startup(config: &Config) {
    use crate::{commands, shell, builtins};
    
    for cmd_line in &config.startup {
        let parts = commands::parse_input(cmd_line);
        if let Some((cmd, args_vec)) = parts.split_first() {
            let args: Vec<&str> = args_vec.iter().map(|s| s.as_str()).collect();
            
            if let Some(res) = builtins::handle_command(cmd, &args) {
                if let Err(e) = res {
                    eprintln!("Startup builtin failed: {}", e);
                }
            } else if let Err(e) = shell::execute(cmd, &args) {
                eprintln!("Startup command failed: {}", e);
            }
        }
    }
}
