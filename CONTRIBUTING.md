# Contributing

Thank you for helping with Form Builder. This document says how a change gets in.

## Before you start

- **A bug:** open an issue with the smallest form that shows the problem, what you typed or pressed, what happened,
  and what you expected. Say which version of `form_builder` you use, and on which system and terminal.
- **A new kind of field or option:** open an issue first. Describe the form it makes possible and how its prompt
  looks, so its place among the builders is agreed before the code is written.
- **A fix to the documentation:** a pull request is enough.

## Making the change

1. Branch from `main`, in a fork or, with write access, in the repository itself. Name the branch after the change,
   its type first: `feat/date-field`, `fix/scroll-at-the-end`, `docs/preset-examples`.
2. Make the change with its tests. What runs apart from a terminal, such as a selection or the editing of a line,
   is tested on its own; a whole form is tested through a `Console` that replays answers, as `tests/form.rs` does.
3. Update the documents the change touches: [Features](docs/features.md) for what a form can do, the
   [Changelog](CHANGELOG.md) under the next version, and [CONTENTS.md](CONTENTS.md) for a new document.
4. Run every check, as CI does:

   ```bash
   cargo fmt --all --check
   cargo clippy --workspace --all-targets --all-features -- -D warnings
   cargo test --workspace --all-features
   cargo run --example demo
   ```

   The demo is how a change to the terminal is seen working.

## Commits and pull requests

Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/): a type, a colon and
what the commit does, in the imperative and in lower case.

| Type       | For                                                 |
|------------|-----------------------------------------------------|
| `feat`     | a new kind of field, option or console              |
| `fix`      | a form that asks, shows or reads what it should not |
| `docs`     | the documentation only                              |
| `refactor` | a change of the code that changes no behaviour      |
| `test`     | tests only                                          |
| `build`    | the toolchain, dependencies and CI                  |

A change that breaks code that compiles today adds `!` after the type, as `feat!:`, and says what to change in its
body.

Every change reaches `main` through a pull request, the maintainers' own included; `main` takes no direct pushes. A
pull request does one thing, says why, and needs CI to pass before it is merged. It is squash-merged, so its title,
written as a commit message above, becomes the one commit on `main` and the line of the release notes that names the
change.

## License

By contributing, you agree that your contribution is released under the [MIT License](LICENSE), as the rest of Form
Builder.
