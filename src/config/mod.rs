//! The settings tree, stated at four levels of specificity and resolved into
//! one answer.
//!
//! Both files are YAML and both go through the schema loader, so a typo in the
//! config gets the same pointed diagnostic, at the same line and column, as a
//! typo in a resume, and the binary carries one parser rather than two.

mod date;
mod resolve;

use std::path::PathBuf;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::diagnostic::Diagnostic;
use crate::report::Severity;

pub use date::{Stamp, parse_date};
pub use resolve::{Flags, PdfMeta, Resolved, resolve};

/// `~/.config/cratecv/config.yaml`: every resume on this machine.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub theme: Option<String>,
    pub pdf: Option<Pdf>,
    pub check: Option<CheckSettings>,
    pub output: Option<Output>,
    pub preview: Option<Preview>,
}

/// The `cratecv:` block in a resume: that one document, everywhere.
///
/// It takes any key describing the document. `output` and `preview` describe
/// this machine and this invocation, so they are not here. The head is repeated
/// rather than shared with `#[serde(flatten)]`, which silently defeats
/// `deny_unknown_fields`.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResumeSettings {
    pub theme: Option<String>,
    pub pdf: Option<Pdf>,
    pub check: Option<CheckSettings>,
}

/// Title and author are absent by design: they come from the resume's `name`,
/// and nobody should have to state their own name twice.
#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Pdf {
    pub creator: Option<String>,
    pub producer: Option<String>,
    /// `mtime`, `none`, or a fixed timestamp such as `"2024-03-11T09:12:00Z"`.
    /// Quoted, because an unquoted timestamp is a YAML date.
    pub date: Option<String>,
    pub keywords: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CheckSettings {
    pub max_pages: Option<usize>,
    pub loose_lines: Option<LooseLineSettings>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LooseLineSettings {
    /// `error`, `warn` or `silent`. Not `off`: that is a boolean in YAML.
    pub severity: Option<Severity>,
    pub short_line: Option<f64>,
    pub short_last_line: Option<f64>,
    pub loose_line_gap: Option<f64>,
    pub widow_words: Option<usize>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Output {
    pub dir: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Preview {
    pub dpi: Option<f64>,
}

/// Where the config lives, honouring `$XDG_CONFIG_HOME`. A directory rather
/// than a bare file, because a user template directory belongs beside it.
pub fn directory() -> PathBuf {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".config")))
        .unwrap_or_else(|| PathBuf::from(".config"));
    base.join("cratecv")
}

pub fn path() -> PathBuf {
    directory().join("config.yaml")
}

/// Read the config, or the compiled-in defaults when there is no file.
pub fn load() -> Result<Config, Vec<Diagnostic>> {
    let path = path();
    let Ok(yaml) = std::fs::read_to_string(&path) else {
        return Ok(Config::default());
    };
    crate::schema::load_settings(&yaml)
}

/// What `cratecv init` writes. Commented, and every value is the default, so
/// uncommenting a line changes nothing until its number is changed.
pub const STARTER: &str = include_str!("starter.yaml");
