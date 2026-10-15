//! The state of a select or multiselect field while the user moves through its options, apart from any terminal.

/// How many options a choice takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Exactly one option.
    Single,
    /// One option or more, up to `limit` when there is one.
    Multiple {
        /// The most options that may be chosen.
        limit: Option<usize>,
    },
}

/// A key that moves through or settles a choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Action {
    Up,
    Down,
    Toggle,
    Confirm,
}

/// The options of a choice, the one under the cursor, and those chosen.
#[derive(Debug)]
pub(crate) struct Selection {
    mode: Mode,
    cursor: usize,
    chosen: Vec<bool>,
}

impl Selection {
    /// A selection over `len` options, the cursor on the first and none chosen.
    pub(crate) fn new(len: usize, mode: Mode) -> Self {
        Self {
            mode,
            cursor: 0,
            chosen: vec![false; len],
        }
    }

    pub(crate) fn cursor(&self) -> usize {
        self.cursor
    }

    pub(crate) fn is_chosen(&self, index: usize) -> bool {
        self.chosen[index]
    }

    pub(crate) fn mode(&self) -> Mode {
        self.mode
    }

    /// Applies `action`, and gives the indexes of the chosen options once it settles the choice. A multiselect
    /// settles only with at least one option chosen.
    pub(crate) fn apply(&mut self, action: Action) -> Option<Vec<usize>> {
        match action {
            Action::Up => self.cursor = self.cursor.saturating_sub(1),
            Action::Down => self.cursor = (self.cursor + 1).min(self.chosen.len().saturating_sub(1)),
            Action::Toggle => self.toggle(),
            Action::Confirm => return self.settled(),
        }
        None
    }

    fn toggle(&mut self) {
        let Mode::Multiple { limit } = self.mode else { return };
        let count = self.chosen.iter().filter(|&&chosen| chosen).count();
        let cursor = &mut self.chosen[self.cursor];
        if *cursor || limit.is_none_or(|limit| count < limit) {
            *cursor = !*cursor;
        }
    }

    fn settled(&self) -> Option<Vec<usize>> {
        match self.mode {
            Mode::Single => Some(vec![self.cursor]),
            Mode::Multiple { .. } => {
                let chosen: Vec<usize> = (0..self.chosen.len()).filter(|&index| self.chosen[index]).collect();
                (!chosen.is_empty()).then_some(chosen)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Action, Mode, Selection};

    #[test]
    fn the_cursor_stays_within_the_options() {
        let mut selection = Selection::new(3, Mode::Single);
        selection.apply(Action::Up);
        assert_eq!(selection.cursor(), 0);
        for _ in 0..5 {
            selection.apply(Action::Down);
        }
        assert_eq!(selection.cursor(), 2);
    }

    #[test]
    fn a_single_choice_settles_on_the_cursor() {
        let mut selection = Selection::new(3, Mode::Single);
        selection.apply(Action::Down);
        selection.apply(Action::Toggle);
        assert_eq!(selection.apply(Action::Confirm), Some(vec![1]));
    }

    #[test]
    fn a_multiple_choice_needs_one_option_and_respects_its_limit() {
        let mut selection = Selection::new(3, Mode::Multiple { limit: Some(2) });
        assert_eq!(selection.apply(Action::Confirm), None);
        for _ in 0..3 {
            selection.apply(Action::Toggle);
            selection.apply(Action::Down);
        }
        assert_eq!(selection.apply(Action::Confirm), Some(vec![0, 1]));
        selection.apply(Action::Up);
        selection.apply(Action::Toggle);
        assert!(!selection.is_chosen(1));
        selection.apply(Action::Down);
        selection.apply(Action::Toggle);
        assert_eq!(selection.apply(Action::Confirm), Some(vec![0, 2]));
    }
}
