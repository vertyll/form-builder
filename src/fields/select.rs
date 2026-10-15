//! The choice fields: one option, or several.

use super::{Common, Field, Kind};
use crate::form::Answers;
use crate::selection::Mode;

/// A field whose value is one of its options, each a value of type `T` and the label shown for it.
///
/// ```
/// use form_builder::Select;
///
/// let plan = Select::new("plan", "Plan:", [(1u8, "Free"), (2, "Pro")]).default(&2);
/// ```
#[must_use]
pub struct Select<T> {
    common: Common,
    values: Vec<T>,
    labels: Vec<String>,
    default: Option<usize>,
}

impl<T: Clone + Send + Sync + 'static> Select<T> {
    /// A select over `options`, the cursor starting on the first.
    pub fn new<L: Into<String>>(name: &str, prompt: &str, options: impl IntoIterator<Item = (T, L)>) -> Self {
        let (values, labels) = split(options);
        Self {
            common: Common::new(name, prompt),
            values,
            labels,
            default: None,
        }
    }

    common_methods!();
}

impl<T: Clone + PartialEq + Send + Sync + 'static> Select<T> {
    /// Starts the cursor on the option holding `value`, marked as the default; a value that is not among the options
    /// leaves the cursor on the first.
    pub fn default(mut self, value: &T) -> Self {
        self.default = self.values.iter().position(|option| option == value);
        self
    }
}

impl<T: Clone + Send + Sync + 'static> From<Select<T>> for Field {
    fn from(select: Select<T>) -> Self {
        let values = select.values;
        let kind = Kind::Choice {
            options: select.labels,
            mode: Mode::Single,
            defaults: select.default.into_iter().collect(),
            pick: Box::new(move |chosen| Box::new(values[chosen[0]].clone())),
        };
        Self::new(select.common, kind)
    }
}

/// A field whose value is a `Vec<T>` of one or more of its options.
///
/// ```
/// use form_builder::MultiSelect;
///
/// let tags = MultiSelect::new("tags", "Tags:", [("rust", "Rust"), ("go", "Go"), ("zig", "Zig")])
///     .defaults(&["rust"])
///     .limit(2);
/// ```
#[must_use]
pub struct MultiSelect<T> {
    common: Common,
    values: Vec<T>,
    labels: Vec<String>,
    defaults: Vec<usize>,
    limit: Option<usize>,
}

impl<T: Clone + Send + Sync + 'static> MultiSelect<T> {
    /// A multiselect over `options`, none chosen.
    pub fn new<L: Into<String>>(name: &str, prompt: &str, options: impl IntoIterator<Item = (T, L)>) -> Self {
        let (values, labels) = split(options);
        Self {
            common: Common::new(name, prompt),
            values,
            labels,
            defaults: Vec::new(),
            limit: None,
        }
    }

    /// Lets at most `limit` options be chosen.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    pub(crate) fn limit_to(mut self, limit: Option<usize>) -> Self {
        self.limit = limit;
        self
    }

    common_methods!();
}

impl<T: Clone + PartialEq + Send + Sync + 'static> MultiSelect<T> {
    /// Starts with the options holding `values` chosen, marked as the defaults; values not among the options are
    /// left out.
    pub fn defaults(mut self, values: &[T]) -> Self {
        self.defaults = (0..self.values.len())
            .filter(|&index| values.contains(&self.values[index]))
            .collect();
        self
    }
}

impl<T: Clone + Send + Sync + 'static> From<MultiSelect<T>> for Field {
    fn from(multi: MultiSelect<T>) -> Self {
        let values = multi.values;
        let kind = Kind::Choice {
            options: multi.labels,
            mode: Mode::Multiple { limit: multi.limit },
            defaults: multi.defaults,
            pick: Box::new(move |chosen| {
                Box::new(chosen.iter().map(|&index| values[index].clone()).collect::<Vec<T>>())
            }),
        };
        Self::new(multi.common, kind)
    }
}

fn split<T, L: Into<String>>(options: impl IntoIterator<Item = (T, L)>) -> (Vec<T>, Vec<String>) {
    options.into_iter().map(|(value, label)| (value, label.into())).unzip()
}
