use {
    reedline::{
        default_emacs_keybindings, EditCommand, KeyCode,
        KeyModifiers, Keybindings, ReedlineEvent,
        Validator, ValidationResult}, 
    std::{env, path::PathBuf, process::exit}
};

pub fn system_err(msg: &str){
    eprintln!("Shesh: {msg}")
}

pub fn die(msg: &str)-> !{
    eprintln!("Shesh: {msg})");
    exit(1)
}

pub fn expand_tilde(path: &str) -> PathBuf {
    if let Some(stripped) = path.strip_prefix('~') {
        if let Ok(home) = env::var("HOME") {
            if stripped.is_empty() {
                return PathBuf::from(home);
            } else if let Some(rest) = stripped.strip_prefix('/') {
                return PathBuf::from(home).join(rest);
            }
        }
    }
    PathBuf::from(path)
}

// extract env and give me text
pub fn expand_env_vars(input: &str)-> String{
    let mut result = input.to_string();
    for (key, value) in env::vars() {
        result = result.replace(&format!("${key}"), &value);
    }
    result
}

//------
// keys
pub fn emacs_keys_modify()-> Keybindings{
    let mut keybindings = default_emacs_keybindings();
    keybindings.add_binding(
        KeyModifiers::CONTROL,
        KeyCode::Char('c'),
        ReedlineEvent::Edit(vec![EditCommand::Clear]),
    );
    keybindings.add_binding(
        KeyModifiers::NONE,
        KeyCode::Tab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".into()),
            ReedlineEvent::MenuDown,
        ]),
    );
    keybindings.add_binding(
        KeyModifiers::SHIFT,
        KeyCode::BackTab,
        ReedlineEvent::UntilFound(vec![
            ReedlineEvent::Menu("completion_menu".into()),
            ReedlineEvent::MenuUp,
        ]),
    );
    keybindings
}

//-----
// Validation
pub struct CustomValidator;

// For custom validation, implement the Validator trait
impl Validator for CustomValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        if line.ends_with('\\') {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_utils() {
        let result = expand_tilde("~/Documents/projects");
        let user_name = expand_env_vars("$USER");
        assert_eq!(
            result,
            PathBuf::from(format!("/home/{}/Documents/projects", user_name))
        );
    }
}
