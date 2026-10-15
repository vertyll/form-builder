//! Where a form reads its input: the [`Terminal`], or any other [`Console`], such as one that replays answers in a
//! test.

use std::io::{self, BufRead, IsTerminal, Write};

use crossterm::cursor::{Hide, MoveToPreviousLine, Show};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{self, Clear, ClearType};
use crossterm::{execute, queue};

use crate::Error;
use crate::prompt::{Messages, Prompt};
use crate::selection::{Action, Mode, Selection};

/// A choice to be made: what [`Console::choose`] shows and starts from.
#[derive(Debug)]
#[non_exhaustive]
pub struct Choice<'a> {
    /// The prompt above the options.
    pub prompt: &'a Prompt,
    /// The labels of the options, in order.
    pub options: &'a [String],
    /// How many options may be chosen.
    pub mode: Mode,
    /// The options the field starts with: the cursor on the first, and all of them chosen in a multiselect.
    pub defaults: &'a [usize],
    /// The texts the form shows of its own.
    pub messages: &'a Messages,
}

/// What a form needs from the place it is filled in.
pub trait Console {
    /// Shows `prompt` and reads one line of text, without its line ending; `None` when the input has ended.
    ///
    /// # Errors
    ///
    /// Returns why the input could not be read.
    fn read_line(&mut self, prompt: &Prompt) -> Result<Option<String>, Error>;

    /// Shows `prompt` and reads one line without showing it, as a password is typed; `None` when the user cancels
    /// or the input has ended.
    ///
    /// # Errors
    ///
    /// Returns why the input could not be read.
    fn read_secret(&mut self, prompt: &Prompt) -> Result<Option<String>, Error>;

    /// Shows a message about the last input, such as why it was refused.
    ///
    /// # Errors
    ///
    /// Returns why the message could not be shown.
    fn report(&mut self, message: &str) -> Result<(), Error>;

