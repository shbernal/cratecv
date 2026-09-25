---
name: cratecv
description: Compile a cratecv YAML resume to a one-page PDF and fix what its layout report finds. Use when running cratecv, editing a resume in its YAML format, or acting on a `cratecv check` report. Covers the format, the report and layout fixes, not what a resume should say.
---

# cratecv

`cratecv` compiles a YAML resume to a one-page PDF and reports, as JSON, which
lines left space behind and by how much. You edit the YAML, it measures the
page, you edit again.

## The loop

1. Write or edit the YAML.
2. `cratecv check cv.yaml --json`
3. Fix what the report names, by path.
4. Repeat until `looseLines` is empty or what is left is a deliberate choice,
   then `cratecv build cv.yaml -o cv.pdf`.

Run `cratecv schema` for the JSON Schema of the format. It is generated from
the parser's own types, so it cannot describe something the tool would reject.
Do not guess field names from this file.

## Reading the report

`ok` tracks the exit code, **not** whether anything was found. A resume with
four warnings is `ok: true` with a non-empty `looseLines`. Ask `ok` whether the
run succeeded; ask `looseLines` whether there is anything to fix.

Every finding carries `path` — `sections[1].entries[0].details[2]` — which is
the exact bullet to edit, and `line`, which is where it sits in the file.

`looseLineSeverity: "silent"` with an empty `looseLines` means the check was
switched off. It does not mean the resume is clean.

## Fixing what it finds

The only fix for a half-empty line is **rewriting the sentence**. Never pad a
bullet with filler to reach the margin, and never shrink the type: the
thresholds are measured against a fixed layout, so smaller type just moves the
problem.

| Kind | What it means | What to do |
| --- | --- | --- |
| `widow` | The last line holds one word alone | Cut one or two words earlier in the bullet so it pulls up |
| `shortLastLine` | The last line barely starts | Add a clause worth reading, or cut enough to lose the line |
| `looseLine` | A line stopped early because the next word did not fit | `bumped` names that word — shorten it or move it |
| `shortLine` | A bullet that never wrapped and has room | `roomWords` says roughly how many more words fit — say something concrete |

`wastedLines` is the total space the findings add up to, in whole lines. It
answers a question you cannot answer from the findings one at a time: whether
there is enough slack to fit another bullet, or whether the page is genuinely
full. `remainingMm` is the other half: the empty space left at the bottom of the
page. A bullet line is about 4.5mm.

## When it runs over a page

`overflowMm` says how far past the limit the content runs, so you know whether
you are cutting a sentence or a whole entry.

Cut the weakest bullet. Do not reach for a smaller font or narrower margins:
they are not configurable, on purpose — every number in the report is measured
against a fixed page, and a report whose numbers move with the settings cannot
be compared between runs.

If an academic CV genuinely needs two pages, that is a setting:

```yaml
cratecv:
  check:
    maxPages: 2
```

## The format

- `dates` is the only required field on an entry. An education entry with no
  role and a project with no location are both legal and both look deliberate.
- `label` is a bold lead-in before a bullet's sentence.
- Skills in `rows` layout are measured, so a nearly empty row is reported. Use
  `badges` for short lists like languages, which are not measured because a
  badge is sized to its own text.
- Every string must carry text. Drop a key rather than leaving it empty.

## What not to do

- Do not edit the PDF. It is a build artifact.
- Do not tune thresholds to make warnings disappear. `severity: silent` is the
  honest way to switch the check off, and it shows up in the report.
- Do not restate this tool's JSON contract from memory — read `cratecv schema`
  and the report itself.
