//! The kinds of field a form asks, each built by its own type: [`Text`], [`Select`], [`MultiSelect`], [`Confirm`]
//! and [`Password`]. Each takes a hint shown with its prompt and a condition under which it is asked at all.

use std::any::Any;
use std::fmt::{self, Display};
use std::marker::PhantomData;
use std::str::FromStr;

use crate::form::Answers;
use crate::selection::Mode;
use crate::validate::Validator;

pub(crate) type Value = Box<dyn Any>;
pub(crate) type Condition = Box<dyn Fn(&Answers<'_>) -> bool>;
pub(crate) type Check = Box<dyn Fn(&Answers<'_>, &str) -> bool>;
pub(crate) type Parse = Box<dyn Fn(&str) -> Result<Value, String>>;
pub(crate) type Pick = Box<dyn Fn(&[usize]) -> Value>;

/// A check of an input against the answers given before it, and the message shown when it fails.
pub(crate) struct ContextRule {
    pub(crate) check: Check,
    pub(crate) message: String,
}

/// What an empty answer to a text field gives: the default as shown, and the value itself.
pub(crate) struct Fallback {
    pub(crate) shown: String,
    pub(crate) value: Box<dyn Fn() -> Value>,
}

/// What a field asks for, with what turns the answer into its value.
pub(crate) enum Kind {
    Text {
        validator: Option<Validator>,
        rules: Vec<ContextRule>,
        parse: Parse,
        fallback: Option<Fallback>,
        optional: bool,
    },
    Secret {
        validator: Option<Validator>,
        rules: Vec<ContextRule>,
        repeat: Option<String>,
    },
    Confirm {
        default: Option<bool>,
    },
    Choice {
        options: Vec<String>,
        mode: Mode,
        defaults: Vec<usize>,
        pick: Pick,
    },
}

/// Whether a field has been asked, and what it gave.
pub(crate) enum State {
    Pending,
    Skipped,
    Answered(Value),
}

/// A field of a form, made from a [`Text`], [`Select`], [`MultiSelect`], [`Confirm`] or [`Password`] and added with
/// [`FormBuilder::field`](crate::FormBuilder::field).
pub struct Field {
    pub(crate) name: String,
    pub(crate) prompt: String,
    pub(crate) hint: Option<String>,
    pub(crate) condition: Option<Condition>,
    pub(crate) kind: Kind,
    pub(crate) state: State,
}

impl Field {
    fn new(common: Common, kind: Kind) -> Self {
        Self {
            name: common.name,
            prompt: common.prompt,
            hint: common.hint,
            condition: common.condition,
            kind,
            state: State::Pending,
        }
    }

    /// Whether the field needs an answer: every one but an optional text field.
    pub(crate) fn is_required(&self) -> bool {
        !matches!(self.kind, Kind::Text { optional: true, .. })
    }
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Field")
            .field("name", &self.name)
            .field("prompt", &self.prompt)
            .finish_non_exhaustive()
    }
}

/// What every kind of field has: its name, prompt, hint and condition.
struct Common {
    name: String,
    prompt: String,
    hint: Option<String>,
    condition: Option<Condition>,
}

impl Common {
    fn new(name: &str, prompt: &str) -> Self {
        Self {
            name: name.to_owned(),
            prompt: prompt.to_owned(),
            hint: None,
            condition: None,
        }
    }
}

/// Adds the methods every kind of field has.
macro_rules! common_methods {
    () => {
        /// Shows `hint` with the prompt, as a short note on what to enter.
        pub fn hint(mut self, hint: impl Into<String>) -> Self {
            self.common.hint = Some(hint.into());
            self
        }

        /// Asks the field only when `condition` holds for the answers given before it; otherwise the field is
        /// skipped, and reading it gives [`Error::Skipped`](crate::Error::Skipped).
        pub fn when(mut self, condition: impl Fn(&Answers<'_>) -> bool + 'static) -> Self {
            self.common.condition = Some(Box::new(condition));
            self
        }
    };
}

/// Adds the methods of the fields whose input is checked: a validator, and rules that see earlier answers.
macro_rules! check_methods {
    () => {
        /// Checks the trimmed input with `validator` before it is taken; a failed check shows its message, and the
        /// field asks again.
        pub fn validator(mut self, validator: Validator) -> Self {
            self.validator = Some(validator);
            self
        }

        /// Checks the input against the answers given before it, such as a range that depends on an earlier field;
        /// `message` is shown when `check` fails.
        pub fn rule_with(
            mut self,
            check: impl Fn(&Answers<'_>, &str) -> bool + 'static,
            message: impl Into<String>,
        ) -> Self {
            self.rules.push(ContextRule {
                check: Box::new(check),
                message: message.into(),
            });
            self
        }
    };
}

/// A field read as text and parsed into a `T`, or into an `Option<T>` when [optional](Self::optional).
///
/// ```
/// use form_builder::Text;
///
/// let port = Text::<u16>::new("port", "Port:").default(8080).hint("1-65535");
/// ```
#[must_use]
pub struct Text<T> {
    common: Common,
    validator: Option<Validator>,
    rules: Vec<ContextRule>,
    fallback: Option<Fallback>,
    optional: bool,
    value: PhantomData<fn() -> T>,
}

impl<T> Text<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    /// A required field: an empty answer is refused unless the field has a [default](Self::default).
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            validator: None,
            rules: Vec::new(),
            fallback: None,
            optional: false,
            value: PhantomData,
        }
    }

    /// A field that may be left empty: its value is an `Option<T>`, `None` for an empty answer, which skips the
    /// checks.
    pub fn optional(name: &str, prompt: &str) -> Self {
        Self {
            optional: true,
            ..Self::new(name, prompt)
        }
    }

    common_methods!();
    check_methods!();
}

