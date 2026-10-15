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

## Defaults and hints

A default is what Enter on an empty answer gives. It is shown in grey after the prompt, and a hint in grey before it:

```rust
Text::<u16>::new("port", "Port:").default(8080).hint("1-65535")
```

```text
Port * (1-65535) [8080]:
```

A `Select` starts on its default option, marked `(default)`; a `MultiSelect` starts with its defaults chosen. A
`Confirm` shows its default as the capital letter: `[y/N]`.

## Choices

A select or multiselect is drawn where the form is, below the earlier answers, and redrawn in place as the user
moves. A grey line under the prompt shows the keys:

| Key              | Does                                                             |
|------------------|------------------------------------------------------------------|
| `↑` `↓`, `k` `j` | moves the cursor                                                 |
| Space            | marks or unmarks an option of a multiselect                      |
| Enter            | chooses the option under the cursor, or confirms the marked ones |
| `d`              | goes back to the defaults, when the field has them               |
| Esc, Ctrl+C      | cancels, and `fill` returns `Error::Cancelled`                   |

A list taller than the terminal shows the part around the cursor, with how many options are above and below it.
Once chosen, the list gives way to one line with the answer, such as `Plan *: Pro`. A multiselect may limit how many
options are chosen, and needs at least one.

## Passwords

A `Password` shows `*` for each character typed. With `repeat`, it asks a second time and starts over when the two
differ. When standard input is not a terminal, as in a pipe, it is read as a plain line.

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