    /// Lets the user make `choice`, and gives the indexes of the options chosen: one for [`Mode::Single`], one or
    /// more for [`Mode::Multiple`]. `None` when the user cancels.
    ///
    /// # Errors
    ///
    /// Returns why the choice could not be read.
    fn choose(&mut self, choice: &Choice<'_>) -> Result<Option<Vec<usize>>, Error>;
}

/// The terminal the program runs in: text from standard input, passwords typed unseen, and choices drawn in place
/// and moved through with the arrow keys, Space and Enter. Hints and defaults are shown in grey.
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

/// Writes `prompt` with its hint and default in grey, and a space after it.
fn write_prompt(out: &mut impl Write, prompt: &Prompt) -> io::Result<()> {
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

/// Moves up over the `drawn` rows of the last drawing and clears them, so the next one takes their place.
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

/// The size of the terminal a choice is drawn in.
#[derive(Debug, Clone, Copy)]
struct Screen {
    columns: u16,
    rows: u16,
}

/// A line of a drawn choice: its text, and whether it is shown in grey.
struct Line {
    text: String,
    note: Option<String>,
}

impl Line {
    fn plain(text: String) -> Self {
        Self { text, note: None }
    }
}

/// Draws `choice` where the last drawing, `drawn` rows high, was, and gives how many rows it takes on `screen`.
fn draw(
    out: &mut impl Write,
    choice: &Choice<'_>,
    selection: &mut Selection,
    drawn: u16,
    screen: Screen,
) -> io::Result<u16> {
    erase(out, drawn)?;
    let mut rows: u16 = 0;
    for line in lines(choice, selection, screen.rows) {
        queue!(out, Print(&line.text))?;
        let mut width = line.text.chars().count();
        if let Some(note) = &line.note {
            let gap = if line.text.is_empty() { "" } else { " " };
            queue!(out, Print(gap), PrintStyledContent(note.clone().dark_grey()))?;
            width += gap.len() + note.chars().count();
        }
        queue!(out, Print("\r\n"))?;
        rows = rows.saturating_add(rows_of(width, screen.columns));
    }
    out.flush()?;
    Ok(rows)
}

/// The lines of `choice` on a screen `rows` high: the prompt, the controls of a multiselect, and the options. A list
/// taller than the screen shows the part around the cursor, with how many options are above and below it.
fn lines(choice: &Choice<'_>, selection: &mut Selection, rows: u16) -> Vec<Line> {
    let mut lines = vec![Line::plain(choice.prompt.to_string()), controls(choice)];
    let len = choice.options.len();
    let room = usize::from(rows).saturating_sub(lines.len() + 1).max(1);
    let window = selection.window(if len > room { room.saturating_sub(2).max(1) } else { len });
    let cut = window.len() < len;
    if cut {
        lines.push(more(window.start, &choice.messages.more_above, "↑"));
    }
    lines.extend(window.clone().map(|index| option_line(choice, selection, index)));
    if cut {
        lines.push(more(len - window.end, &choice.messages.more_below, "↓"));
    }
    lines
}

/// The line of keys under the prompt, in grey: how to move, choose and cancel, and how to go back to the defaults
/// when the choice has them.
fn controls(choice: &Choice<'_>) -> Line {
    let messages = choice.messages;
    let mut keys = match choice.mode {
        Mode::Single => messages.controls_single.clone(),
        Mode::Multiple { .. } => messages.controls_multiple.clone(),
    };
    if !choice.defaults.is_empty() {
        keys.push_str(&messages.controls_separator);
        keys.push_str(&messages.controls_default);
    }
    Line {
        text: String::new(),
        note: Some(keys),
    }
}

/// The line of option `index`: the cursor, the mark of a multiselect, the label, and a note when it is a default.
fn option_line(choice: &Choice<'_>, selection: &Selection, index: usize) -> Line {
    let cursor = if index == selection.cursor() { '>' } else { ' ' };
    let label = &choice.options[index];
    let text = match selection.mode() {
        Mode::Single => format!("{cursor} {label}"),
        Mode::Multiple { .. } => {
            let mark = if selection.is_chosen(index) { '*' } else { ' ' };
            format!("{cursor} [{mark}] {label}")
        }
    };
    let note = choice
        .defaults
        .contains(&index)
        .then(|| format!("({})", choice.messages.default_option));
    Line { text, note }
}

/// The line that says how many options are out of view on one side, or an empty one when none are.
fn more(count: usize, words: &str, arrow: &str) -> Line {
    if count == 0 {
        Line::plain(String::new())
    } else {
        Line {
            text: String::new(),
            note: Some(format!("{arrow} {count} {words}")),
        }
    }
}

/// How many rows a line `width` characters wide takes in a terminal `columns` wide: one, or more when it wraps.
fn rows_of(width: usize, columns: u16) -> u16 {
    let rows = width.max(1).div_ceil(usize::from(columns.max(1)));
    u16::try_from(rows).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    use super::{Choice, Screen, action, draw, is_cancel, rows_of, settle, write_prompt};
    use crate::prompt::{Messages, Prompt};
    use crate::selection::{Action, Mode, Selection};

    fn options(labels: &[&str]) -> Vec<String> {
        labels.iter().map(|&label| label.to_owned()).collect()
    }

    fn drawing(choice: &Choice<'_>, selection: &mut Selection, screen: Screen) -> (String, u16) {
        let mut out = Vec::new();
        let rows = draw(&mut out, choice, selection, 0, screen).unwrap();
        (String::from_utf8(out).unwrap(), rows)
    }

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
        let mut prompt = Prompt::new("Port:");
        prompt.marker = Some("*".to_owned());
        prompt.default = Some("8080".to_owned());
        let mut out = Vec::new();
        write_prompt(&mut out, &prompt).unwrap();
        let text = String::from_utf8(out).unwrap();
        assert!(text.starts_with("Port * \u{1b}[38;5;8m[8080]\u{1b}[39m:"), "{text:?}");
    }

    #[test]
    fn a_multiselect_draws_its_marks_cursor_and_default() {
        let prompt = Prompt::new("Hobbies");
        let labels = options(&["Reading", "Music"]);
        let messages = Messages::default();
        let choice = Choice {
            prompt: &prompt,
            options: &labels,
            mode: Mode::Multiple { limit: None },
            defaults: &[0],
            messages: &messages,
        };
        let mut selection = Selection::new(2, choice.mode, &[0]);
        let (text, rows) = drawing(&choice, &mut selection, Screen { columns: 80, rows: 24 });
        assert!(text.contains("> [*] Reading \u{1b}[38;5;8m(default)"), "{text:?}");
        assert!(text.contains("  [ ] Music\r\n"), "{text:?}");
        assert!(
            text.contains("Space mark · Enter confirm · Esc cancel · d default"),
            "{text:?}"
        );
        assert_eq!(rows, 4, "the prompt, the controls and two options");
    }

    #[test]
    fn a_long_list_is_cut_to_the_screen_with_what_is_out_of_view() {
        let prompt = Prompt::new("Pick:");
        let labels: Vec<String> = (0..20).map(|n| format!("Option {n}")).collect();
        let messages = Messages::default();
        let choice = Choice {
            prompt: &prompt,
            options: &labels,
            mode: Mode::Single,
            defaults: &[10],
            messages: &messages,
        };
        let mut selection = Selection::new(20, Mode::Single, &[10]);
        let (text, rows) = drawing(&choice, &mut selection, Screen { columns: 80, rows: 8 });
        assert_eq!(
            rows, 7,
            "the prompt, the controls, two counts and three options, below eight rows"
        );
        assert!(
            text.contains("↑ 8 more above") && text.contains("↓ 9 more below"),
            "{text:?}"
        );
        assert!(
            text.contains("↑↓ move · Enter choose · Esc cancel · d default"),
            "{text:?}"
        );
        assert!(text.contains("> Option 10") && !text.contains("Option 7\r"), "{text:?}");
    }

    #[test]
    fn a_long_line_counts_the_rows_it_wraps_onto() {
        assert_eq!(rows_of(0, 10), 1);
        assert_eq!(rows_of(10, 10), 1);
        assert_eq!(rows_of(11, 10), 2);
    }

    #[test]
    fn the_summary_takes_the_place_of_the_last_drawing() {
        let mut out = Vec::new();
        settle(&mut out, 3, "Plan: Pro").unwrap();
        let text = String::from_utf8(out).unwrap();
        assert_eq!(text, "\u{1b}[3F\u{1b}[JPlan: Pro\r\n");
    }
}
