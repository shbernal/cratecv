# cratecv

Compile a YAML resume into a one-page PDF, and get told what is wrong with it
in a form a script can read.

A resume has hard constraints that a word processor will not enforce: it has to
fit one page, its bullets should fill the width they are given, and it has to
survive the text extractor an applicant tracking system will run over it.
`cratecv` treats those as compile errors rather than as matters of taste.

- One page is a build failure, not a warning. Nothing is written when the
  resume runs over, and the report says by how much.
- Bullets that end far short of the line are reported with their path into the
  YAML and their fill ratio, so the next revision can be precise.
- Output is tagged PDF with embedded fonts and passes PDF/UA-1 validation, so
  the text comes back out of an applicant tracking system in reading order.
- `check --json` is the primary interface. It is meant to be driven in a loop
  by a person or by an assistant.

Status: early.

## Install

Download a binary for your platform from the
[releases](https://github.com/shbernal/cratecv/releases), or build from source
with a Rust toolchain:

```bash
cargo install --git https://github.com/shbernal/cratecv
```

The binary carries its own fonts and its own typesetter. There is no browser,
no Node, and no system font to install, which is also why the same YAML lays
out identically on every machine.

## Use

```bash
cratecv init                        # a starter resume, and a config if there is none
cratecv build cv.yaml -o cv.pdf     # compile, refusing to overflow a page
cratecv check cv.yaml --json        # layout report, no file written
cratecv preview cv.yaml -o cv.png   # what the page looks like
cratecv watch cv.yaml               # recompile while editing
cratecv schema                      # JSON Schema for the resume format
```

## Docs

- [The resume format](docs/resume-schema.md)
- [Guardrails](docs/guardrails.md) — the four kinds of half-empty line, and what
  `ok` does and does not mean
- [The command line](docs/cli.md) — commands and exit codes
- [Configuration](docs/configuration.md) — the four layers
- [Architecture](docs/architecture.md), and the
  [decision records](docs/technical/adr/)

## For assistants

`skills/cratecv/SKILL.md` teaches the loop: edit the YAML, run
`cratecv check --json`, fix what the report names by path, repeat. Copy or link it into
your project. It covers the tool only; what the resume says belongs to your
own skill. A project that wraps `cratecv` in a script should pass the command
and its arguments through unchanged, so the skill still reads true.

## License

MIT. See [LICENSE](LICENSE).
