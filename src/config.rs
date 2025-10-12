use {
    std::{
        env,
        fs::{self, create_dir_all, OpenOptions},
        io::Write,
        path::{PathBuf}
    },
    crate::{
        shell::exec,
        utils::die
    }
};


pub struct Config {
    pub prompt: Option<String>,
    pub startup: Vec<String>,
}

impl Default for Config {
    fn default()-> Self{
        Self {
            prompt: None,
            startup: vec![],
        }
    }
}

fn get_home()-> PathBuf{
    env::var("HOME").map(PathBuf::from).unwrap_or_else(|_| die("GET_HOME", "can't find the home dir"))
}

fn config_file_path() -> PathBuf {
    get_home().join(".config/shesh/shesh.24")
}

pub fn history_file_path() -> PathBuf {
    get_home().join(".local/share/shesh/history")
}

//config file
pub fn init_config() -> Config {
    let config_path = config_file_path();

    if let Some(parent) = config_path.parent() {
        let _ = create_dir_all(parent);
    }

    if !config_path.exists() {
        fs::write(&config_path, "#prompt = \"shesh> \"\necho \"shesh ready!\"",)
        .unwrap_or_else(|_| die("CONFIG", "Unable to creat config file"))
    }

    parse_config(&fs::read_to_string(&config_path).unwrap_or_else(|_| die("CONFIG", "Unable to load a config file")))
}

fn parse_config(content: &str)-> Config{
    let mut config = Config::default();

    for linee in content.lines() {
        let line = linee.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        if let Some((key, value)) = line.split_once('=') {
            if key.trim() == "prompt" {
                config.prompt = Some(value.trim().trim_matches('"').to_string());
                continue;
            }
        }
        config.startup.push(line.to_string());
    }
    config
}

pub fn run_startup(config: &Config) {
    for cmd in &config.startup {
        if let Err(e) = exec(cmd) {
            eprintln!("[X] Startup failed: {e}");
        }
    }
}

//history file
pub fn append_to_history(command: &str) {
    let path = history_file_path();

    if !path.exists() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        } else {
            eprintln!("[X] HISTORY_error to make history folder")
        }
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .unwrap();
    
    writeln!(file, "{}", command).unwrap();
}
