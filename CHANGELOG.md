# Changelog

Every release, newest first. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## 2.0.1

### Changed

- `#[derive(Form)]` is built on `syn` 3; the code it generates is unchanged.

## 2.0.0

A rewrite of the API around values read as their types, errors to match, and a terminal that works on Linux, macOS
and Windows. [Migrating from 1.x to 2.0](docs/migration.md) shows each change, before and after.

### Added

- A builder for each kind of field, `Text`, `Select`, `MultiSelect`, `Confirm` and `Password`, added with
  `FormBuilder::field`.
- Yes-or-no questions, and passwords typed unseen and asked twice with `repeat`.
- Defaults, shown in grey after the prompt, and hints; `d` in a choice goes back to its defaults.
- Checks of the parsed value with `Text::check`, and rules that compare with earlier answers with `rule_with`.
- Fields asked only when a condition on earlier answers holds, with `when`.
- The step of every prompt, `[3/8]`, and the `*` of every required field.
- A review of the answers before the form is done, where any can be changed, or kept with Enter or Esc.
- Lists that scroll when taller than the terminal, with the keys shown under the prompt.
- Text fields and passwords typed key by key, with the arrows, Home, End, Delete and Esc.
- `Preset`, which answers a form from the environment or the command line, where nobody can.
- `Messages`, which translates every text the form shows of its own.
- `#[derive(Form)]`, with the `derive` feature, which asks for a struct field by field.
- `Console`, to fill a form from anything other than the terminal, and `Form` and `FormBuilder` as `Send` and `Sync`.

### Changed

- Every failure is a `form_builder::Error` instead of a `String`.
- `Form::get` reads every field as its type, `Vec<T>` for a multiselect, and replaces `get_value` and
  `get_value_vec`.
- `Option<T>`, through `add_optional_field`, replaces the `Optional` type.
- `Validator::new().rule(check, message)` builds a validator, and the checks moved to the `validate` module.
- A required text field refuses an empty answer.
- A choice is drawn in place and leaves one line with the answer, instead of clearing the screen.

### Fixed

- A form no longer asks forever when its input ends: it returns `Error::EndOfInput`.
- The letters `A` and `B` no longer move a choice, and the terminal gets its usual mode back however a choice ends.
- A select or multiselect with no options returns `Error::NoOptions` instead of panicking.
- Options holding commas or quotes come back whole from a multiselect.
