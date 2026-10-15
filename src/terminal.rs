//! The terminal a program runs in, as a [`Console`].

use std::io::{self, BufRead, IsTerminal, Write};

use crossterm::cursor::{Hide, MoveToColumn, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType};
use crossterm::{execute, queue, terminal};

use crate::Error;
use crate::console::{Choice, Console};
use crate::drawing::{Screen, draw, settle};
use crate::line::{Edit, Line};
use crate::prompt::Prompt;
use crate::selection::{Action, Selection};

/// The terminal the program runs in.
///
/// Text and passwords are typed key by key, a password shown as `*`: the arrows, Home and End move in the line,
/// Backspace and Delete remove a character, and Esc leaves it. Choices are drawn in place and moved through with the
/// arrow keys, Space and Enter. Hints and defaults are shown in grey. A standard input that is no terminal, as a
/// pipe is, is read line by line.
#[derive(Debug, Default)]
pub struct Terminal;

impl Console for Terminal {
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        if !io::stdin().is_terminal() {
            return read_plain_line(prompt);
        }
        Ok(match edit_line(prompt, false)? {
            Edit::Done(text) => Some(text),
            Edit::Escaped => Some(ESC.to_string()),
            Edit::Ended | Edit::Typing => None,
        })
    }

    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        if !io::stdin().is_terminal() {
            return read_plain_line(prompt);
        }
        Ok(match edit_line(prompt, true)? {
            Edit::Done(text) => Some(text),
            Edit::Escaped | Edit::Ended | Edit::Typing => None,
        })
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        writeln!(io::stderr(), "{message}")?;
        Ok(())
    }

    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error> {
        let mut selection =
            Selection::new(choice.options.len(), choice.mode, choice.defaults).starting_at(choice.cursor);
        let mut stdout = io::stdout();
        let _raw = RawMode::enable(true)?;
        let mut drawn = 0;
        loop {
            let (columns, rows) = terminal::size()?;
            let screen = Screen { columns, rows };
            drawn = draw(&mut stdout, choice, &mut selection, drawn, screen)?;
            let Event::Key(key) = event::read()? else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if is_cancel(key) {
                let summary = if choice.back {
                    let kept: Vec<&str> = choice
                        .defaults
                        .iter()
                        .map(|&index| choice.options[index].as_str())
                        .collect();
                    format!("{} {}", choice.prompt, kept.join(", "))
                } else {
                    format!("{} ({})", choice.prompt, choice.messages.cancelled)
                };
                settle(&mut stdout, drawn, &summary)?;
                return Ok(None);
            }
            if let Some(action) = action(key.code)
                && let Some(chosen) = selection.apply(action)
            {
                let labels: Vec<&str> = chosen.iter().map(|&index| choice.options[index].as_str()).collect();
                settle(&mut stdout, drawn, &format!("{} {}", choice.prompt, labels.join(", ")))?;
                return Ok(Some(chosen));
            }
        }
    }

    /// Whether the standard input is a terminal: a program fed through a pipe is answered by a script, which neither
    /// reviews its answers nor types a password twice.
    fn is_interactive(&self) -> bool {
        io::stdin().is_terminal()
    }
}

/// The Escape character, which [`Console::read_line`] gives for Esc, so the form can tell it from an answer.
const ESC: char = '\u{1b}';

/// Reads one line from a standard input that is no terminal, as a pipe is, without its line ending.
fn read_plain_line(prompt: &Prompt) -> Result<Option<String>, Error> {
    let mut stdout = io::stdout();
    write_prompt(&mut stdout, prompt)?;
    stdout.flush()?;
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line)? == 0 {
        return Ok(None);
    }
    Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
}

/// Lets the user type a line after `prompt`, key by key, so Esc leaves it at once and the arrows move in it; a
/// `masked` line shows `*` for each character.
fn edit_line(prompt: &Prompt, masked: bool) -> Result<Edit, Error> {
    let mut stdout = io::stdout();
    let start = prompt.to_string().chars().count() + 1;
    let _raw = RawMode::enable(false)?;
    let mut line = Line::default();
    loop {
        queue!(stdout, Print('\r'), Clear(ClearType::CurrentLine))?;
        write_prompt(&mut stdout, prompt)?;
        let shown = if masked { "*".repeat(line.len()) } else { line.text() };
        let column = u16::try_from(start + line.cursor()).unwrap_or(u16::MAX);
        queue!(stdout, Print(shown), MoveToColumn(column))?;
        stdout.flush()?;
        let Event::Key(key) = event::read()? else { continue };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        let edit = line.apply(key);
        if edit != Edit::Typing {
            execute!(stdout, Print("\r\n"))?;
            return Ok(edit);
        }
    }
}

/// Writes `prompt` with its step, hint and default in grey, and a space after it.
fn write_prompt(out: &mut impl Write, prompt: &Prompt) -> io::Result<()> {
    if let Some(step) = prompt.shown_step() {
        queue!(out, PrintStyledContent(step.dark_grey()), Print(' '))?;
    }
    queue!(out, Print(prompt.lead()))?;
    for part in [prompt.shown_hint(), prompt.shown_default()].into_iter().flatten() {
        queue!(out, Print(' '), PrintStyledContent(part.dark_grey()))?;
    }
    queue!(out, Print(prompt.end()), Print(' '))
}

/// Raw mode for as long as it lives, with the cursor hidden when asked: the terminal gives every key at once and
/// shows none of them, and gets its usual mode and cursor back however the input ends.
struct RawMode;

impl RawMode {
    fn enable(hide_cursor: bool) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        if hide_cursor {
            execute!(io::stdout(), Hide)?;
        }
        Ok(Self)
    }
}

impl Drop for RawMode {
    fn drop(&mut self) {
        let _ = execute!(io::stdout(), Show);
        let _ = terminal::disable_raw_mode();
    }
}

fn action(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(Action::Up),
        KeyCode::Down | KeyCode::Char('j') => Some(Action::Down),
        KeyCode::Char(' ') => Some(Action::Toggle),
        KeyCode::Char('d') => Some(Action::Default),
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}

fn is_cancel(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{action, is_cancel, write_prompt};
    use crate::prompt::Prompt;
    use crate::selection::Action;

    #[test]
    fn keys_map_to_actions() {
        assert_eq!(action(KeyCode::Up), Some(Action::Up));
        assert_eq!(action(KeyCode::Char('j')), Some(Action::Down));
        assert_eq!(action(KeyCode::Char(' ')), Some(Action::Toggle));
        assert_eq!(action(KeyCode::Enter), Some(Action::Confirm));
        assert_eq!(action(KeyCode::Char('d')), Some(Action::Default));
        assert_eq!(action(KeyCode::Char('A')), None);
    }

    #[test]
    fn esc_and_ctrl_c_cancel() {
        assert!(is_cancel(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE)));
        assert!(is_cancel(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)));
        assert!(!is_cancel(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE)));
    }

    #[test]
    fn a_prompt_shows_its_hint_and_default_in_grey() {
        let mut prompt = Prompt::new("port", "Port:");
        prompt.marker = Some("*".to_owned());
        prompt.default = Some("8080".to_owned());
        let mut out = Vec::new();
        write_prompt(&mut out, &prompt).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("Port * \u{1b}[38;5;8m[8080]\u{1b}[39m:"), "{text:?}");
    }
}
