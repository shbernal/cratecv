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
  margin-x: num("marginX", 14) * 1mm,
  margin-y: num("marginY", 12) * 1mm,
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

#set page(paper: theme.paper, margin: (x: theme.margin-x, y: theme.margin-y))
#set text(font: "Noto Serif", size: theme.size, fill: ink, lang: "en")
#set par(leading: 0.35em, justify: false, spacing: 0.35em)

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

#let icon(path) = box(
  baseline: 1.5pt,
  height: small * 0.85,
  image(path, fit: "contain"),
)

// ── header ────────────────────────────────────────────────────────────────

#let contact-items = {
  let c = resume.contact
  let out = ()
  if "phone" in c { out.push(text(c.phone.replace(" ", sym.space.nobreak))) }
  if "email" in c { out.push(link("mailto:" + c.email, c.email)) }
  if "github" in c {
    out.push(link("https://" + c.github, box(icon("/icons/github.svg")) + h(3pt) + c.github))
  }
  if "linkedin" in c {
    out.push(link("https://" + c.linkedin, box(icon("/icons/linkedin.svg")) + h(3pt) + c.linkedin))
  }
  out
}

#align(center, {
  block(below: 4pt, text(size: theme.size * 1.9, weight: "semibold", tracking: 0.02em, resume.name))
  if "headline" in resume {
    block(below: 4pt, text(size: theme.size, style: "italic", fill: quiet, resume.headline.text))
  }
  set text(size: small, fill: muted)
  block(contact-items.join(box(inset: (x: 7pt), text(fill: rgb("#a3a3a3"), sym.bullet)))) 
})

// ── sections ──────────────────────────────────────────────────────────────

#let section-title(title) = block(
  width: 100%,
  above: 7pt * theme.density,
  below: 3pt,
  stroke: (bottom: 0.5pt + ink),
  inset: (bottom: 1.5pt),
  text(size: theme.size * 1.048, weight: 800, tracking: 0.06em, upper(title)),
)

#let bullets(path, details) = {
  set text(size: small)
  grid(
    columns: (10pt, 1fr),
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

#let entry(path, item) = block(breakable: false, inset: (y: 2pt * theme.density), grid(
  columns: (68pt, 1fr),
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
      block(above: 2pt, below: 0pt, text(size: tiny, style: "italic", fill: quiet, item.role))
    }
    if "details" in item {
      block(above: 3.5pt * theme.density, below: 0pt, bullets(path, item.details))
    }
  },
))

#let skill-rows(path, skills) = {
  set text(size: small)
  grid(
    columns: (116pt, 1fr),
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
          inset: (x: 6pt, y: 2.5pt),
          text(weight: "semibold", skill.label),
        ),
        block(
          width: 100%,
          stroke: 0.5pt + hair,
          radius: 2pt,
          inset: (x: 6pt, y: 2.5pt),
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
  set text(size: small)
  block(width: 100%, skills
    .map(skill => box(radius: 2pt, clip: true, stroke: 0.5pt + hair, {
      box(fill: wash, inset: (x: 6pt, y: 2.5pt), text(weight: "semibold", skill.label))
      box(inset: (x: 6pt, y: 2.5pt), skill.value)
    }))
    .join([ ]))
}

#for (index, section) in resume.sections.enumerate() {
  let path = "sections[" + str(index) + "]"
  section-title(section.title)
  if "entries" in section {
    for (n, item) in section.entries.enumerate() {
      entry(path + ".entries[" + str(n) + "]", item)
    }
  } else if section.at("layout", default: "rows") == "badges" {
    block(inset: (y: 2pt * theme.density), skill-badges(section.skills))
  } else {
    block(inset: (y: 2pt * theme.density), skill-rows(path, section.skills))
  }
}
