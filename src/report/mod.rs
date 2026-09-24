//! The JSON contract, and the settings whose numbers it reports against.
//!
//! This is the tool's product. Everything else is an implementation detail
//! behind it, and it is going to be read by things that cannot ask a follow-up
//! question, so it stays flat and it says what it measured against.

mod rules;

use serde::{Deserialize, Serialize};

use crate::diagnostic::Diagnostic;
use crate::layout::Measured;
use crate::schema::Locator;

pub use rules::LooseKind;

/// What a check does when it finds something.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Severity {
    /// Reported, and the run fails.
    Error,
    /// Reported, and the run still succeeds.
    #[default]
    Warn,
    /// Not run at all.
    Silent,
}

/// The numbers every rule reads. Never a constant at the point of use: the
/// settings are resolved once and echoed into the report, so a number in the
/// output can always be traced to the setting that produced it.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    /// Page count is the guarantee, so it is a value with no severity. A second
    /// way to express the same intent would quietly disable it.
    pub max_pages: usize,
    pub loose_lines: LooseLines,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LooseLines {
    /// The only supported way to silence this check. Widening a threshold until
    /// nothing trips would read back as a real measurement.
    pub severity: Severity,
    /// A block that never wrapped and fills less than this has room for more.
    pub short_line: f64,
    /// A last line shorter than this share of its width stopped early.
    pub short_last_line: f64,
    /// A line before the last one leaving more than this share empty was cut
    /// short by a word that did not fit.
    pub loose_line_gap: f64,
    /// A last line with at most this many words is a widow.
    pub widow_words: usize,
}

impl Default for Check {
    fn default() -> Self {
        Self {
            max_pages: 1,
            loose_lines: LooseLines::default(),
        }
    }
}

impl Default for LooseLines {
    /// Tuned against real resumes, and the only numbers here with provenance.
    fn default() -> Self {
        Self {
            severity: Severity::Warn,
            short_line: 0.75,
            short_last_line: 0.35,
            loose_line_gap: 0.20,
            widow_words: 1,
        }
    }
}

/// One line that left space behind.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LooseLine {
    pub kind: LooseKind,
    /// Canonical path into the YAML.
    pub path: String,
    /// 1-based line in the resume file.
    pub line: usize,
    /// The block's text, shortened.
    pub text: String,
    /// How much of its width the line covers, 0.0 to 1.0.
    pub fill: f64,
    /// Roughly how many more words of the same length would fit. Only a line
    /// that never wrapped has a meaningful answer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub room_words: Option<usize>,
    /// The word that did not fit, for a line cut short.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bumped: Option<String>,
}

/// What a run of this tool has to say. `ok` tracks the exit code, not "nothing
/// was found": a resume with four loose lines at the default `warn` severity is
/// `ok: true` with a non-empty `looseLines`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub ok: bool,
    pub pages: usize,
    pub max_pages: usize,
    /// How far past `maxPages` the content runs. Set only when it does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow_mm: Option<f64>,
    pub schema_errors: Vec<Diagnostic>,
    pub loose_lines: Vec<LooseLine>,
    /// Echoed, so `looseLines: []` at `silent` cannot read as a clean resume.
    pub loose_line_severity: Severity,
    /// The space the findings add up to, counted in whole lines.
    pub wasted_lines: f64,
    /// Every setting the numbers above were measured against.
    pub settings: Check,
}

impl Report {
    /// A resume that never got as far as being laid out.
    pub fn rejected(errors: Vec<Diagnostic>, check: &Check) -> Self {
        Self {
            ok: false,
            pages: 0,
            max_pages: check.max_pages,
            overflow_mm: None,
            schema_errors: errors,
            loose_lines: Vec::new(),
            loose_line_severity: check.loose_lines.severity,
            wasted_lines: 0.0,
            settings: check.clone(),
        }
    }

    /// Whether the page count is within the limit.
    pub fn fits(&self) -> bool {
        self.pages <= self.max_pages
    }
}

/// Judge a laid-out page against the settings.
pub fn report(measured: &Measured, locator: &Locator, check: &Check) -> Report {
    let findings = rules::findings(measured, locator, &check.loose_lines);
    let wasted_lines = findings
        .iter()
        .map(|found| (1.0 - found.fill).max(0.0))
        .sum();
    let fits = measured.pages <= check.max_pages;
    let fatal_lines = check.loose_lines.severity == Severity::Error && !findings.is_empty();

    Report {
        ok: fits && !fatal_lines,
        pages: measured.pages,
        max_pages: check.max_pages,
        overflow_mm: measured.overflow_mm(check.max_pages),
        schema_errors: Vec::new(),
        loose_lines: findings,
        loose_line_severity: check.loose_lines.severity,
        wasted_lines,
        settings: check.clone(),
    }
}
