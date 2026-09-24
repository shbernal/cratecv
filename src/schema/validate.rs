//! The rules the type system cannot state.

use marked_yaml::Node;

use super::path::Path;
use super::types::{Contact, Entry, Headline, Resume, Section, Skill, Source, THEMES};
use crate::config::ResumeSettings;
use crate::diagnostic::Diagnostic;

/// Collects every rule violation in one pass, so a caller fixing a resume sees
/// all of them rather than the first.
pub(super) fn validate(resume: &Resume, root: &Node) -> Vec<Diagnostic> {
    let mut found = Findings {
        root,
        out: Vec::new(),
    };
    let at = Path::root();

    found.text(&at.key("name"), &resume.name);
    if let Some(headline) = &resume.headline {
        headline_rules(&mut found, &at.key("headline"), headline);
    }
    contact_rules(&mut found, &at.key("contact"), &resume.contact);

    if resume.sections.is_empty() {
        found.push(&at.key("sections"), "a resume needs at least one section");
    }
    for (index, section) in resume.sections.iter().enumerate() {
        section_rules(&mut found, &at.key("sections").index(index), section);
    }

    if let Some(settings) = &resume.cratecv {
        settings_rules(&mut found, &at.key("cratecv"), settings);
    }

    found.out
}

fn headline_rules(found: &mut Findings, at: &Path, headline: &Headline) {
    found.text(&at.key("text"), &headline.text);
    found.source(&at.key("source"), headline.source.as_ref());
}

fn contact_rules(found: &mut Findings, at: &Path, contact: &Contact) {
    for (key, value) in [
        ("phone", &contact.phone),
        ("email", &contact.email),
        ("github", &contact.github),
        ("linkedin", &contact.linkedin),
    ] {
        if let Some(value) = value {
            found.text(&at.key(key), value);
        }
    }
}

fn section_rules(found: &mut Findings, at: &Path, section: &Section) {
    found.text(&at.key("title"), &section.title);

    match (&section.entries, &section.skills) {
        (Some(_), Some(_)) => found.push(at, "a section takes entries or skills, not both"),
        (None, None) => found.push(at, "a section needs either entries or skills"),
        _ => {}
    }

    if section.layout.is_some() && section.skills.is_none() {
        found.push(&at.key("layout"), "only a skills section takes a layout");
    }

    for (index, entry) in section.entries.iter().flatten().enumerate() {
        entry_rules(found, &at.key("entries").index(index), entry);
    }
    for (index, skill) in section.skills.iter().flatten().enumerate() {
        skill_rules(found, &at.key("skills").index(index), skill);
    }
}

fn entry_rules(found: &mut Findings, at: &Path, entry: &Entry) {
    found.text(&at.key("dates"), &entry.dates);
    for (key, value) in [
        ("organization", &entry.organization),
        ("organizationSubtitle", &entry.organization_subtitle),
        ("location", &entry.location),
        ("role", &entry.role),
    ] {
        if let Some(value) = value {
            found.text(&at.key(key), value);
        }
    }
    for (index, bullet) in entry.details.iter().flatten().enumerate() {
        let at = at.key("details").index(index);
        if let Some(label) = &bullet.label {
            found.text(&at.key("label"), label);
        }
        found.text(&at.key("text"), &bullet.text);
        found.source(&at.key("source"), bullet.source.as_ref());
    }
}

fn skill_rules(found: &mut Findings, at: &Path, skill: &Skill) {
    found.text(&at.key("label"), &skill.label);
    found.text(&at.key("value"), &skill.value);
    found.source(&at.key("source"), skill.source.as_ref());
}

fn settings_rules(found: &mut Findings, at: &Path, settings: &ResumeSettings) {
    if let Some(theme) = &settings.theme {
        let at = at.key("theme");
        found.text(&at, theme);
        if !theme.is_empty() && !THEMES.contains(&theme.as_str()) {
            found.push(
                &at,
                format!("unknown theme `{theme}`. Available: {}", THEMES.join(", ")),
            );
        }
    }
}

struct Findings<'a> {
    root: &'a Node,
    out: Vec<Diagnostic>,
}

impl Findings<'_> {
    fn push(&mut self, at: &Path, message: impl Into<String>) {
        let (line, column) = at.locate(self.root);
        self.out.push(Diagnostic {
            path: at.to_string(),
            line,
            column,
            message: message.into(),
        });
    }

    /// Every string in a resume carries meaning, so a blank one is a mistake
    /// rather than an omission: the key itself should go.
    fn text(&mut self, at: &Path, value: &str) {
        if value.trim().is_empty() {
            self.push(at, "this is empty. Give it a value or drop the key");
        }
    }

    fn source(&mut self, at: &Path, source: Option<&Source>) {
        match source {
            None => {}
            Some(Source::One(one)) => self.text(at, one),
            Some(Source::Many(many)) => {
                if many.is_empty() {
                    self.push(at, "a source list needs at least one entry");
                }
                for (index, one) in many.iter().enumerate() {
                    self.text(&at.index(index), one);
                }
            }
        }
    }
}
