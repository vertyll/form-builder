//! The terminal a program runs in, as a [`Console`].

use std::io::{self, BufRead, IsTerminal, Write};

use crossterm::cursor::{Hide, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::{execute, queue, terminal};

use crate::Error;
use crate::console::{Choice, Console};
use crate::drawing::{Screen, draw, settle};
use crate::prompt::Prompt;
use crate::selection::{Action, Selection};

/// The terminal the program runs in.
///
/// Text comes from standard input, passwords are typed unseen, and choices are drawn in place and moved through with
/// the arrow keys, Space and Enter. Hints and defaults are shown in grey.
#[derive(Debug, Default)]
pub struct Terminal;

impl Console for Terminal {
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        let mut stdout = io::stdout();
        write_prompt(&mut stdout, prompt)?;
        stdout.flush()?;
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line)? == 0 {
            return Ok(None);
        }
        Ok(Some(line.trim_end_matches(['\r', '\n']).to_owned()))
    }

    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error> {
        if !io::stdin().is_terminal() {
            return self.read_line(prompt);
        }
        let mut stdout = io::stdout();
        write_prompt(&mut stdout, prompt)?;
        stdout.flush()?;
        let raw = RawMode::enable()?;
        let mut secret = String::new();
        loop {
            let Event::Key(key) = event::read()? else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if is_cancel(key) {
                drop(raw);
                execute!(stdout, Print("\r\n"))?;
                return Ok(None);
            }
            match key.code {
                KeyCode::Enter => {
                    drop(raw);
                    execute!(stdout, Print("\r\n"))?;
                    return Ok(Some(secret));
                }
                KeyCode::Backspace if secret.pop().is_some() => execute!(stdout, Print("\u{8} \u{8}"))?,
                KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
                    secret.push(c);
                    execute!(stdout, Print('*'))?;
                }
                _ => {}
            }
        }
    }

    fn report(&mut self, message: &str) -> Result<(), Error> {
        writeln!(io::stderr(), "{message}")?;
        Ok(())
    }

    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error> {
        let mut selection = Selection::new(choice.options.len(), choice.mode, choice.defaults);
        let mut stdout = io::stdout();
        let _raw = RawMode::enable()?;
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
                settle(
                    &mut stdout,
                    drawn,
                    &format!("{} ({})", choice.prompt, choice.messages.cancelled),
                )?;
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

/// Raw mode, with the cursor hidden, for as long as it lives: the terminal gives every key at once and shows none of
/// them, and gets its usual mode and cursor back however the input ends.
struct RawMode;

impl RawMode {
    fn enable() -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        execute!(io::stdout(), Hide)?;
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
