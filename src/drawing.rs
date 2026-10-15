//! Drawing a choice in place: the prompt, the keys, and the part of the options that fits the screen.

use std::io::{self, Write};

use crossterm::cursor::MoveToPreviousLine;
use crossterm::queue;
use crossterm::style::{Print, PrintStyledContent, Stylize};
use crossterm::terminal::{Clear, ClearType};

use crate::console::Choice;
use crate::selection::{Mode, Selection};

/// Moves up over the `drawn` rows of the last drawing and clears them, so the next one takes their place.
fn erase(out: &mut impl Write, drawn: u16) -> io::Result<()> {
    if drawn > 0 {
        queue!(out, MoveToPreviousLine(drawn))?;
    }
    queue!(out, Clear(ClearType::FromCursorDown))
}

/// Replaces the choice with one line, `summary`, which stays above what comes next.
pub(crate) fn settle(out: &mut impl Write, drawn: u16, summary: &str) -> io::Result<()> {
    erase(out, drawn)?;
    queue!(out, Print(format!("{summary}\r\n")))?;
    out.flush()
}

/// The size of the terminal a choice is drawn in.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Screen {
    pub(crate) columns: u16,
    pub(crate) rows: u16,
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
pub(crate) fn draw(
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

/// The line of keys under the prompt, in grey: how to move and choose, how to go back to the defaults when the
/// choice has them, and how to cancel, or to go back to the review when an answer is being changed.
fn controls(choice: &Choice<'_>) -> Line {
    let messages = choice.messages;
    let mut keys = vec![&messages.key_move];
    match choice.mode {
        Mode::Single => keys.push(&messages.key_choose),
        Mode::Multiple { .. } => keys.extend([&messages.key_mark, &messages.key_confirm]),
    }
    if !choice.defaults.is_empty() {
        keys.push(&messages.key_default);
    }
    keys.push(if choice.back {
        &messages.key_back
    } else {
        &messages.key_cancel
    });
    let keys: Vec<&str> = keys.into_iter().map(String::as_str).collect();
    Line {
        text: String::new(),
        note: Some(keys.join(&messages.controls_separator)),
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
    use super::{Screen, draw, rows_of, settle};
    use crate::console::Choice;
    use crate::prompt::{Messages, Prompt};
    use crate::selection::{Mode, Selection};

    fn options(labels: &[&str]) -> Vec<String> {
        labels.iter().map(|&label| label.to_owned()).collect()
    }

    fn drawing(choice: &Choice<'_>, selection: &mut Selection, screen: Screen) -> (String, u16) {
        let mut out = Vec::new();
        let rows = draw(&mut out, choice, selection, 0, screen).unwrap();
        (String::from_utf8(out).unwrap(), rows)
    }

    #[test]
    fn a_multiselect_draws_its_marks_cursor_and_default() {
        let prompt = Prompt::new("hobbies", "Hobbies");
        let labels = options(&["Reading", "Music"]);
        let messages = Messages::default();
        let choice = Choice {
            prompt: &prompt,
            options: &labels,
            mode: Mode::Multiple { limit: None },
            defaults: &[0],
            cursor: 0,
            messages: &messages,
            back: false,
        };
        let mut selection = Selection::new(2, choice.mode, &[0]);
        let (text, rows) = drawing(&choice, &mut selection, Screen { columns: 80, rows: 24 });
        assert!(text.contains("> [*] Reading \u{1b}[38;5;8m(default)"), "{text:?}");
        assert!(text.contains("  [ ] Music\r\n"), "{text:?}");
        assert!(
            text.contains("↑↓ move · Space mark · Enter confirm · d default · Esc cancel"),
            "{text:?}"
        );
        assert_eq!(rows, 4, "the prompt, the controls and two options");
    }

    #[test]
    fn a_long_list_is_cut_to_the_screen_with_what_is_out_of_view() {
        let prompt = Prompt::new("pick", "Pick:");
        let labels: Vec<String> = (0..20).map(|n| format!("Option {n}")).collect();
        let messages = Messages::default();
        let choice = Choice {
            prompt: &prompt,
            options: &labels,
            mode: Mode::Single,
            defaults: &[10],
            cursor: 10,
            messages: &messages,
            back: false,
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
            text.contains("↑↓ move · Enter choose · d default · Esc cancel"),
            "{text:?}"
        );
        assert!(text.contains("> Option 10") && !text.contains("Option 7\r"), "{text:?}");
    }

    #[test]
    fn a_change_from_the_review_goes_back_with_esc() {
        let prompt = Prompt::new("plan", "Plan:");
        let labels = options(&["Free", "Pro"]);
        let messages = Messages::default();
        let choice = Choice {
            prompt: &prompt,
            options: &labels,
            mode: Mode::Single,
            defaults: &[],
            cursor: 0,
            messages: &messages,
            back: true,
        };
        let mut selection = Selection::new(2, Mode::Single, &[]);
        let (text, _) = drawing(&choice, &mut selection, Screen { columns: 80, rows: 24 });
        assert!(text.contains("↑↓ move · Enter choose · Esc back"), "{text:?}");
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
