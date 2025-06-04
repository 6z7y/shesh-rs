use std::env;
use std::io::{stdin, stdout, Write};
use std::io::Stdout;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::terminal_size;

use crate::utils::{green, gray};
use crate::config;
use crate::commands;

/// Represents the state of the line editor
struct EditorState {
    input: String,              // Current input line
    cursor: usize,              // Current cursor position
    history: Vec<String>,       // Command history
    history_index: Option<usize>, // Current position in history
    temp_input: String,         // Temporary storage when browsing history
    matched_hint: bool,         // Flag for matched hint state
    prompt: String,             // Custom prompt from config
    completions: Vec<String>,   // List of possible completions
    completion_index: usize,    // Current index in completions list
    show_completions: bool,     // Flag to show completions list
}

impl EditorState {
    /// Create a new editor state
    fn new(history: &[String], prompt: String) -> Self {
        Self {
            input: String::new(),
            cursor: 0,
            history: history.to_vec(),
            history_index: None,
            temp_input: String::new(),
            matched_hint: false,
            prompt,
            completions: Vec::new(),
            completion_index: 0,
            show_completions: false,
        }
    }

    /// Get hint from history based on current input
    fn get_hint(&self) -> Option<String> {
        if self.input.is_empty() {
            None
        } else {
            self.history
                .iter()
                .rev()
                .find(|cmd| cmd.starts_with(&self.input) && *cmd != &self.input)
                .cloned()
        }
    }

    /// Redraw the editor interface
    fn redraw(&mut self, stdout: &mut RawTerminal<Stdout>) {
        // Get terminal width
        let (width, _) = terminal_size().unwrap_or((80, 24));
        
        // Clear current line and print prompt
        write!(stdout, "\r{}{}", termion::clear::CurrentLine, self.prompt).unwrap();
        
        // Print input text
        write!(stdout, "{}", self.input).unwrap();
        
        // Get hint if available
        let hint = self.get_hint();
        
        // Display hint if not showing completions
        if !self.show_completions {
            if let Some(hint) = hint.as_deref() {
                if hint.starts_with(&self.input) {
                    let remaining = &hint[self.input.len()..];
                    let colored = if self.matched_hint {
                        green(remaining)
                    } else {
                        gray(remaining)
                    };
                    write!(stdout, "{}", colored).unwrap();
                }
            }
        }
        
        // Calculate cursor position
        let prompt_len = self.prompt.chars().count();
        let cursor_pos = prompt_len + self.input[..self.cursor].chars().count();
        write!(stdout, "\r{}", termion::cursor::Right(cursor_pos as u16)).unwrap();
        
        // Show completions list if needed
        if self.show_completions && !self.completions.is_empty() {
            write!(stdout, "\r\n").unwrap();
            
            // Limit to 100 items for performance
            let display_items: Vec<_> = self.completions.iter().take(100).collect();
            
            // Calculate how many items fit per row
            let max_len = display_items.iter().map(|s| s.len()).max().unwrap_or(0) + 2;
            let items_per_row = std::cmp::max(1, (width as usize) / max_len); // Ensure at least 1 item per row
            
            for (i, item) in display_items.iter().enumerate() {
                // Highlight current completion
                if i == self.completion_index {
                    write!(stdout, "{}", green(item)).unwrap();
                } else {
                    write!(stdout, "{}", item).unwrap();
                }
                
                // Add spacing between items
                let spaces = " ".repeat(max_len - item.len());
                write!(stdout, "{}", spaces).unwrap();
                
                // Start new row when needed
                if (i + 1) % items_per_row == 0 {
                    write!(stdout, "\r\n").unwrap();
                }
            }
            
            // Move cursor back to input position
            write!(
                stdout, 
                "\r\n{}{}{}", 
                termion::clear::CurrentLine, 
                self.prompt, 
                self.input
            ).unwrap();
        }
        
        stdout.flush().unwrap();
    }

    /// Move cursor left
    fn move_cursor_left(&self) -> usize {
        if self.cursor == 0 {
            return 0;
        }
        let mut new_cursor = self.cursor - 1;
        while !self.input.is_char_boundary(new_cursor) {
            new_cursor -= 1;
        }
        new_cursor
    }

