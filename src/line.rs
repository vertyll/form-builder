//! The editing of one line typed at the terminal, apart from the terminal itself: the text, the cursor in it, and
//! what each key does to them.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// What a key did to a line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Edit {
    /// The line changed, or the cursor moved, and is still being typed.
    Typing,
    /// Enter: the line as typed.
    Done(String),
    /// Esc or Ctrl+C: the line is left.
    Escaped,
    /// Ctrl+D on an empty line: the input has ended.
    Ended,
}

/// A line being typed, and the cursor in it, counted in characters.
#[derive(Debug, Default)]
pub(crate) struct Line {
    chars: Vec<char>,
    cursor: usize,
}

impl Line {
    pub(crate) fn text(&self) -> String {
        self.chars.iter().collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.chars.len()
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    /// Applies `key`: typing inserts at the cursor, the arrows, Home and End move it, Backspace and Delete remove a
    /// character around it, and Ctrl+U clears what is before it.
    pub(crate) fn apply(&mut self, key: KeyEvent) -> Edit {
        let control = key.modifiers.contains(KeyModifiers::CONTROL);
        match key.code {
            KeyCode::Enter => return Edit::Done(self.text()),
            KeyCode::Esc => return Edit::Escaped,
            KeyCode::Char('c') if control => return Edit::Escaped,
            KeyCode::Char('d') if control && self.chars.is_empty() => return Edit::Ended,
            KeyCode::Char('a') if control => self.cursor = 0,
            KeyCode::Char('e') if control => self.cursor = self.chars.len(),
            KeyCode::Char('u') if control => {
                self.chars.drain(..self.cursor);
                self.cursor = 0;
            }
            KeyCode::Char(c) if !control && !key.modifiers.contains(KeyModifiers::ALT) => {
                self.chars.insert(self.cursor, c);
                self.cursor += 1;
            }
            KeyCode::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.chars.remove(self.cursor);
            }
            KeyCode::Delete if self.cursor < self.chars.len() => {
                self.chars.remove(self.cursor);
            }
            KeyCode::Left => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Right => self.cursor = (self.cursor + 1).min(self.chars.len()),
            KeyCode::Home => self.cursor = 0,
            KeyCode::End => self.cursor = self.chars.len(),
            _ => {}
        }
        Edit::Typing
    }
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{Edit, Line};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn control(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    fn typed(line: &mut Line, text: &str) {
        for c in text.chars() {
            line.apply(key(KeyCode::Char(c)));
        }
    }

    #[test]
    fn typing_inserts_at_the_cursor() {
        let mut line = Line::default();
        typed(&mut line, "Ann");
        line.apply(key(KeyCode::Left));
        line.apply(key(KeyCode::Left));
        typed(&mut line, "x");
        assert_eq!((line.text(), line.cursor()), ("Axnn".to_owned(), 2));
        assert_eq!(line.apply(key(KeyCode::Enter)), Edit::Done("Axnn".to_owned()));
    }

    #[test]
    fn backspace_and_delete_remove_around_the_cursor() {
        let mut line = Line::default();
        typed(&mut line, "żółw");
        line.apply(key(KeyCode::Backspace));
        line.apply(key(KeyCode::Home));
        line.apply(key(KeyCode::Delete));
        assert_eq!(line.text(), "ół");
        line.apply(key(KeyCode::Backspace));
        assert_eq!(line.text(), "ół", "nothing before the cursor to remove");
    }

    #[test]
    fn the_cursor_stays_within_the_line() {
        let mut line = Line::default();
        typed(&mut line, "ab");
        for _ in 0..5 {
            line.apply(key(KeyCode::Right));
        }
        assert_eq!(line.cursor(), 2);
        line.apply(control('a'));
        assert_eq!(line.cursor(), 0);
        line.apply(control('e'));
        assert_eq!(line.cursor(), 2);
        line.apply(key(KeyCode::Left));
        line.apply(control('u'));
        assert_eq!((line.text(), line.cursor()), ("b".to_owned(), 0));
    }

    #[test]
    fn esc_and_ctrl_c_leave_and_ctrl_d_ends_an_empty_line() {
        let mut line = Line::default();
        assert_eq!(line.apply(key(KeyCode::Esc)), Edit::Escaped);
        assert_eq!(line.apply(control('c')), Edit::Escaped);
        assert_eq!(line.apply(control('d')), Edit::Ended);
        typed(&mut line, "a");
        assert_eq!(line.apply(control('d')), Edit::Typing, "Ctrl+D ends only an empty line");
        assert_eq!(line.len(), 1);
    }
}