impl<T> Text<T>
where
    T: FromStr + Clone + Display + 'static,
    T::Err: Display,
{
    /// The value an empty answer gives, shown with the prompt as `[value]`; for an optional field, `Some(value)`.
    pub fn default(mut self, value: T) -> Self {
        let shown = value.to_string();
        let optional = self.optional;
        self.fallback = Some(Fallback {
            shown,
            value: Box::new(move || {
                if optional {
                    Box::new(Some(value.clone()))
                } else {
                    Box::new(value.clone())
                }
            }),
        });
        self
    }
}

impl<T> From<Text<T>> for Field
where
    T: FromStr + 'static,
    T::Err: Display,
{
    fn from(text: Text<T>) -> Self {
        let optional = text.optional;
        let parse: Parse = Box::new(move |input| match input.parse::<T>() {
            Ok(value) if optional => Ok(Box::new(Some(value))),
            Ok(value) => Ok(Box::new(value)),
            Err(_) if optional && input.is_empty() => Ok(Box::new(None::<T>)),
            Err(error) => Err(error.to_string()),
        });
        let kind = Kind::Text {
            validator: text.validator,
            rules: text.rules,
            parse,
            fallback: text.fallback,
            optional,
        };
        Field::new(text.common, kind)
    }
}

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

impl<T: Clone + 'static> Select<T> {
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

impl<T: Clone + PartialEq + 'static> Select<T> {
    /// Starts the cursor on the option holding `value`, marked as the default; a value that is not among the options
    /// leaves the cursor on the first.
    pub fn default(mut self, value: &T) -> Self {
        self.default = self.values.iter().position(|option| option == value);
        self
    }
}

impl<T: Clone + 'static> From<Select<T>> for Field {
    fn from(select: Select<T>) -> Self {
        let values = select.values;
        let kind = Kind::Choice {
            options: select.labels,
            mode: Mode::Single,
            defaults: select.default.into_iter().collect(),
            pick: Box::new(move |chosen| Box::new(values[chosen[0]].clone())),
        };
        Field::new(select.common, kind)
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

impl<T: Clone + 'static> MultiSelect<T> {
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

impl<T: Clone + PartialEq + 'static> MultiSelect<T> {
    /// Starts with the options holding `values` chosen, marked as the defaults; values not among the options are
    /// left out.
    pub fn defaults(mut self, values: &[T]) -> Self {
        self.defaults = (0..self.values.len())
            .filter(|&index| values.contains(&self.values[index]))
            .collect();
        self
    }
}

impl<T: Clone + 'static> From<MultiSelect<T>> for Field {
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
        Field::new(multi.common, kind)
    }
}

/// A yes-or-no question, whose value is a `bool`.
///
/// ```
/// use form_builder::Confirm;
///
/// let newsletter = Confirm::new("newsletter", "Subscribe to the newsletter?").default(false);
/// ```
#[must_use]
pub struct Confirm {
    common: Common,
    default: Option<bool>,
}

impl Confirm {
    /// A question with no default: the answer must be given.
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            default: None,
        }
    }

    /// The answer an empty input gives, shown as `[Y/n]` or `[y/N]`.
    pub fn default(mut self, value: bool) -> Self {
        self.default = Some(value);
        self
    }

    common_methods!();
}

impl From<Confirm> for Field {
    fn from(confirm: Confirm) -> Self {
        Field::new(
            confirm.common,
            Kind::Confirm {
                default: confirm.default,
            },
        )
    }
}

/// A field read without showing what is typed, whose value is a `String`.
///
/// ```
/// use form_builder::Password;
/// use form_builder::validate::{self, Validator};
///
/// let password = Password::new("password", "Password:")
///     .validator(Validator::new().rule(validate::min_length(12), "At least 12 characters"))
///     .repeat("Repeat the password:");
/// ```
#[must_use]
pub struct Password {
    common: Common,
    validator: Option<Validator>,
    rules: Vec<ContextRule>,
    repeat: Option<String>,
}

impl Password {
    /// A password field: an empty answer is refused.
    pub fn new(name: &str, prompt: &str) -> Self {
        Self {
            common: Common::new(name, prompt),
            validator: None,
            rules: Vec::new(),
            repeat: None,
        }
    }

    /// Asks for the password a second time, with `prompt`, and asks again from the start when the two differ.
    pub fn repeat(mut self, prompt: impl Into<String>) -> Self {
        self.repeat = Some(prompt.into());
        self
    }

    common_methods!();
    check_methods!();
}

impl From<Password> for Field {
    fn from(password: Password) -> Self {
        let kind = Kind::Secret {
            validator: password.validator,
            rules: password.rules,
            repeat: password.repeat,
        };
        Field::new(password.common, kind)
    }
}

fn split<T, L: Into<String>>(options: impl IntoIterator<Item = (T, L)>) -> (Vec<T>, Vec<String>) {
    options.into_iter().map(|(value, label)| (value, label.into())).unzip()
}

impl<T> Text<T>
where
    T: FromStr + 'static,
    T::Err: Display,
{
    pub(crate) fn validator_if(self, validator: Option<Validator>) -> Self {
        match validator {
            Some(validator) => self.validator(validator),
            None => self,
        }
    }
}
