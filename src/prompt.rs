use {
    std::{env,borrow::Cow},
    reedline::{Prompt, PromptEditMode, PromptHistorySearch, PromptViMode},
    crate::utils::expand_env_vars
};

pub struct PromptSystem {
    custom_prompt: Option<String>,
}

impl PromptSystem {
    pub fn new(custom_prompt: Option<String>)-> Self{
        Self { custom_prompt }
    }
}

impl Prompt for PromptSystem {
    fn render_prompt_left(&self)-> std::borrow::Cow<'static, str>{
        if let Some(prompt) = &self.custom_prompt {
            return Cow::Owned(expand_env_vars(prompt));
        }

        let path = env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or("no path".into());

        let homedir = env::var("HOME").unwrap_or_default();
        let short_path = path.replace(&homedir, "~");

        let segments: Vec<&str> = short_path.split('/').filter(|s| !s.is_empty()).collect();
        let len = segments.len();

        let base_prompt = if segments.is_empty() {
            if short_path.starts_with('/') { "/ " } else { "/ " }.to_string()
        } else {
            let start = if short_path.starts_with('/') { "/" } else { "" };
            let shortened = segments.iter().enumerate()
                .fold(String::new(), |mut acc, (i, seg)| {
                    if i > 0 {
                        acc.push('/');
                    }
                    if i == len - 1 {
                        acc.push_str(seg);
                    } else if seg.starts_with('.') {
                        acc.push_str(&seg[..2]);
                    } else {
                        acc.push(seg.chars().next().unwrap_or(' '));
                    }
                    acc
                });
            format!("\x1b[32m{start}{shortened}\x1b[0m ")
        };

        std::borrow::Cow::Owned(base_prompt)
    }

    fn render_prompt_right(&self)-> std::borrow::Cow<'static, str>{
        std::borrow::Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, edit_mode: PromptEditMode)-> std::borrow::Cow<'static, str>{
        match edit_mode {
            PromptEditMode::Vi(PromptViMode::Normal) => {
                print!("\x1b[0 q"); // Reset cursor to default shape
                std::borrow::Cow::Borrowed("\x1b[33m[N]\x1b[0m ")
            }
            PromptEditMode::Vi(PromptViMode::Insert) => {
                print!("\x1b[6 q"); // Vertical cursor shape (|) for Insert mode
                std::borrow::Cow::Borrowed("\x1b[32m[I]\x1b[0m ")
            }
            _ => std::borrow::Cow::Borrowed(""),
        }
    }

    fn render_prompt_multiline_indicator(&self)-> std::borrow::Cow<'static, str>{
        std::borrow::Cow::Borrowed("")
    }

    fn render_prompt_history_search_indicator(&self,_history_search: PromptHistorySearch)-> std::borrow::Cow<'static, str>{
        std::borrow::Cow::Borrowed("[Search]: ")
    }
}
