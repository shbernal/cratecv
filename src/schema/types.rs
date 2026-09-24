//! The resume, as declared. Strict: an unknown key is an error, not a hint.

use serde::{Deserialize, Serialize};

/// The built-in themes a resume may name.
pub const THEMES: &[&str] = &["default"];

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resume {
    pub name: String,
    /// One line under the name, tailored to the offer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<Headline>,
    pub contact: Contact,
    pub sections: Vec<Section>,
    /// Settings for this tool. Consumed during validation, never rendered.
    #[serde(default, skip_serializing)]
    pub cratecv: Option<Settings>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Headline {
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub linkedin: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Section {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Entry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<Skill>>,
    /// How a skills section lays out: one row per skill, or badges on one line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    pub dates: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<Bullet>>,
}

/// `label` renders in bold before the text, as a lead-in.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bullet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Skill {
    pub label: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

/// Where a line comes from, one path or several. The content repository checks
/// it; this tool accepts it without reading it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Source {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Rows,
    Badges,
}

/// The resume's own `cratecv:` block.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<String>,
}

impl Section {
    /// A skills section lays out as rows unless it says otherwise.
    pub fn layout(&self) -> Layout {
        self.layout.unwrap_or(Layout::Rows)
    }
}
