# Contributing to gerb

Contributions are welcome and encouraged.
They can be anything from spelling corrections, art, documentation, or source code fixes & additions.
If a source code contribution is correct, functional and follows the code style and feature goals of the rest of the project, it will be merged.

**Table of contents**:

- [Important links](#important-links)
- [Developing environment](#developing-environment)
- [Testing](#testing)
- [How to submit changes](#how-to-submit-changes)
- [Choosing what to work on](#choosing-what-to-work-on)
- [How to request an enhancement, new features](#how-to-request-an-enhancement-new-features)
- [Style Guide / Coding conventions](#style-guide--coding-conventions)
- [Specific questions and answers](#specific-questions-and-answers)
  - [How do I include new images / icons?](#how-do-i-include-new-images--icons)

## Important links

- Main repository: <https://github.com/epilys/gerb>
- Bug/Issue tracker: <https://github.com/epilys/gerb/issues>
- Discussions: <https://github.com/epilys/gerb/discussions>

To privately contact the repository's owner, check their github profile for communication methods.
You can also find email addresses in git commits.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## Developing environment

You will need an operating system that is supported by gtk3 and rust.
You can install rust and cargo with the [`rustup`](https://rustup.rs) tool.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## Testing

All tests can be executed with `cargo`.
Run

```shell
cargo test
```

to run all tests.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## How to submit changes

Use github's PR functionality.
Alternatively, small patches can be emailed to the owner of the repository.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## Choosing what to work on

You can find some tasks in the bug tracker.
Additionally, tasks are annotated inside the source code with the keywords `FIXME`, `TODO`, `needs_user_doc` and others. For a list of all tags see `src/lib.rs`.
To find tag references you can use a text search tool of your choice such as `grep`, `ripgrep` or others.
The CLI tool `tagref` can also be used:

```shell
/path/to/gerb $ tagref list-refs
[ref:FIXME] @ ./src/utils.rs:106
[ref:FIXME] @ ./src/glyphs.rs:867
[ref:FIXME] @ ./src/project.rs:30
[ref:hardcoded_color_value] @ ./src/project.rs:411
...
```

You can of course filter or sort them by tag:

```shell
/path/to/gerb $ tagref list-refs | grep TODO
[ref:TODO] @ ./src/api/shell.rs:36
[ref:TODO] @ ./src/api/shell.rs:38
[ref:TODO] @ ./src/utils/range_query.rs:371
[ref:TODO] @ ./src/glyphs/obj.rs:304
/path/to/gerb $ tagref list-refs | sort -u
...
```

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## How to request an enhancement, new features

Simply open a new issue on the bug tracker.
If you are uncertain if an enhancement would be appropriate or if you would like to request feedback you can alternatively post a thread in Discussions.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## Style Guide / Coding conventions

All Rust code must be formatted by `rustfmt`, and python code by `black`.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>

## Specific questions and answers

### How do I include new images / icons?

First of all their format must be SVG.
You can convert them to PNG with a simple command, that is wrapped in the script `src/resources/svg_to_png.sh` for convenience.
You will need to install `rsvg-convert`.
For optimizing PNG files you should use `optipng`.
The location of all media resources must be in `src/resources/`.

<sup><sub><a href="#contributing-to-gerb">back to top</a></sub></sup>
