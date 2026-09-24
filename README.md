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
- Output is tagged PDF with embedded fonts, so the text comes back out in
  reading order.
- `check --json` is the primary interface. It is meant to be driven in a loop
  by a person or by an assistant.

Status: early. Not yet released.

## Install

Not published yet. Build from source with a Rust toolchain:

```bash
cargo install --path .
```

## Use

```bash
cratecv build cv.yaml -o cv.pdf     # compile, refusing to overflow a page
cratecv check cv.yaml --json        # layout report, no file written
cratecv preview cv.yaml -o cv.png   # what the page looks like
cratecv watch cv.yaml               # recompile while editing
```

## License

MIT. See [LICENSE](LICENSE).
