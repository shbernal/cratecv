# 0004. Guardrails read the laid-out frame tree, not the PDF

Status: accepted

## Context

The checks need two things a PDF gives up reluctantly: where every line sits,
and how wide it is against the measure it was broken against.

Reading the exported PDF is possible — the predecessor did exactly that,
counting `/Type /Page` and reconstructing text positions from content streams.
It works, and it is forensics. Glyph advances have to be reassembled from font
metrics, a run's logical extent has to be inferred from where the ink landed,
and every quirk of the PDF writer becomes something the checks have to know
about. Most of the predecessor's PDF test suite existed to police its renderer
rather than to say anything about the resume.

## Decision

Compile to `PagedDocument` and walk `Frame::items()`. `FrameItem::Text` carries
the run's text and `width()`, which is the advance the layout engine used;
`FrameItem::Group` nests a subframe with a transform; `FrameItem::Tag` carries
the template's markers. The PDF is exported afterwards and is never read back.

The walk asserts two things rather than tolerating them, because both would
silently corrupt every number downstream and neither is something a resume can
cause: a group transform that is not a pure translation, and a run positioned
outside its page. Both are template bugs and both exit 3.

## Consequences

Page fitting and fill ratios are measurements, not inferences, and `overflowMm`
can say "over by 23mm" instead of "over by one page".

The cost is a hard dependency on Typst's internal layout types. `Frame`,
`FrameItem`, `TextItem::width()` and the `Content` inside `Tag::Start` are not a
stable API and move between minor releases, which is why the typst family is
pinned exactly rather than by caret. Every version bump is a deliberate piece of
work on three places: the `World` impl, this walk, and the marker payload read.
