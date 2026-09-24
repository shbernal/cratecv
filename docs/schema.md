# The resume format

A resume is a single YAML file. Keys are camelCase, unknown keys are an error,
and every string must carry text: a blank value is a mistake, not an omission,
so drop the key instead.

```yaml
name: Mira Halvorsen
headline:
  text: Platform engineer who makes build systems fast
  source: headlines/platform.yaml
contact:
  email: mira@halvorsen.example
sections:
  - title: Experience
    entries:
      - dates: 2022 — present
        organization: Tessellate
        role: Staff Engineer
        details:
          - label: Remote execution
            text: Moved a 40-minute monorepo build onto a shared cache.
```

`examples/resume.yaml` exercises every field below.

## Root

| Key | Required | Notes |
| --- | --- | --- |
| `name` | yes | |
| `headline` | no | `text`, plus an optional `source` |
| `contact` | yes | the block is required, its four fields are not |
| `sections` | yes | at least one |
| `cratecv` | no | settings for this tool, never rendered |

`contact` takes `phone`, `email`, `github` and `linkedin`, each optional.

## Sections

A section has a `title` and exactly one of `entries` or `skills`.

An entry's only required key is `dates`. `organization`,
`organizationSubtitle`, `location`, `role` and `details` are all optional, so an
education entry with no role and a project with no location are both legal.

Each bullet in `details` has `text`, an optional `label` that renders in bold as
a lead-in, and an optional `source`.

A skill has `label` and `value`. A skills section may set `layout` to `rows`
(the default, one skill per row) or `badges` (all of them on one line).
`layout` on an entries section is an error.

## `source`

One path, or a non-empty list of them:

```yaml
source: work/tessellate/resolver.yaml
source: [work/tessellate/remote-exec.yaml, metrics/ci-2024.yaml]
```

It records where a line came from. This tool accepts it and never reads it: the
content lives upstream and is checked there.

## `cratecv`

```yaml
cratecv:
  theme: default
```

`theme` names a built-in theme. `default` is the only one.

## Diagnostics

A resume that does not load produces diagnostics rather than one error. Each
carries a message, a 1-based line and column, and a path:

```
7:9 at sections[0].entries[0].details[2].text: this is empty. Give it a value or drop the key
```

That path is the same spelling the layout report uses to attribute a line back
to the bullet it came from, so a diagnostic and a warning about the same bullet
name it identically.
