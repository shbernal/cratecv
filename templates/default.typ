// The built-in theme. It reads the resume out of the virtual file tree and
// marks the blocks whose fill the guardrails measure.
//
// Typography is theme-internal. The keys below come from sys.inputs, which no
// YAML key reaches: report numbers stay comparable between machines only while
// the measure a line was broken against is a property of the theme.

#let resume = json("/resume.json")

#let opt(key, fallback) = sys.inputs.at(key, default: fallback)
#let num(key, fallback) = float(opt(key, str(fallback)))

#let theme = (
  paper: opt("paper", "a4"),
  margin-x: num("marginX", 18) * 1pt,
  margin-y: num("marginY", 24) * 1pt,
  size: num("size", 10.5) * 1pt,
  density: num("density", 1),
  accent: rgb(opt("accent", "#171717")),
)

#let small = theme.size * 0.905  // 9.5pt at the default size
#let tiny = theme.size * 0.857 // 9pt
#let ink = theme.accent
#let muted = rgb("#525252")
#let dim = rgb("#404040")
#let quiet = rgb("#262626")
#let wash = rgb("#f5f5f5")
#let hair = rgb("#e5e5e5")
#let gap = theme.density * 1pt
// Bullets are set a hair tight, about 1% narrower than the font's own
// advances. That is the measure resumes written for a browser-printed page were
// filled against, where each glyph's advance was rounded to a whole pixel.
#let fit = num("fit", -0.008) * 1em

// A line is `line` ems tall, with the glyphs centred in it: Noto Serif's
// ascender and descender, less half of whatever the line leaves over.
#let lines(line) = {
  let over = (line - 1.362) / 2
  (top-edge: (1.069 + over) * 1em, bottom-edge: -(0.293 + over) * 1em)
}

#set page(paper: theme.paper, margin: (x: theme.margin-x, y: theme.margin-y))
#set text(font: "Noto Serif", size: theme.size, fill: ink, lang: "en", ..lines(1.35))
#set par(leading: 0pt, justify: false, spacing: 0pt)
#set block(above: 0pt, below: 0pt)

// Marks a block whose text is written to fill the width it is given, and
// records the measure the layout engine actually broke it against. Everything
// the guardrails report comes from one of these; anything unmarked is simply
// absent from the walk, so there is no exemption list to maintain.
//
// The closing marker is what bounds a block. Without it the walk would have to
// infer where a block ends from how Typst happened to nest its frames, and the
// bullet glyph after the last line would read as a line of the bullet.
#let measured(path, body) = block(width: 100%, {
  place(context layout(size => metadata((path: path, available: size.width / 1pt))))
  body
  place(metadata((end: path)))
})

// `alt` is not decoration: an extractor walking the structure tree reads it,
// and PDF/UA-1 export refuses to write without it.
#let icon(path, alt) = box(
  baseline: 1.5pt,
  height: small * 0.85,
  image(path, alt: alt, fit: "contain"),
)

// ── header ────────────────────────────────────────────────────────────────

#let contact-items = {
  let c = resume.contact
  let out = ()
  if "phone" in c { out.push(text(c.phone.replace(" ", sym.space.nobreak))) }
  if "email" in c { out.push(link("mailto:" + c.email, c.email)) }
  if "github" in c {
    out.push(link(
      "https://github.com/" + c.github,
      box(icon("/icons/github.svg", "GitHub")) + h(3pt) + c.github,
    ))
  }
  if "linkedin" in c {
    out.push(link(
      "https://www.linkedin.com/in/" + c.linkedin,
      box(icon("/icons/linkedin.svg", "LinkedIn")) + h(3pt) + c.linkedin,
    ))
  }
  out
}

#align(center, {
  block(below: 3pt, text(size: theme.size * 1.905, weight: "semibold", tracking: 0.02em, resume.name))
  if "headline" in resume {
    block(below: 3pt, text(size: theme.size, style: "italic", fill: quiet, resume.headline.text))
  }
  set text(size: small, fill: muted)
  block(below: 6pt, contact-items.join(box(
    inset: (x: 12pt),
    baseline: -0.3em,
    circle(radius: 2.25pt, fill: rgb("#a3a3a3"), stroke: none),
  )))
})

