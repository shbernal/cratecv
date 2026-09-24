# 0001. Fonts are compiled into the binary

Status: accepted

## Context

A resume compiler that lays a page out and then reports how full each line is
only means anything if the measurement is the same everywhere. Line breaks come
from glyph advances, so the faces in use decide every number in the report. A
tool that resolves fonts from the host resolves them differently on every host:
the same YAML produces a different page on a laptop and in CI, and "fills 62% of
its line" stops being a fact about the resume.

`typst-kit` is the obvious way to get fonts into a `World`. Its font value is a
store over `include_bytes!` faces, which is a few lines to own outright;
everything else it offers is package resolution and system font scanning, and
its `scan-fonts` feature links fontconfig.

## Decision

Four static faces of Noto Serif are committed under `assets/fonts/` and pulled
in with `include_bytes!`. The `World` implementation registers exactly those and
nothing else. System fonts are never consulted, and `typst-kit` is not a
dependency: a crate whose main feature set has to be remembered to stay disabled
is a standing risk to this guarantee, and the guarantee is better kept by not
linking the code that could break it.

`scripts/build-fonts.sh` produces the faces from the upstream variable fonts,
pinned by version and checksum, with `SOURCE_DATE_EPOCH` fixed so rebuilds are
byte-identical. All four keep `Noto Serif` as their family name and differ only
by weight class and italic bit, which is how Typst selects between them.

## Consequences

Output is identical on every machine, and the binary has no runtime font
dependency at all.

The faces are subset to the Latin range, so there is no fallback for anything
outside it. A resume in Greek, Cyrillic or any non-Latin script renders as
missing glyphs rather than falling back to a system face. That is a real cost,
and the honest statement of scope is that this tool sets Latin text today.

Each face is around fifty kilobytes, so the four add roughly two hundred
kilobytes to the binary.
