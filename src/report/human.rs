//! The report, said out loud.
//!
//! Every finding names what it is, where it is, what it says and what to do
//! about it. The wording matters: the only fix for a half-empty line is
//! rewording, so a message that does not suggest one has wasted the reader's
//! time.

use std::fmt::Write;

use super::{LooseKind, LooseLine, Report};

/// What to print for a run, without the schema errors, which go to stderr.
pub fn summary(report: &Report, file: &str) -> String {
    let mut out = String::new();

    if let Some(over) = report.overflow_mm {
        let _ = writeln!(
            out,
            "{}, and the limit is {}. Content runs {:.0}mm past it.",
            plural(report.pages, "page", "pages"),
            report.max_pages,
            over,
        );
    } else {
        let left = report
            .remaining_mm
            .map(|mm| format!(", {mm:.0}mm left at the bottom"))
            .unwrap_or_default();
        let _ = writeln!(
            out,
            "{}, within the {} allowed{left}.",
            plural(report.pages, "page", "pages"),
            plural(report.max_pages, "page", "pages"),
        );
    }

    if report.loose_line_severity == super::Severity::Silent {
        out.push_str("Half-empty lines: not checked.\n");
        return out;
    }

    if report.loose_lines.is_empty() {
        out.push_str("No half-empty lines.\n");
        return out;
    }

    let _ = writeln!(
        out,
        "{}, {:.1} lines' worth of space:",
        plural(
            report.loose_lines.len(),
            "half-empty line",
            "half-empty lines"
        ),
        report.wasted_lines,
    );
    for found in &report.loose_lines {
        let _ = writeln!(out, "  {}:{} {}", file, found.line, message(found));
    }
    out
}

/// One finding, in the voice the caller can act on.
pub fn message(found: &LooseLine) -> String {
    let text = &found.text;
    match found.kind {
        LooseKind::ShortLine => format!(
            "short line: \"{text}\" fills {} of its line, with room for about {} more words.",
            percent(found.fill),
            found.room_words.unwrap_or(1),
        ),
        LooseKind::ShortLastLine => format!(
            "short last line: \"{text}\" ends at {} of a line. Reword to pull it up, or say more.",
            percent(found.fill),
        ),
        LooseKind::LooseLine => match &found.bumped {
            Some(word) => format!(
                "loose line: \"{text}\" leaves {} of a line empty before \"{word}\", which did not fit.",
                percent(1.0 - found.fill),
            ),
            None => format!(
                "loose line: \"{text}\" leaves {} of a line empty.",
                percent(1.0 - found.fill),
            ),
        },
        LooseKind::Widow => format!(
            "widow: \"{text}\" ends with one word on a line of its own. Reword to pull it up."
        ),
    }
}

fn percent(fill: f64) -> String {
    format!("{:.0}%", fill * 100.0)
}

fn plural(count: usize, one: &str, many: &str) -> String {
    format!("{count} {}", if count == 1 { one } else { many })
}
