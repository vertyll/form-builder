# Standards

The specifications this library follows, what for, and where the documentation covers it.

| Standard               | Title                                                     | Used for                                      | Explained in                                        |
|------------------------|-----------------------------------------------------------|-----------------------------------------------|-----------------------------------------------------|
| [ISO 8601][iso8601]    | Date and time format                                      | `validate::date` and `validate::time`         | [Features: Validation](docs/features.md#validation) |
| [E.164][e164]          | The international public telecommunication numbering plan | `validate::phone_number`                      | [Features: Validation](docs/features.md#validation) |
| [RFC 9562][uuid]       | Universally Unique IDentifiers (UUIDs)                    | `validate::uuid`                              | [Features: Validation](docs/features.md#validation) |
| [RFC 3986][uri]        | Uniform Resource Identifier: Generic Syntax               | `validate::url`, for `http` and `https` only  | [Features: Validation](docs/features.md#validation) |
| [ECMA-48][ecma48]      | Control functions for coded character sets                | drawing choices and grey text in the terminal | [Features: Choices](docs/features.md#choices)       |
| [NO_COLOR][nocolor]    | Disabling ANSI colour output                              | turning the grey of hints and defaults off    | [Features: Platforms](docs/features.md#platforms)   |
| [SemVer 2.0.0][semver] | Semantic Versioning                                       | release versions                              | [Migrating from 1.x to 2.0](docs/migration.md)      |

The checks follow the shape these standards give, not all of their rules: `validate::date` takes `2023-02-31`, since
it checks the ranges of month and day but not the calendar, and `validate::url` an `http` or `https` URL with no
spaces, not the full grammar of RFC 3986. `validate::email` checks the shape `local@domain.tld`, not the grammar of
an address in RFC 5322, and `validate::postal_code` the five or nine digits of a US ZIP code.

[iso8601]: https://www.iso.org/iso-8601-date-and-time-format.html
[e164]: https://www.itu.int/rec/T-REC-E.164
[uuid]: https://www.rfc-editor.org/rfc/rfc9562
[uri]: https://www.rfc-editor.org/rfc/rfc3986
[ecma48]: https://ecma-international.org/publications-and-standards/standards/ecma-48/
[nocolor]: https://no-color.org/
[semver]: https://semver.org/spec/v2.0.0.html
