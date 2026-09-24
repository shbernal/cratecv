# 0002. Text is attributed to YAML paths by frame-tree markers

Status: accepted

## Context

The report's value is that it names the bullet. "A line is 42% full" is useless;
"`sections[1].entries[0].details[2]` fills 42% of its line" is actionable. So
every measured line has to be traced back to the YAML it came from, and the
measure it was broken against has to be known exactly.

The obvious alternative is text matching: take the laid-out runs, take the
bullet strings, and join them by content. It works until it does not. Runs are
split at every styling change, so a bullet with a bold lead-in arrives in
pieces. Smart quotes, ligatures and hyphenation mean the laid-out text is not
the input text, so both sides need normalizing. And two bullets that open with
the same words stay ambiguous no matter how good the normalization is.

Inferring the available width from the enclosing frame has the same shape of
problem. A bullet sits inside list indentation, a grid cell and a block, each
contributing insets. Recovering the measure from the outside means
reconstructing the layout engine's decisions, which is precisely the inference
this design exists to avoid.

## Decision

The template marks each measurable block with a `metadata` element carrying its
YAML path and the width the layout engine gave it:

```typst
#let measured(path, body) = block(width: 100%, {
  place(context layout(size => metadata((path: path, available: size.width / 1pt))))
  body
})
```

`FrameItem::Tag(Tag::Start(content, _))` exposes that `Content` directly, so the
payload is read off the frame tree with no introspector and with per-item
positions, which the introspector's `position()` does not give.

Only bullets and skill rows are marked. Dates, roles, organizations and headings
are short because they are short, so they are never measured and need no
exemption rule: an unmarked block is simply absent from the walk. Badges are
unmarked for the same reason — a badge is a box sized to its own text.

## Consequences

Attribution is exact and unambiguous, and `available` is the number the layout
engine actually used rather than one reconstructed from the outside.

It depends on a Typst internal: the `Content` inside `Tag::Start`, and
`MetadataElem` being reachable from it. That is why the typst family is pinned
exactly. `tests/markers.rs` is the entire insurance policy — there is no
fallback path in the code, and if that test fails, nothing can be attributed.

A theme that forgets to mark a block does not report a wrong number; it reports
nothing for that block. Silence is the failure mode, which is why the test
asserts a count rather than only a shape.
