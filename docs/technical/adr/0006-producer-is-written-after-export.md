# 0006. PDF origin metadata is written after export

Status: accepted

## Context

A resume is read by people who are not the recipient of the application, and a
PDF announces where it came from. Being able to set Creator and Producer is
therefore a feature of the tool rather than a curiosity.

Typst covers part of this. `DocumentInfo` supplies title, author, keywords and
date, which `typst-pdf` writes into both the Info dictionary and the XMP packet.
`PdfOptions` exposes `creator`, which lands in `/Creator` and
`xmp:CreatorTool`.

It does not cover Producer. `PdfOptions` has no field for it, and as of 0.15.1
the writer emits no Producer at all — neither in the Info dictionary nor in XMP.
A document with a Creator and no Producer is itself a signature.

## Decision

Creator goes through `PdfOptions`, the route Typst supports.

Producer is written by a pass over the finished bytes, after `typst_pdf` returns
and after the guardrails have passed, into the Info dictionary and the XMP
packet **together**. A reader that trusts one and not the other would otherwise
see a contradiction, which is worse than either value alone. `lopdf` does the
rewrite; it has no system prerequisites, and a round trip through it leaves the
structure tree, the font subsets and the text extraction intact.

Values outside ASCII are written as UTF-16BE with a byte order mark, because a
PDF text string without one is PDFDocEncoded and not UTF-8.

## Consequences

The ceiling is worth stating rather than implying: **this changes metadata and
nothing else**. A document claiming a different origin still carries this tool's
font subsets, object layout and structure tree. The feature makes output quiet,
it does not disguise it.

`date` defaults to the resume's modification time rather than the clock, so two
builds of one resume are byte-identical. That property is asserted in the tests,
because it is the kind of thing that breaks silently.

If a later Typst release exposes Producer, the pass can go and this record is
the reason to check.
