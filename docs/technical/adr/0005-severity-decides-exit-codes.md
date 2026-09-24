# 0005. Severity decides exit codes, and `maxPages` is a value

Status: accepted

## Context

The tool produces two kinds of output: a page count against a limit, and advice
about lines that left space behind. Both are guardrails, and the obvious design
is to treat every guardrail the same way and fail on any of them.

That design has a predictable ending. The only fix for a half-empty line is
rewording, and sometimes no better wording exists. A tool that fails a build for
it puts itself in charge of the prose. The first thing everyone would do is
silence the check to get their pipeline back, which destroys the signal the tool
exists to produce.

## Decision

Exit codes track severity, not which check spoke:

- `0` compiled, within `maxPages`, nothing at `error` severity. Loose lines at
  the default `warn` land here: reported, not fatal.
- `1` a check failed — over `maxPages`, or loose lines when their severity is
  `error`.
- `2` the resume is invalid, or the arguments are.
- `3` a bug in this tool: a template or compiler failure, or a frame-walk
  assertion.

`looseLines.severity` is the only supported way to silence that check, so
silencing never has to be faked by widening a threshold until nothing trips —
which would read back as a real measurement.

`maxPages` has no severity. It *is* the knob: a second way to express the same
intent would quietly disable the guarantee the tool leads with.

`build` writes nothing on exit 1 unless `--allow-overflow`, so the last good PDF
at that path survives a bad run. The flag decides whether a file is written; it
does not change the verdict, so a build with `--allow-overflow` over the limit
still exits 1.

## Consequences

The default install never fails CI for advice, which is what keeps the advice
worth reading. A caller who does want loose lines enforced sets
`severity: error` and gets exit 1 without any other change.

`ok` in the JSON report tracks the exit code rather than "nothing was found". A
caller branching on `ok` is asking whether the run succeeded; `looseLines`
answers whether there is anything to fix. That pairing is stated in the docs,
because a reader who guesses will guess the other way half the time.
