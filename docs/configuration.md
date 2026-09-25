# Configuration

Settings are stated at four levels, most specific winning:

1. Built-in defaults, compiled in.
2. `~/.config/cratecv/config.yaml` — every resume on this machine.
3. A `cratecv:` block in a resume — that one document, everywhere.
4. Command-line flags — this invocation.

Both files are YAML and both go through the same strict loader, so a typo'd key
in the config gets the same diagnostic, at the same line and column, as a typo'd
key in a resume. `$XDG_CONFIG_HOME` is honoured. `cratecv init` writes a
commented starter config when there is none.

There is no convention directory for resumes. A resume is a personal document
that wants your own version control, not this tool's opinion about filesystems.

## The tree

```yaml
theme: default

pdf:
  creator: cratecv
  producer: cratecv
  date: mtime               # mtime | none | a quoted timestamp
  keywords: []

check:
  maxPages: 1
  looseLines:
    severity: warn          # error | warn | silent
    shortLine: 0.75
    shortLastLine: 0.35
    looseLineGap: 0.2
    widowWords: 1

output:
  dir: .

preview:
  dpi: 150
```

A resume's `cratecv:` block takes `theme`, `pdf` and `check`. `output` and
`preview` describe this machine and this invocation, so they only belong in the
config file.

```yaml
# academic-cv.yaml
cratecv:
  check:
    maxPages: 2
    looseLines:
      severity: silent
```

### Values that dodge YAML

`severity: silent`, not `off` — `off` is a boolean in YAML and would either
become `false` or fail confusingly. A fixed `date` is quoted for the same
reason: unquoted, it is a YAML date rather than a string.

### What is not here

There is no typography section. Page size, margins, base font size, density and
accent colour are theme-internal, because every number the report gives you is
measured against the laid-out page. See
[ADR 0003](technical/adr/0003-typography-is-theme-internal.md).

Title and author are not here either: they come from the resume's `name`, and
the title reads `<name> - Resume`.

## Severity, and what the exit code means

`looseLines.severity` is the only supported way to switch that check off, so
silencing never has to be faked by widening a threshold until nothing trips —
which would read back as a real measurement. `maxPages` has no severity: it is
the knob, and the report echoes both.

A silenced check reports as `looseLineSeverity: "silent"` with an empty
`looseLines`, which is not the same as a clean resume and must not be read as
one.

## PDF metadata

`pdf.creator` and `pdf.producer` are written into the Info dictionary and the
XMP packet together.

This changes metadata and nothing else. A document claiming a different origin
still carries this tool's font subsets, object layout and structure tree. The
feature makes output quiet, not disguised.

`date` defaults to the resume file's modification time, so two builds of the
same resume are byte-identical. `none` omits the field, which is itself
conspicuous on a document claiming an origin that always stamps one. A fixed
timestamp is normalized to UTC, so `"2024-03-11T09:12:00+01:00"` is written as
`08:12` UTC.
