# The command line

```bash
cratecv build   cv.yaml [-o PATH] [--allow-overflow] [--max-pages N]
cratecv check   cv.yaml [--json] [--max-pages N]
cratecv preview cv.yaml [-o PATH] [--dpi N]
cratecv watch   cv.yaml [-o PATH] [--max-pages N]
```

`-o` takes a file or a directory. A directory receives the output under a name
derived from the resume's `name`, so `-o out/` writes `out/mira-halvorsen.pdf`.

## Exit codes

Callers branch on these, so they are part of the contract. They track severity,
not which check spoke.

| Code | Meaning |
| --- | --- |
| `0` | compiled, within the page limit, nothing at `error` severity |
| `1` | a check failed: over the page limit, or loose lines set to `error` |
| `2` | the resume is invalid, or the arguments are |
| `3` | a bug in this tool |

Half-empty lines are advice at their default severity, so they report and exit
`0`. See [ADR 0005](technical/adr/0005-severity-decides-exit-codes.md).

`build` writes nothing on exit `1` unless `--allow-overflow`, so the last good
PDF at that path survives a bad run. The flag decides whether a file is written;
it does not change the verdict.

## Output

Human output goes to stderr. `--json` puts the report on stdout and nothing
else, so `cratecv check cv.yaml --json | jq` never has to strip a banner.

```bash
cratecv check cv.yaml --json | jq '.looseLines[] | {path, fill}'
```

## preview and watch

`preview` renders the page as PNG, or as SVG when the output path ends in
`.svg`. It is the real output rather than an approximation of it, which is what
`watch` plus an image viewer makes into a development loop — there is no dev
server here and no need for one.

`watch` recompiles on every save, debounced, and writes even when the page count
is exceeded so the preview keeps up with the edit.
