# Features

What a form can ask, how it shows it, and how its answers are read. The API documentation on
[docs.rs](https://docs.rs/form_builder) has every method.

## Fields

Each kind of field has its own builder, added to a form with `FormBuilder::field`, and is read back with
`Form::get` as the type it holds.

| Builder               | Asks for                       | Read as     |
|-----------------------|--------------------------------|-------------|
| `Text::<T>::new`      | text parsed into any `FromStr` | `T`         |
| `Text::<T>::optional` | text that may be left empty    | `Option<T>` |
| `Select`              | one of its options             | `T`         |
| `MultiSelect`         | one or more of its options     | `Vec<T>`    |
| `Confirm`             | yes or no                      | `bool`      |
| `Password`            | text typed unseen              | `String`    |

`add_field`, `add_optional_field`, `add_select` and `add_multiselect` add the plain cases in one call.

A required text field refuses an empty answer, and a value that does not parse is reported with the reason its
`FromStr` gives; either way the field asks again. A field is required unless it is an optional text field.

## Steps

Every prompt starts with its step, in grey: `[3/8] Port *:`. It counts the fields of the form, so a skipped
conditional field leaves a gap. `no_steps()` leaves the steps out.

## Defaults and hints

A default is what Enter on an empty answer gives. It is shown in grey after the prompt, and a hint in grey before it:

```rust
Text::<u16>::new("port", "Port:").default(8080).hint("1-65535")
```

```text
Port * (1-65535) [8080]:
```

A `Select` starts on its default option, marked `(default)`; a `MultiSelect` starts with its defaults chosen. A
`Confirm` shows the short answers as a hint and its default as the word: `Newsletter? * (y/n) [no]`.

## Choices

A select or multiselect is drawn where the form is, below the earlier answers, and redrawn in place as the user
moves. A grey line under the prompt shows the keys:

| Key                                           | Does                                                             |
|-----------------------------------------------|------------------------------------------------------------------|
| `↑` `↓`, `k` `j`                              | moves the cursor                                                 |
| Space                                         | marks or unmarks an option of a multiselect                      |
| Enter                                         | chooses the option under the cursor, or confirms the marked ones |
| `d`                                           | goes back to the defaults, when the field has them               |
| Esc, while changing an answer from the review | goes back to the review and keeps the answer                     |
| Esc, Ctrl+C                                   | cancels, and `fill` returns `Error::Cancelled`                   |

A list taller than the terminal shows the part around the cursor, with how many options are above and below it.
Once chosen, the list gives way to one line with the answer, such as `Plan *: Pro`. A multiselect may limit how many
options are chosen, and needs at least one.

## Passwords

A `Password` shows `*` for each character typed. With `repeat`, it asks a second time, at the same step, and starts
over when the two differ. When standard input is not a terminal, as in a pipe, it is read as a plain line.

## Conditional fields

A field with `when` is asked only when its condition holds for the answers before it; otherwise it is skipped, and
reading it gives `Error::Skipped`:

```rust
Text::<String>::new("vat", "VAT ID:").when(|answers| answers.is("company", &true))
```

## Validation

A `Validator` checks the trimmed input of a text field or a password before it is taken, rule by rule; the first
rule that fails shows its message, and the field asks again. The checks in `validate` cover the common cases, and
any closure can be a rule:

```rust
Validator::new()
    .rule(validate::not_empty, "Name cannot be empty")
    .rule(|input: &str| !input.contains(' '), "One word, please")
```

`rule_with` gives a rule the answers before the field, to compare with them, as a maximum that may not be below an
earlier minimum.

`check` tests the value once it is parsed, as the type the field holds, and shows its message alone when it fails:

```rust
Text::<u16>::new("port", "Port:").check(|port| *port >= 1024, "Use a port above 1023")
```

| Check                            | Accepts                                         |
|----------------------------------|-------------------------------------------------|
| `not_empty`                      | any input but an empty one                      |
| `name`                           | input without digits                            |
| `email`                          | the shape `local@domain.tld`                    |
| `min_length(n)`, `max_length(n)` | at least or at most `n` characters              |
| `alphabetic`                     | letters only                                    |
| `integer`, `float`               | a number an `i64` or an `f64` reads             |
| `date`, `time`                   | `YYYY-MM-DD` and `HH:MM:SS`                     |
| `url`                            | an `http` or `https` URL                        |
| `phone_number`                   | up to 15 digits, with an optional leading `+`   |
| `postal_code`                    | five digits, with an optional `-` and four more |
| `credit_card`                    | four groups of four digits, with or without `-` |
| `uuid`                           | a hyphenated UUID                               |

The formats behind them are listed in [Standards](../STANDARDS.md).

## Review

With `review()`, the form lists the answers once every field is asked, a password as `********` and an empty
optional field as `-`, with `Submit` under the cursor. Choosing an answer asks its field again, showing the answer it
has as its default: Enter on an empty input keeps it, and in a choice, which starts on it, Esc goes back to the review.
A changed answer asks the fields its change calls for and skips those it rules out, and keeps the others. Esc in the
review itself cancels the form.

## Answers given ahead

A `Preset` is a console that answers each field by its name, so the same form runs where nobody can answer it, as in
CI or a script:

```rust
let mut console = Preset::from_env("APP_")
    .merge(Preset::from_args(std::env::args().skip(1)))
    .or(Terminal);
form.fill_with(&mut console)?;
```

| Source                            | Answers                                                |
|-----------------------------------|--------------------------------------------------------|
| `Preset::new([("port", "8080")])` | the pairs given                                        |
| `Preset::from_env("APP_")`        | `APP_PORT=8080` for the field `port`                   |
| `Preset::from_args(args)`         | `port=8080` or `--port=8080`                           |
| `.answer("port", "8080")`         | one more answer; `.merge(other)` adds another preset's |

A select takes the label of an option, case aside, a multiselect labels separated by commas, and a yes-or-no
question the form's words for yes and no. A field without an answer takes its default, or `None` when it is
optional, and otherwise goes to the console given with `or`; with none, the form ends with `Error::EndOfInput`. An
answer the form refuses ends it with `Error::Refused` and the reason, or is asked again through `or`. A preset
without `or` skips the review and the second entry of a password.

## Derive

With the `derive` feature, `#[derive(Form)]` asks for a struct field by field:

```rust
use form_builder::{Form, FromForm};

#[derive(Form)]
struct SignUp {
    first_name: String,
    #[form(prompt = "Port:", default = 8080, hint = "1024-65535")]
    port: u16,
    #[form(password, repeat = "Repeat the password:")]
    password: String,
    newsletter: bool,
    company: Option<String>,
}

let sign_up = SignUp::fill()?;
```

The type of a field decides what is asked: `bool` a yes-or-no question, `Option<T>` an optional text field, and any
other type a required text field. The prompt is the field's name as words, `First name:`, unless `prompt` gives it.

| Attribute        | Does                                                               |
|------------------|--------------------------------------------------------------------|
| `prompt = "..."` | the prompt                                                         |
| `hint = "..."`   | a hint shown with the prompt                                       |
| `default = expr` | the answer an empty input gives                                    |
| `password`       | asks a `String` without showing it                                 |
| `repeat = "..."` | with `password`, asks it a second time                             |
| `skip`           | leaves the field out of the form, filled with `Default::default()` |

`SignUp::form()` gives the builder before it is built, to add `review()` or messages, and `SignUp::from_form(&form)`
reads the struct from a filled form. Selects and multiselects are built with their own builders.

## Required marks

The prompt of every required field carries `*` before its closing colon: `Name:` is asked as `Name *:`.
`required_marker("(required)")` uses another mark, and `no_required_marker()` leaves it out.

## Languages

The texts a form shows of its own, such as `This field is required`, the keys of a choice and the words of a
yes-or-no question, are in `Messages`, which `FormBuilder::messages` replaces:

```rust
FormBuilder::new().messages(Messages {
    required: "To pole jest wymagane".to_owned(),
    yes: "tak".to_owned(),
    no: "nie".to_owned(),
    ..Messages::default()
})
```

## Errors

Every failure is an `Error` variant, which implements `std::error::Error`:

| Variant      | When                                                                        |
|--------------|-----------------------------------------------------------------------------|
| `Io`         | the terminal could not be read or written                                   |
| `EndOfInput` | the input ended, as a closed standard input does, before a field was filled |
| `Cancelled`  | the user pressed Esc or Ctrl+C in a choice or a password                    |
| `Refused`    | a preset answer was refused where nobody can answer again                   |
| `NoOptions`  | a select or multiselect has no options                                      |
| `NotFound`   | `get` names no field of the form                                            |
| `NotFilled`  | `get` reads a field not asked yet                                           |
| `Skipped`    | `get` reads a field whose condition kept it from being asked                |
| `WrongType`  | `get` asks for another type than the field holds                            |

## Other consoles

`Form::fill` asks in the terminal. `Form::fill_with` asks through any `Console`, which reads lines, reads passwords,
reports refused input and makes choices; a console that replays answers tests a form without a terminal. A `Form`
is `Send` and `Sync`, so it can be filled on another thread, as `spawn_blocking` does in an async program.

## Platforms

Linux, macOS and Windows, through [crossterm](https://github.com/crossterm-rs/crossterm). Colours follow the
terminal, and `NO_COLOR` turns the grey off.
