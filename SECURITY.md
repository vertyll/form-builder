# Security

## Supported versions

Fixes go into the latest release only: a vulnerability is fixed in a new patch version of the latest minor version,
and older versions are not patched.

## Reporting a vulnerability

Report it privately, never in a public issue: open
[Report a vulnerability](https://github.com/vertyll/form-builder/security/advisories/new) on the repository's Security
tab. Say which version of `form_builder` you use, on which system and terminal, and give the smallest form and the
keys that show the problem.

The report is answered within a week. Once it is confirmed, a fix is prepared in a private fork, released as a new
version, and the advisory is published with it, naming who reported the problem unless they ask otherwise.

## What counts

Form Builder reads what a user types at a terminal, or answers given ahead, and hands it to the program as values. A
vulnerability is anything that breaks that, such as:

- a password shown on the screen, in the terminal's history, or in an error or a report;
- the terminal left in raw mode or with a hidden cursor after a form ends, however it ends;
- input that makes a form hang or use memory without bound;
- a crash on input a form should refuse with a message.

A password is handed to the program as a `String`, as typed: storing it safely, such as hashing it, is the program's
work. A field that takes what it should refuse, or refuses what it should take, is a bug: open an issue for it, as
[CONTRIBUTING.md](CONTRIBUTING.md) says.
