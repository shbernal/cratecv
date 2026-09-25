//! The resume, as declared. Strict: an unknown key is an error, not a hint.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::config::ResumeSettings;

/// The built-in themes a resume may name.
pub const THEMES: &[&str] = &["default"];

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Resume {
    #[schemars(length(min = 1))]
    pub name: String,
    /// One line under the name, tailored to the offer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headline: Option<Headline>,
    pub contact: Contact,
    #[schemars(length(min = 1))]
    pub sections: Vec<Section>,
    /// Settings for this tool. Consumed during validation, never rendered.
    #[serde(default, skip_serializing)]
    pub cratecv: Option<ResumeSettings>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Headline {
    #[schemars(length(min = 1))]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Contact {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub email: Option<String>,
    /// The GitHub username alone, which links to `github.com/<username>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1), regex(pattern = r"^[^/\s]+$"))]
    pub github: Option<String>,
    /// The LinkedIn profile handle alone, which links to
    /// `linkedin.com/in/<handle>`.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1), regex(pattern = r"^[^/\s]+$"))]
    pub linkedin: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Section {
    #[schemars(length(min = 1))]
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<Entry>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skills: Option<Vec<Skill>>,
    /// How a skills section lays out: one row per skill, or badges on one line.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Entry {
    #[schemars(length(min = 1))]
    pub dates: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub organization_subtitle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Vec<Bullet>>,
}

/// `label` renders in bold before the text, as a lead-in.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Bullet {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub label: Option<String>,
    #[schemars(length(min = 1))]
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Skill {
    #[schemars(length(min = 1))]
    pub label: String,
    #[schemars(length(min = 1))]
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
}

/// Where a line comes from, one path or several. The content repository checks
/// it; this tool accepts it without reading it.
#[derive(Debug, Clone, Deserialize, Serialize, JsonSchema)]
#[serde(untagged)]
pub enum Source {
    One(String),
    Many(#[schemars(length(min = 1))] Vec<String>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Rows,
    Badges,
}

impl Section {
    /// A skills section lays out as rows unless it says otherwise.
    pub fn layout(&self) -> Layout {
        self.layout.unwrap_or(Layout::Rows)
    }
}
