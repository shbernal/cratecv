# 0003. Typography is theme-internal, not user-configurable

Status: accepted

## Context

Page size, margins, base font size, density and accent colour are the settings a
resume tool is normally expected to expose. The template does read them, from
`sys.inputs`, because a second theme will need that seam.

But every number this tool reports is measured against the laid-out page. A fill
ratio is a line's width over the measure it was broken against, and that measure
is a function of page size, margins and font size. The one-page guarantee is the
same: content fits or it does not, given a page.

## Decision

No YAML key reaches those `sys.inputs` values, and the settings tree has no
section for them. They are a theme's own business.

## Consequences

A user who wants wider margins, or Letter instead of A4, has no knob. That is a
real limitation and the most likely thing someone asks for first.

What it buys is that "fills 42% of its line" means the same thing on two
machines and in CI, and that two runs of the same resume are comparable. The
moment margins vary per machine, the report's numbers become incomparable with
nothing in the output explaining why.

If this is reopened, it has to come with a plan for keeping the report
self-describing — most likely echoing the resolved typography beside the
resolved check settings, the way the check settings are echoed today.
