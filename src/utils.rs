use termion::color;

pub fn gray(text: &str) -> String {
    format!("{}{}{}", color::Fg(color::LightBlack), text, color::Fg(color::Reset))
}

pub fn green(text: &str) -> String {
    format!("{}{}{}", color::Fg(color::Green), text, color::Fg(color::Reset))
}

// pub fn red(text: &str) -> String {
//     format!("{}{}{}", color::Fg(color::Red), text, color::Fg(color::Reset))
// }
