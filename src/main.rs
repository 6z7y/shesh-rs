mod builtins;
mod completions;
mod config;
mod parse;
mod process_exec;
mod prompt;
mod shell;
mod utils;

<<<<<<< HEAD
use {
    std::{time::{Duration}, thread::sleep},
    libc::{signal, SIGINT, SIGQUIT, SIG_IGN},
    reedline::{ColumnarMenu, DefaultHinter, Emacs, FileBackedHistory, MenuBuilder, Reedline, ReedlineMenu, Signal},
    nu_ansi_term::{Color, Style},
    crate::{
        completions::create_default_completer,
        prompt::PromptSystem,
        utils::{emacs_keys_modify, CustomValidator}
    }
};

fn main() {
=======
use nu_ansi_term::{Color, Style};
use reedline::{
    ColumnarMenu, DefaultHinter, EditCommand, Emacs, FileBackedHistory, KeyCode, KeyModifiers,
    MenuBuilder, Reedline, ReedlineEvent, ReedlineMenu, Signal, Vi, default_emacs_keybindings,
};

use crate::{completions::create_default_completer, prompt::PromptSystem};

fn main() {
    // Initialize VIM_MODE
    builtins::init_vim_mode();

>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f
    // [1] Load configuration and run startup script
    let cfg = config::init_config();
    config::run_startup(&cfg);

    // [2] Initialize prompt style
    let prompt = PromptSystem::new(cfg.prompt);

    // [3] Set up command history with file persistence
    let history = Box::new(FileBackedHistory::with_file(6000, config::history_file_path()).unwrap_or_default());

    // [4] Set up auto-completion
    let completer = create_default_completer();

    let menu = ReedlineMenu::EngineCompleter(Box::new(
        ColumnarMenu::default().with_name("completion_menu").with_columns(3).with_column_width(Some(60)),
    ));

    // [5] Configure keybindings for Emacs mode
    let keybindings = emacs_keys_modify();

    // [6] Build the line editor
    let mut line_editor = Reedline::create()
        .with_history(history)
        .with_completer(completer)
        .with_menu(menu)
<<<<<<< HEAD
        .with_edit_mode(Box::new(Emacs::new(keybindings)))
        .with_validator(Box::new(CustomValidator))
        .with_hinter(Box::new(DefaultHinter::default()
            .with_style(Style::new().underline().italic().fg(Color::Rgb(120, 120, 120)))
            .with_min_chars(1),
        )
    );
=======
        .with_hinter(Box::new(
            DefaultHinter::default()
                .with_style(
                    Style::new()
                        .underline()
                        .italic()
                        .fg(Color::Rgb(120, 120, 120)),
                )
                .with_min_chars(1),
        ))
        .with_edit_mode(Box::new(Emacs::new(keybindings)));
>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f

    unsafe {
        signal(SIGINT, SIG_IGN);
        signal(SIGQUIT, SIG_IGN);
    }

    // [7] Main REPL loop
    loop {
        match line_editor.read_line(&prompt) {
            Ok(Signal::Success(buf)) if !buf.is_empty() => {
                config::append_to_history(&buf);
<<<<<<< HEAD
=======

                if buf.trim() == "24! vim_keys" {
                    let enabled = builtins::toggle_vim_mode();
                    println!("Vim keys {}", if enabled { "enabled" } else { "disabled" });

                    editor = editor.with_edit_mode(if enabled {
                        Box::new(Vi::default())
                    } else {
                        Box::new(Emacs::new(default_emacs_keybindings()))
                    });
                }

>>>>>>> 950830b84f455dbe5ae6ec262cb26bf882e18b2f
                if let Err(e) = shell::exec(&buf) {
                    eprintln!("{e}");
                }
            }
            Ok(Signal::Success(_)) => continue,
            Ok(Signal::CtrlD) => break,
            _ => eprintln!("Reedline error"),
        }
        sleep(Duration::from_millis(10))
    }
}
