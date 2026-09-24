//! The four kinds of half-empty line.
//!
//! Ported from the predecessor's loose-line checks with their thresholds
//! intact. They were tuned against real resumes, and they are the only numbers
//! in this project with provenance, so they live in the settings rather than
//! at the point of use.

use serde::Serialize;

use super::{LooseLine, LooseLines, Severity};
use crate::layout::{Block, Line, Measured};
use crate::schema::Locator;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum LooseKind {
    /// A block that never wrapped and has room for more words.
    ShortLine,
    /// A wrapped block whose last line barely starts.
    ShortLastLine,
    /// A line that stopped early because the next word did not fit.
    LooseLine,
    /// A wrapped block ending on one word of its own.
    Widow,
}

/// Warnings quote this much of a block, so one fits on a terminal line.
const EXCERPT: usize = 50;

pub(super) fn findings(
    measured: &Measured,
    locator: &Locator,
    settings: &LooseLines,
) -> Vec<LooseLine> {
    if settings.severity == Severity::Silent {
        return Vec::new();
    }
    let mut out = Vec::new();
    for block in &measured.blocks {
        if block.lines.is_empty() {
            continue;
        }
        let text = excerpt(&block.text());
        let line = locator.line(&block.path);

        if block.lines.len() == 1 {
            let only = &block.lines[0];
            let fill = only.fill(block.available);
            if fill < settings.short_line {
                out.push(LooseLine {
                    kind: LooseKind::ShortLine,
                    path: block.path.clone(),
                    line,
                    text,
                    fill,
                    room_words: Some(room_words(only, block.available)),
                    bumped: None,
                });
            }
            continue;
        }

        let last = block.lines.len() - 1;
        for (index, laid) in block.lines.iter().enumerate() {
            let fill = laid.fill(block.available);
            let found = if index == last {
                if laid.words <= settings.widow_words {
                    Some((LooseKind::Widow, None))
                } else if fill < settings.short_last_line {
                    Some((LooseKind::ShortLastLine, None))
                } else {
                    None
                }
            } else if 1.0 - fill > settings.loose_line_gap {
                // The word that broke the line opens the next one, and it is
                // the word to shorten or move.
                Some((LooseKind::LooseLine, bumped(block, index + 1)))
            } else {
                None
            };
            if let Some((kind, bumped)) = found {
                out.push(LooseLine {
                    kind,
                    path: block.path.clone(),
                    line,
                    text: text.clone(),
                    fill,
                    room_words: None,
                    bumped,
                });
            }
        }
    }
    out
}

/// How many more words of the same length would fit in what a line left empty.
fn room_words(line: &Line, available: f64) -> usize {
    if line.words == 0 {
        return 1;
    }
    let word = line.width / line.words as f64;
    if word <= 0.0 {
        return 1;
    }
    (((available - line.width) / word).round() as usize).max(1)
}

fn bumped(block: &Block, index: usize) -> Option<String> {
    let next = block.lines.get(index)?;
    next.text.split_whitespace().next().map(str::to_owned)
}

fn excerpt(text: &str) -> String {
    let single = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if single.chars().count() <= EXCERPT {
        return single;
    }
    let head: String = single.chars().take(EXCERPT - 1).collect();
    format!("{head}…")
}
