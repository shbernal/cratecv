// Placeholder. The theme lands with the layout work.
#let resume = json("/resume.json")
#set text(font: "Noto Serif", size: 9.5pt)
#set page(paper: "a4", margin: 1.5cm)
= #resume.name
#for section in resume.sections [
  == #section.title
]
