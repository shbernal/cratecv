# Guardrails

Two checks run against the laid-out page: whether it fits, and whether its lines
fill the width they were given. Both are measurements taken from Typst's frame
tree rather than inferences from the PDF — see
[ADR 0004](technical/adr/0004-guardrails-read-the-frame-tree.md).

## One page

`maxPages` defaults to 1. When the content runs over, `overflowMm` says how far
past the last allowed page it reaches — computed from the position of the last
run, not estimated — so the message can say "over by 23mm" rather than "over by
a page". When it fits, `remainingMm` says how much of the last page is left
below the last line's baseline, down to the bottom margin the theme marks.

Page count has no severity. It is the guarantee, and a second way to express the
same intent would quietly disable it. `build` writes nothing when it is
exceeded, so the last good PDF survives, unless `--allow-overflow` says
otherwise.

## Half-empty lines

Only text written to fill its width is measured: bullets, and skill values in
the `rows` layout. A date, a role or a heading is short because it is short.
Badges are not measured either — a badge is a box sized to its own text.

Four kinds, each with its own threshold:

| Kind | Trips when | Default |
| --- | --- | --- |
| `shortLine` | a block that never wrapped fills less than `shortLine` | `0.75` |
| `widow` | the last line holds at most `widowWords` words | `1` |
| `shortLastLine` | the last line fills less than `shortLastLine` | `0.35` |
| `looseLine` | an earlier line leaves more than `looseLineGap` empty | `0.2` |

A `looseLine` names the word that did not fit, in `bumped`. A `shortLine`
carries `roomWords`, roughly how many more words of the same length would fit.

`wastedLines` sums `1 - fill` over every finding: the space they add up to, in
whole lines. It is what tells you whether there is room for another bullet.

These four numbers were tuned against real resumes, and they are the only
numbers here with provenance. They are settings, not constants — see
[configuration](configuration.md) — but moving one changes what the report
means, so change it deliberately.

### The only fix is rewriting

A half-empty line is fixed by rewording the sentence. Padding it with filler
defeats the point, and shrinking the type just moves the problem: every
threshold is a ratio against a fixed layout.

That is why loose lines are advice rather than failures by default. A tool that
failed a build over prose would put itself in charge of the prose, and everyone
would silence it. See
[ADR 0005](technical/adr/0005-severity-decides-exit-codes.md).

## `ok` is not "nothing was found"

`ok` tracks the exit code. A resume with four findings at the default `warn`
severity is:

```json
{ "ok": true, "looseLines": [ ... ], "wastedLines": 2.5 }
```

A caller branching on `ok` is asking whether the run succeeded. `looseLines`
answers whether there is anything to fix. They are different questions and the
report answers both.

The settings the numbers were measured against are echoed back in `settings`, so
a report is readable on its own. In particular, `looseLines: []` with
`looseLineSeverity: "silent"` is a check that never ran, not a clean resume.
