//! Where a form reads its input: the [`Terminal`], or any other [`Console`], such as one that replays answers in a
//! test.

use std::io::{self, BufRead, Write};

use crossterm::cursor::MoveTo;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::Print;
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};

use crate::Error;
use crate::selection::{Action, Mode, Selection};

/// What a form needs from the place it is filled in.
pub trait Console {
    /// Shows `prompt` and reads one line of text, without its line ending; `None` when the input has ended.
    ///
    /// # Errors
    ///
    /// Returns why the input could not be read.
    fn read_line(&mut self, prompt: &str) -> Result<Option<String>, Error>;

    /// Shows a message about the last input, such as why it was refused.
    ///
    /// # Errors
    ///
    /// Returns why the message could not be shown.
    fn report(&mut self, message: &str) -> Result<(), Error>;

    /// Lets the user choose among `options`, and gives the indexes of those chosen: one for [`Mode::Single`], one or
    /// more for [`Mode::Multiple`]. `None` when the user cancels.
    ///
    /// # Errors
    ///
    /// Returns why the choice could not be read.
    fn choose(&mut self, prompt: &str, options: &[String], mode: Mode) -> Result<Option<Vec<usize>>, Error>;
}

/// The terminal the program runs in: text from standard input, and choices drawn on the screen and moved through
/// with the arrow keys, Space and Enter.
#[derive(Debug, Default)]
pub struct Terminal;

impl Console for Terminal {
    fn read_line(&mut self, prompt: &str) -> Result<Option<String>, Error> {
        let mut stdout = io::stdout();
        write!(stdout, "{prompt} ")?;
        stdout.flush()?;
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line)? == 0 {
            return Ok(None);
        }
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        writeln!(io::stderr(), "{message}")?;
        Ok(())
    }

    fn choose(&mut self, prompt: &str, options: &[String], mode: Mode) -> Result<Option<Vec<usize>>, Error> {
        let mut selection = Selection::new(options.len(), mode);
        let mut stdout = io::stdout();
        let raw = RawMode::enable()?;
        loop {
            draw(&mut stdout, prompt, options, &selection)?;
            let Event::Key(key) = event::read()? else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if is_cancel(key) {
                drop(raw);
                return Ok(None);
            }
            if let Some(action) = action(key.code)
                && let Some(chosen) = selection.apply(action)
            {
                drop(raw);
                execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                return Ok(Some(chosen));
            }
        }
    }
}

/// Raw mode for as long as it lives: the terminal gives every key at once and shows none of them, and gets its
/// usual mode back however the choice ends.
struct RawMode;

impl RawMode {
    fn enable() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
    }
}

fn action(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Char(' ') => Some(Action::Toggle),
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}

fn is_cancel(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

fn draw(out: &mut impl Write, prompt: &str, options: &[String], selection: &Selection) -> io::Result<()> {
    queue!(out, Clear(ClearType::All), MoveTo(0, 0), Print(format!("{prompt}\r\n")))?;
    if let Mode::Multiple { .. } = selection.mode() {
        queue!(out, Print("Space chooses, Enter confirms\r\n"))?;
    }
    for (index, option) in options.iter().enumerate() {
        let cursor = if index == selection.cursor() { '>' } else { ' ' };
        let line = match selection.mode() {
            Mode::Single => format!("{cursor} {option}\r\n"),
            Mode::Multiple { .. } => {
                let mark = if selection.is_chosen(index) { '*' } else { ' ' };
                format!("{cursor} [{mark}] {option}\r\n")
            }
        };
        queue!(out, Print(line))?;
    }
    out.flush()
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{action, draw, is_cancel};
    use crate::selection::{Action, Mode, Selection};

    #[test]
    fn keys_map_to_actions() {
        assert_eq!(action(KeyCode::Up), Some(Action::Up));
        assert_eq!(action(KeyCode::Char('j')), Some(Action::Down));
        assert_eq!(action(KeyCode::Char(' ')), Some(Action::Toggle));
        assert_eq!(action(KeyCode::Enter), Some(Action::Confirm));
        assert_eq!(action(KeyCode::Char('A')), None);
    }

    #[test]
    fn esc_and_ctrl_c_cancel() {
        assert!(is_cancel(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(is_cancel(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!is_cancel(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)));
    }

    #[test]
    fn a_multiselect_draws_its_marks_and_cursor() {
        let mut selection = Selection::new(2, Mode::Multiple { limit: None });
        selection.apply(Action::Toggle);
        let mut out = Vec::new();
        draw(
            &mut out,
            "Hobbies",
            &["Reading".to_owned(), "Music".to_owned()],
            &selection,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("> [*] Reading\r\n") && text.contains("  [ ] Music\r\n"),
            "{text:?}"
        );
    }
}