    /// Move cursor right
    fn move_cursor_right(&self) -> usize {
        if self.cursor >= self.input.len() {
            return self.input.len();
        }
        let mut new_cursor = self.cursor + 1;
        while new_cursor < self.input.len() && !self.input.is_char_boundary(new_cursor) {
            new_cursor += 1;
        }
        new_cursor
    }
    
    /// Generate completions based on current input
    fn update_completions(&mut self) {
        self.completions.clear();
        
        let parts: Vec<&str> = self.input.split_whitespace().collect();
        let last_part = parts.last().unwrap_or(&"");
        
        // تحديد نوع الإكمال بناءً على السياق
        if self.input.ends_with(' ') {
            // مسافة بعد الأمر: إكمال ملفات
            self.completions = commands::complete_path("");
        } else if parts.is_empty() {
            // لا يوجد إدخال: لا شيء
        } else if parts.len() == 1 {
            // أمر واحد فقط: إكمال أوامر
            self.completions = commands::complete_command(last_part);
        } else {
            // وسائط: إكمال ملفات
            self.completions = commands::complete_path(last_part);
        }
        
        // معالجة خاصة لمسار ~
        self.completions = self.completions.iter()
            .map(|p| p.replace(&env::var("HOME").unwrap(), "~"))
            .collect();
    }
}

/// Read a line with advanced editing capabilities
pub fn read_line_raw(history: &[String]) -> String {
    let config = config::init();
    let prompt = config.prompt.clone();
    
    let stdin = stdin();
    let mut stdin = stdin.lock().keys();
    let mut stdout = stdout().into_raw_mode().unwrap();
    
    let mut state = EditorState::new(history, prompt);
    state.redraw(&mut stdout);

    loop {
        let evt = stdin.next().unwrap().unwrap();
        
        match evt {
            Key::Char('\n') => {
                // Clear completions and hints
                if state.show_completions {
                    state.show_completions = false;
                    state.completions.clear();
                }
                write!(stdout, "\r\n").unwrap();
                break;
            }
            Key::Char('\t') => {
                // First tab: generate completions
                if !state.show_completions {
                    state.update_completions();
                    state.show_completions = !state.completions.is_empty();
                } 
                // Subsequent tabs: cycle through completions
                else if !state.completions.is_empty() {
                    state.completion_index = (state.completion_index + 1) % state.completions.len();
                }
            }
            Key::Char(c) => {
                state.show_completions = false;
                state.input.insert(state.cursor, c);
                state.cursor = state.move_cursor_right();
                state.matched_hint = false;
            }
            Key::Backspace => {
                if state.cursor > 0 {
                    let prev = state.move_cursor_left();
                    state.input.replace_range(prev..state.cursor, "");
                    state.cursor = prev;
                    state.matched_hint = false;
                    state.show_completions = false;
                }
            }
            Key::Left => {
                state.cursor = state.move_cursor_left();
                state.matched_hint = false;
                state.show_completions = false;
            }
            Key::Right => {
                if state.cursor == state.input.len() {
                    if let Some(h) = state.get_hint() {
                        if h.starts_with(&state.input) {
                            state.input = h.clone();
                            state.cursor = state.input.len();
                            state.matched_hint = true;
                        }
                    }
                } else {
                    state.cursor = state.move_cursor_right();
                }
                state.show_completions = false;
            }
            Key::Up => {
                state.show_completions = false;
                
                if state.history.is_empty() {
                    continue;
                }
                
                if state.history_index.is_none() {
                    state.temp_input = state.input.clone();
                    state.history_index = Some(state.history.len().saturating_sub(1));
                } else if let Some(i) = state.history_index {
                    if i > 0 {
                        state.history_index = Some(i - 1);
                    }
                }
                
                if let Some(i) = state.history_index {
                    state.input = state.history[i].clone();
                    state.cursor = state.input.len();
                    state.matched_hint = false;
                }
            }
            Key::Down => {
                state.show_completions = false;
                
                if let Some(i) = state.history_index {
                    if i + 1 < state.history.len() {
                        state.history_index = Some(i + 1);
                        state.input = state.history[i + 1].clone();
                    } else {
                        state.history_index = None;
                        state.input = state.temp_input.clone();
                    }
                    state.cursor = state.input.len();
                    state.matched_hint = false;
                }
            }
            Key::Ctrl('c') => {
                write!(stdout, "\r\n").unwrap();
                state.input.clear();
                break;
            }
            _ => {}
        }
        
        state.redraw(&mut stdout);
    }

    state.input
}
