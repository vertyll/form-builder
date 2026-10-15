# Migrating from 1.x to 2.0

Version 2.0 reads values as the types the fields hold, reports failures as an `Error` you can match, and draws its
choices with [crossterm](https://github.com/crossterm-rs/crossterm), so it runs on Windows too. Most forms change in
three places: how a validator is written, how a value is read, and the error type of the function that fills the form.

## At a glance

| 1.x                                                | 2.0                                                             |
|----------------------------------------------------|-----------------------------------------------------------------|
| `Result<_, String>`                                | `Result<_, form_builder::Error>`                                |
| `form.get_value::<T>(name)`                        | `form.get::<T>(name)`                                           |
| `form.get_value_vec::<T>(name)`                    | `form.get::<Vec<T>>(name)`                                      |
| `add_field::<Optional<T>>`, the `Optional` type    | `add_optional_field::<T>`, read as `Option<T>`                  |
| `Validator::new(vec![(check, Some("message"))])`   | `Validator::new().rule(check, "message")`                       |
| `ValidationMethods::validate_email` and the others | `validate::email` and the others: see [Validation](#validation) |
| `ValidationMethods::min_length(value, n)`          | `validate::min_length(n)`                                       |
| the `field`, `input` and other modules             | private; `Console` and `Terminal` are the way to supply input   |

## Errors

Every fallible call returns `form_builder::Error` instead of a `String`. It implements `std::error::Error`, so `?`
carries it into `Box<dyn Error>` or any error type that converts from it, and its variants say what went wrong:

```rust
use form_builder::Error;

match form.fill() {
    Ok(()) => {}
    Err(Error::EndOfInput(field)) => eprintln!("the input ended before {field}"),
    Err(Error::Cancelled(field)) => eprintln!("{field} was cancelled"),
    Err(error) => return Err(error.into()),
}
```

A form no longer asks forever when the standard input ends, as it does in a pipe or in CI: it returns
`Error::EndOfInput`. A select or multiselect with no options returns `Error::NoOptions` instead of panicking, and Esc
or Ctrl+C in a choice returns `Error::Cancelled`.

## Reading values

One method reads every field, as the type it holds:

```rust
// 1.x
let age: u32 = form.get_value("age")?;
let hobbies: Vec<String> = form.get_value_vec("hobbies")?;

// 2.0
let age: u32 = form.get("age")?;
let hobbies: Vec<String> = form.get("hobbies")?;
```

A select gives back the value of the option chosen, and a multiselect a `Vec` of them, as they were given to the
builder: the values are no longer printed and parsed again, so options holding commas or quotes come back whole, and
the option type needs only `Clone`. Reading a field as another type returns `Error::WrongType`.

## Optional fields

The `Optional` type is gone; an optional field holds a standard `Option`:

```rust
// 1.x
.add_field::<Optional<u32>>("width", "Width (optional):", None)
let width: Optional<u32> = form.get_value("width")?;

// 2.0
.add_optional_field::<u32>("width", "Width (optional):", None)
let width: Option<u32> = form.get("width")?;
```

An empty answer gives `None` and skips the field's validator. The arithmetic `Optional` offered, such as adding two
values where a missing one counts as nothing, is left to `Option`'s own methods: `width.unwrap_or(0) + 20`.

## Validation

A validator is built rule by rule, each rule a check and the message shown when the input fails it:

```rust
// 1.x
Validator::new(vec![
    (ValidationMethods::not_empty, Some("Name cannot be empty")),
    (ValidationMethods::validate_name, Some("Name cannot contain numbers")),
])

// 2.0
Validator::new()
    .rule(validate::not_empty, "Name cannot be empty")
    .rule(validate::name, "Name cannot contain numbers")
```

A rule can be any closure, `.rule(|input: &str| input.ends_with('!'), "Shout it")`, where 1.x took only a function.
The checks moved from `ValidationMethods` to the `validate` module:

| 1.x                                 | 2.0                                     |
|-------------------------------------|-----------------------------------------|
| `validate_name`                     | `validate::name`                        |
| `validate_email`                    | `validate::email`                       |
| `not_empty`                         | `validate::not_empty`                   |
| `min_length(value, n)`              | `validate::min_length(n)`               |
| `max_length(value, n)`              | `validate::max_length(n)`               |
| `is_alpha`                          | `validate::alphabetic`                  |
| `is_integer`                        | `validate::integer`                     |
| `is_float`                          | `validate::float`                       |
| `is_date`, `is_time`, `is_url`      | `validate::date`, `time`, `url`         |
| `is_phone_number`, `is_postal_code` | `validate::phone_number`, `postal_code` |
| `is_credit_card`, `is_uuid`         | `validate::credit_card`, `uuid`         |

`min_length` and `max_length` count characters, where 1.x counted bytes, so `żółw` is four long rather than seven.
`integer` accepts any `i64`, where 1.x accepted an `i32`.

## Text fields

A field's type still needs `FromStr`, but its parse error must implement `Display` rather than `Debug`, as the errors
of the standard library's types and most others do. A value that does not parse is reported as `Invalid input:`
followed by that error, and the field asks again.

## Input

A choice is drawn where the form is, below the earlier answers, and leaves one line with the answer, such as
`Plan: Pro`, where 1.x cleared the whole screen for each key.

The `input` module and its functions are private. To fill a form from something other than the terminal, such as
answers replayed in a test, implement `Console` and call `form.fill_with(&mut console)`; `Terminal` is the console
`fill` uses.