// ── sections ──────────────────────────────────────────────────────────────

#let section-title(title) = block(
  width: 100%,
  below: 4.5pt * theme.density,
  sticky: true,
  stroke: (bottom: 0.75pt + ink),
  text(size: theme.size * 1.048, weight: 800, tracking: 0.06em, upper(title)),
)

#let bullets(path, details) = {
  set text(size: small, tracking: fit)
  grid(
    columns: (12pt, 1fr),
    row-gutter: 3pt * theme.density,
    ..details
      .enumerate()
      .map(((index, item)) => (
        text(fill: dim, sym.bullet),
        measured(path + ".details[" + str(index) + "]", {
          if "label" in item { text(weight: "semibold", item.label + ": ") }
          item.text
        }),
      ))
      .flatten(),
  )
}

#let entry(path, item) = block(breakable: false, inset: (y: 3pt * theme.density), grid(
  columns: (67.5pt, 1fr),
  column-gutter: 18pt,
  text(size: small, fill: muted, item.dates),
  {
    if "organization" in item or "location" in item {
      grid(
        columns: (1fr, auto),
        column-gutter: 12pt,
        align: (left + bottom, right + bottom),
        {
          if "organization" in item {
            text(weight: 800, upper(item.organization) + if "organizationSubtitle" in item { "," })
            if "organizationSubtitle" in item {
              h(4pt)
              text(size: small, style: "italic", fill: dim, item.organizationSubtitle)
            }
          }
        },
        if "location" in item { text(size: small, fill: muted, item.location) },
      )
    }
    if "role" in item {
      block(text(size: tiny, style: "italic", fill: quiet, item.role))
    }
    if "details" in item {
      block(above: 3pt * theme.density, bullets(path, item.details))
    }
  },
))

#let skill-rows(path, skills) = {
  set text(size: small, ..lines(1))
  grid(
    columns: (116.25pt, 1fr),
    column-gutter: 6pt,
    row-gutter: 3pt * theme.density,
    align: horizon,
    ..skills
      .enumerate()
      .map(((index, skill)) => (
        block(
          width: 100%,
          fill: wash,
          radius: 2pt,
          inset: (x: 6pt, y: 1.5pt),
          text(weight: "semibold", skill.label),
        ),
        block(
          width: 100%,
          stroke: 0.75pt + hair,
          radius: 2pt,
          // The border sits inside the box, as a browser draws one.
          inset: (x: 6pt, y: 2.25pt),
          measured(path + ".skills[" + str(index) + "]", skill.value),
        ),
      ))
      .flatten(),
  )
}

// Badges carry no marker. A badge is a box sized to its own text, so it has no
// measure to fall short of, and a fill ratio for one would be a number with
// nothing behind it.
#let skill-badges(skills) = {
  set text(size: small, ..lines(1))
  set par(leading: 3pt)
  block(width: 100%, skills
    .map(skill => box(radius: 2pt, clip: true, stroke: 0.75pt + hair, inset: 0.75pt, {
      box(fill: wash, inset: (x: 6pt, y: 1.5pt), text(weight: "semibold", skill.label))
      box(inset: (x: 6pt, y: 1.5pt), skill.value)
    }))
    .join(h(6pt)))
}

#for (index, section) in resume.sections.enumerate() {
  let path = "sections[" + str(index) + "]"
  if index > 0 { v(3pt * theme.density) }
  section-title(section.title)
  let blocks = if "entries" in section {
    section.entries.enumerate().map(((n, item)) => entry(path + ".entries[" + str(n) + "]", item))
  } else if section.at("layout", default: "rows") == "badges" {
    (block(inset: (y: 3pt * theme.density), skill-badges(section.skills)),)
  } else {
    (block(inset: (y: 3pt * theme.density), skill-rows(path, section.skills)),)
  }
  blocks.join(v(4.5pt * theme.density))
}

// Where the last page's content area ends, so the report can say how much of
// it the resume leaves empty.
#place(bottom, metadata((floor: true)))
