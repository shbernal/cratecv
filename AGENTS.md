# AI project guidelines

`cratecv`: compiles a YAML resume into a one-page PDF with Typst, and reports
layout problems as JSON that an agent can act on.

- Key commands
  - `cargo run -- build examples/resume.yaml -o output/resume.pdf`
  - `cargo run -- check examples/resume.yaml --json`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --check`

- Architecture
  - `src/lib.rs` is the library, `src/main.rs` the CLI. Everything the CLI does
    is reachable from the library, so checks are testable without a subprocess.
  - Typst is embedded as a crate, not shelled out to. `World` serves the
    template, the fonts, and the resume YAML as a virtual file tree.
  - Guardrails read the laid-out `PagedDocument` frame tree, not the PDF. Page
    count, overflow, and per-line fill all come from `Frame::items()`.
  - Fonts are compiled into the binary. The tool never reads system fonts.

- Iron Laws
  - Tokens are expensive, state of the art models need minimal guidance, don't repeat yourself, don't babysit, don't be over-specific.
  - AI-native project. All code is AI-generated.
  - Minimal attention when model implements without errors, we document in more detail when model struggles.
  - Do not expect the user to have read each line, don't lose him on the internals, give visibility on a higher-architectural level.
  - No journaling: code comments / documentation describe current state, they don't carry a log of their own edit history.
