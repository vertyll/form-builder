//! Where a form reads its input: the [`Terminal`], or any other [`Console`], such as one that replays answers in a
//! test.

use std::io::{self, BufRead, Write};

use crossterm::cursor::{Hide, MoveToPreviousLine, Show};
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
        let _raw = RawMode::enable()?;
        let mut drawn = 0;
        loop {
            let (columns, _) = terminal::size()?;
            drawn = draw(&mut stdout, prompt, options, &selection, drawn, columns)?;
            let Event::Key(key) = event::read()? else { continue };
            if key.kind != KeyEventKind::Press {
                continue;
            }
            if is_cancel(key) {
                settle(&mut stdout, drawn, &format!("{prompt} (cancelled)"))?;
                return Ok(None);
            }
            if let Some(action) = action(key.code)
                && let Some(chosen) = selection.apply(action)
            {
                let labels: Vec<&str> = chosen.iter().map(|&index| options[index].as_str()).collect();
                settle(&mut stdout, drawn, &format!("{prompt} {}", labels.join(", ")))?;
                return Ok(Some(chosen));
            }
        }
    }
}

/// Raw mode, with the cursor hidden, for as long as it lives: the terminal gives every key at once and shows none of
/// them, and gets its usual mode and cursor back however the choice ends.
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
        KeyCode::Enter => Some(Action::Confirm),
        _ => None,
    }
}

fn is_cancel(key: KeyEvent) -> bool {
    key.code == KeyCode::Esc || (key.code == KeyCode::Char('c') && key.modifiers.contains(KeyModifiers::CONTROL))
}

/// Moves up over the `drawn` lines of the last drawing and clears them, so the next one takes their place.
fn erase(out: &mut impl Write, drawn: u16) -> io::Result<()> {
    if drawn > 0 {
        queue!(out, MoveToPreviousLine(drawn))?;
    }
    queue!(out, Clear(ClearType::FromCursorDown))
}

/// Replaces the choice with one line, `summary`, which stays above what comes next.
fn settle(out: &mut impl Write, drawn: u16, summary: &str) -> io::Result<()> {
    erase(out, drawn)?;
    queue!(out, Print(format!("{summary}\r\n")))?;
    out.flush()
}

/// Draws the choice where the last drawing, `drawn` rows high, was, and gives how many rows it takes in a terminal
/// `columns` wide, where a line longer than that wraps onto more rows.
fn draw(
    out: &mut impl Write,
    prompt: &str,
    options: &[String],
    selection: &Selection,
    drawn: u16,
    columns: u16,
) -> io::Result<u16> {
    erase(out, drawn)?;
    let mut lines = vec![prompt.to_owned()];
    if let Mode::Multiple { .. } = selection.mode() {
        lines.push("Space chooses, Enter confirms".to_owned());
    }
    for (index, option) in options.iter().enumerate() {
        let cursor = if index == selection.cursor() { '>' } else { ' ' };
        lines.push(match selection.mode() {
            Mode::Single => format!("{cursor} {option}"),
            Mode::Multiple { .. } => {
                let mark = if selection.is_chosen(index) { '*' } else { ' ' };
                format!("{cursor} [{mark}] {option}")
            }
        });
    }
    let mut rows: u16 = 0;
    for line in &lines {
        queue!(out, Print(format!("{line}\r\n")))?;
        rows = rows.saturating_add(rows_of(line, columns));
    }
    out.flush()?;
    Ok(rows)
}

/// How many rows `line` takes in a terminal `columns` wide: one, or more when it wraps.
fn rows_of(line: &str, columns: u16) -> u16 {
    let width = line.chars().count().max(1);
    let rows = width.div_ceil(usize::from(columns.max(1)));
    u16::try_from(rows).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{action, draw, is_cancel, rows_of, settle};
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
        let drawn = draw(
            &mut out,
            "Hobbies",
            &["Reading".to_owned(), "Music".to_owned()],
            &selection,
            0,
            80,
        )
        .unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.contains("> [*] Reading\r\n") && text.contains("  [ ] Music\r\n"),
            "{text:?}"
        );
        assert_eq!(drawn, 4, "the prompt, the hint and two options");
    }

    #[test]
    fn a_long_line_counts_the_rows_it_wraps_onto() {
        assert_eq!(rows_of("", 10), 1);
        assert_eq!(rows_of("0123456789", 10), 1);
        assert_eq!(rows_of("0123456789a", 10), 2);
        assert_eq!(rows_of("żółw", 2), 2);
        let selection = Selection::new(1, Mode::Single);
        let drawn = draw(&mut Vec::new(), "Pick:", &["a".repeat(25)], &selection, 0, 10).unwrap();
        assert_eq!(drawn, 1 + 3, "the prompt, and the option over three rows");
    }

    #[test]
    fn a_redraw_and_the_summary_take_the_place_of_the_last_drawing() {
        let selection = Selection::new(2, Mode::Single);
        let options = ["Free".to_owned(), "Pro".to_owned()];
        let mut out = Vec::new();
        draw(&mut out, "Plan:", &options, &selection, 3, 80).unwrap();
        settle(&mut out, 3, "Plan: Pro").unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(
            text.starts_with("\u{1b}[3F"),
            "moves up over the last drawing first: {text:?}"
        );
        assert!(!text.contains("\u{1b}[2J"), "never clears the whole screen: {text:?}");
        assert!(text.ends_with("\u{1b}[3F\u{1b}[JPlan: Pro\r\n"), "{text:?}");
    }
}
