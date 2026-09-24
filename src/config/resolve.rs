//! Four layers into one answer. Most specific wins, and a layer that says
//! nothing about a key leaves the layer below it alone.

use std::path::PathBuf;

use super::{CheckSettings, Config, LooseLineSettings, ResumeSettings, Stamp};
use crate::diagnostic::Diagnostic;
use crate::report::Check;

/// The fourth layer: this invocation.
#[derive(Debug, Clone, Default)]
pub struct Flags {
    pub max_pages: Option<usize>,
    pub dpi: Option<f64>,
    pub output: Option<PathBuf>,
}

/// What goes into the PDF's metadata, and nothing else about the file.
#[derive(Debug, Clone)]
pub struct PdfMeta {
    pub creator: String,
    pub producer: String,
    pub date: Stamp,
    pub keywords: Vec<String>,
}

/// Every setting, decided.
#[derive(Debug, Clone)]
pub struct Resolved {
    pub theme: String,
    pub pdf: PdfMeta,
    pub check: Check,
    pub output: Option<PathBuf>,
    pub dpi: f64,
}

fn identity() -> String {
    format!("cratecv {}", env!("CARGO_PKG_VERSION"))
}

/// Fold the layers. The only thing that can fail here is a `pdf.date` that is
/// neither of the two words nor a timestamp.
pub fn resolve(
    config: &Config,
    resume: Option<&ResumeSettings>,
    flags: &Flags,
) -> Result<Resolved, Vec<Diagnostic>> {
    let resume_pdf = resume.and_then(|settings| settings.pdf.as_ref());
    let resume_check = resume.and_then(|settings| settings.check.as_ref());

    let date = pick(config.pdf.as_ref(), resume_pdf, |pdf| pdf.date.clone())
        .map(|value| super::parse_date(&value))
        .transpose()
        .map_err(|why| vec![why])?
        .unwrap_or(Stamp::Mtime);

    Ok(Resolved {
        theme: resume
            .and_then(|settings| settings.theme.clone())
            .or_else(|| config.theme.clone())
            .unwrap_or_else(|| "default".to_owned()),
        pdf: PdfMeta {
            creator: pick(config.pdf.as_ref(), resume_pdf, |pdf| pdf.creator.clone())
                .unwrap_or_else(identity),
            producer: pick(config.pdf.as_ref(), resume_pdf, |pdf| pdf.producer.clone())
                .unwrap_or_else(identity),
            date,
            keywords: pick(config.pdf.as_ref(), resume_pdf, |pdf| pdf.keywords.clone())
                .unwrap_or_default(),
        },
        check: check(config.check.as_ref(), resume_check, flags),
        output: flags
            .output
            .clone()
            .or_else(|| config.output.as_ref().and_then(|out| out.dir.clone())),
        dpi: flags
            .dpi
            .or_else(|| config.preview.as_ref().and_then(|preview| preview.dpi))
            .unwrap_or(150.0),
    })
}

fn check(config: Option<&CheckSettings>, resume: Option<&CheckSettings>, flags: &Flags) -> Check {
    let mut out = Check::default();
    if let Some(pages) = flags
        .max_pages
        .or_else(|| pick(config, resume, |check| check.max_pages))
    {
        out.max_pages = pages;
    }

    let config_lines = config.and_then(|check| check.loose_lines.as_ref());
    let resume_lines = resume.and_then(|check| check.loose_lines.as_ref());
    let loose = &mut out.loose_lines;
    if let Some(value) = pick(config_lines, resume_lines, |lines| lines.severity) {
        loose.severity = value;
    }
    if let Some(value) = pick(config_lines, resume_lines, |lines| lines.short_line) {
        loose.short_line = value;
    }
    if let Some(value) = pick(config_lines, resume_lines, |lines| lines.short_last_line) {
        loose.short_last_line = value;
    }
    if let Some(value) = pick(config_lines, resume_lines, |lines| lines.loose_line_gap) {
        loose.loose_line_gap = value;
    }
    if let Some(value) = pick(config_lines, resume_lines, |lines: &LooseLineSettings| {
        lines.widow_words
    }) {
        loose.widow_words = value;
    }
    out
}

/// The resume's word on a key, else the config's, else nothing.
fn pick<S, T>(config: Option<&S>, resume: Option<&S>, read: impl Fn(&S) -> Option<T>) -> Option<T> {
    resume.and_then(&read).or_else(|| config.and_then(read))
}
