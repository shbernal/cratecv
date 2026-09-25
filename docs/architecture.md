# Architecture

Data flows one way.

```
resume.yaml ─► parse + validate ─► Resume ─► canonical JSON ─► Typst ─► PagedDocument
                     │                                                        │
                     ▼                                                        ▼
                diagnostics ──────────────────► Report ◄──── frame walk ── measurements
                                                   │
                                                   ▼
                                            PDF, written last
```

- **`schema/`** parses the YAML once into a span-carrying node tree, deserializes
  strict Rust types from it, then checks the rules serde cannot state. Every
  diagnostic carries a canonical path, and that path is the join key everything
  downstream uses.
- **`world/`** is the Typst `World`: a virtual file tree holding the built-in
  template, the resume re-serialized from the validated types, and the contact
  icons. Nothing is read from disk, and no system font is consulted
  ([ADR 0001](technical/adr/0001-fonts-compiled-into-the-binary.md)).
- **`templates/default.typ`** lays the page out and marks each measurable block
  with its YAML path and the measure the layout engine gave it, plus where
  the last page's content area ends
  ([ADR 0002](technical/adr/0002-frame-tree-markers-attribute-text.md)).
- **`layout/`** walks the laid-out frames and turns those markers into blocks,
  lines and fill ratios
  ([ADR 0004](technical/adr/0004-guardrails-read-the-frame-tree.md)).
- **`report/`** judges the measurements against the resolved settings and
  produces the JSON contract.
- **`config/`** folds four layers — defaults, config file, the resume's own
  block, flags — into the settings the report echoes.
- **`pdf/`** writes the origin metadata Typst does not expose, after export and
  after the guardrails pass
  ([ADR 0006](technical/adr/0006-producer-is-written-after-export.md)).

The PDF is written last, and only if the page count is within the limit. It is
never read back.

## Why the frame tree

`Frame::items()` yields positioned items, and a text run carries its own advance
width. That is exact line-box data, so page fitting and line fill are
measurements rather than reconstructions of what a PDF writer did. It costs a
hard dependency on Typst's internal layout types, which is why the typst family
is pinned exactly rather than by caret.

## Why the library holds everything

`src/lib.rs` exposes every operation the CLI performs — loading, compiling,
measuring, reporting, exporting. `src/main.rs` only parses arguments, touches
files and decides what to print. The checks are therefore testable without
spawning a subprocess, and the tool is usable as a crate.
