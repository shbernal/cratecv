//! Measuring the laid-out page.
//!
//! Everything here reads the frame tree Typst produced, never the exported PDF.
//! `Frame::items()` yields positioned `FrameItem`s, `TextItem::width()` is the
//! advance of a run, and the template's markers say which YAML path a block
//! came from and what measure it was broken against. Those are measurements,
//! not inferences, which is the whole reason the guardrails sit here.

use typst_layout::PagedDocument;
use typst_library::foundations::Value;
use typst_library::introspection::{MetadataElem, Tag};
use typst_library::layout::{Abs, Frame, FrameItem, Point};

use crate::diagnostic::Diagnostic;
use crate::error::Error;

/// One laid-out line of a measured block.
#[derive(Debug, Clone)]
pub struct Line {
    /// Sum of the advances of the runs on it, in points.
    pub width: f64,
    pub words: usize,
    pub text: String,
}

impl Line {
    /// How much of the measure it was given this line covers.
    pub fn fill(&self, available: f64) -> f64 {
        if available <= 0.0 {
            1.0
        } else {
            self.width / available
        }
    }
}

/// A block the template marked as written to fill its width.
#[derive(Debug, Clone)]
pub struct Block {
    /// Canonical path into the YAML.
    pub path: String,
    /// The measure the layout engine broke this block against, in points.
    pub available: f64,
    pub lines: Vec<Line>,
}

impl Block {
    /// The block's text, rejoined across the lines it wrapped onto.
    pub fn text(&self) -> String {
        self.lines
            .iter()
            .map(|line| line.text.trim())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// What one compiled document measures out to.
#[derive(Debug, Clone)]
pub struct Measured {
    pub pages: usize,
    /// Every page's height, in points, in order.
    pub page_heights: Vec<f64>,
    /// How far down the last page content reaches, in points.
    pub last_ink: f64,
    pub blocks: Vec<Block>,
}

impl Measured {
    /// How far past `max_pages` the content runs, in millimetres. `None` when
    /// it fits. Measured, never estimated: full pages beyond the limit plus how
    /// far down the final page the last run sits.
    pub fn overflow_mm(&self, max_pages: usize) -> Option<f64> {
        if self.pages <= max_pages {
            return None;
        }
        let beyond: f64 = self.page_heights[max_pages..self.pages - 1].iter().sum();
        Some(Abs::pt(beyond + self.last_ink).to_mm())
    }
}

/// Runs and markers, in the order the page lays them down.
enum Item {
    Open {
        path: String,
        available: f64,
    },
    Close,
    Run {
        baseline: f64,
        width: f64,
        text: String,
    },
}

/// Read the page geometry the guardrails need out of a compiled document.
pub fn measure(document: &PagedDocument) -> Result<Measured, Error> {
    let mut page_heights = Vec::new();
    let mut last_ink: f64 = 0.0;
    let mut blocks: Vec<Block> = Vec::new();

    for (number, page) in document.pages().iter().enumerate() {
        let size = page.frame.size();
        page_heights.push(size.y.to_pt());

        let mut items = Vec::new();
        walk(&page.frame, Point::zero(), size.y, number + 1, &mut items)?;

        if number + 1 == document.pages().len() {
            last_ink = items
                .iter()
                .filter_map(|item| match item {
                    Item::Run { baseline, .. } => Some(*baseline),
                    _ => None,
                })
                .fold(0.0, f64::max);
        }

        collect(items, &mut blocks);
    }

    Ok(Measured {
        pages: document.pages().len(),
        page_heights,
        last_ink,
        blocks,
    })
}

/// Runs whose baselines differ by less than this share of their width are on
/// one line. Exact equality would split a line at every inline glyph, a
/// subscript or a size change.
const BASELINE_EPSILON: f64 = 1.0;

/// Attribute the runs to the block that is open around them. The template
/// brackets each measurable block with a marker, so this is a scan rather than
/// a guess at where a block ends.
fn collect(items: Vec<Item>, blocks: &mut Vec<Block>) {
    let mut open: Option<Block> = None;
    let mut baseline: Option<f64> = None;

    for item in items {
        match item {
            Item::Open { path, available } => {
                baseline = None;
                open = Some(Block {
                    path,
                    available,
                    lines: Vec::new(),
                });
            }
            Item::Close => {
                if let Some(block) = open.take() {
                    blocks.push(block);
                }
            }
            Item::Run {
                baseline: y,
                width,
                text,
            } => {
                let Some(block) = open.as_mut() else { continue };
                if baseline.is_none_or(|last| (y - last).abs() > BASELINE_EPSILON) {
                    baseline = Some(y);
                    block.lines.push(Line {
                        width: 0.0,
                        words: 0,
                        text: String::new(),
                    });
                }
                if let Some(line) = block.lines.last_mut() {
                    line.width += width;
                    line.text.push_str(&text);
                }
            }
        }
    }
    if let Some(block) = open.take() {
        blocks.push(block);
    }
    for block in blocks.iter_mut() {
        for line in &mut block.lines {
            line.words = line.text.split_whitespace().count();
        }
        block.lines.retain(|line| line.words > 0);
    }
}

/// Two things this asserts rather than tolerates, because either one makes the
/// numbers above meaningless and neither is something a resume can cause.
fn walk(
    frame: &Frame,
    origin: Point,
    page_height: Abs,
    page: usize,
    out: &mut Vec<Item>,
) -> Result<(), Error> {
    for (pos, item) in frame.items() {
        let at = origin + *pos;
        match item {
            FrameItem::Group(group) => {
                let t = group.transform;
                let translation_only = t.sx.get() == 1.0
                    && t.sy.get() == 1.0
                    && t.kx.get() == 0.0
                    && t.ky.get() == 0.0;
                if !translation_only {
                    return Err(internal(format!(
                        "page {page} holds a group that is rotated or scaled, so positions \
                         cannot be added up"
                    )));
                }
                let inner = at + Point::new(t.tx, t.ty);
                walk(&group.frame, inner, page_height, page, out)?;
            }
            FrameItem::Text(run) => {
                if at.y < Abs::zero() || at.y > page_height {
                    return Err(internal(format!(
                        "page {page} draws text at {}pt, outside the page",
                        at.y.to_pt().round()
                    )));
                }
                out.push(Item::Run {
                    baseline: at.y.to_pt(),
                    width: run.width().to_pt(),
                    text: run.text.to_string(),
                });
            }
            FrameItem::Tag(Tag::Start(content, _)) => {
                if let Some(item) = marker(content) {
                    out.push(item);
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// The payload the template attached to the start or end of a measurable block.
fn marker(content: &typst_library::foundations::Content) -> Option<Item> {
    let Value::Dict(dict) = &content.to_packed::<MetadataElem>()?.value else {
        return None;
    };
    if dict.get("end").is_ok() {
        return Some(Item::Close);
    }
    let path = match dict.get("path") {
        Ok(Value::Str(text)) => text.to_string(),
        _ => return None,
    };
    let available = match dict.get("available") {
        Ok(Value::Float(pt)) => *pt,
        Ok(Value::Int(pt)) => *pt as f64,
        _ => return None,
    };
    Some(Item::Open { path, available })
}

fn internal(message: String) -> Error {
    Error::Template(vec![Diagnostic {
        path: String::new(),
        line: 1,
        column: 1,
        message,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::ResumeWorld;

    fn laid_out(template: &str) -> PagedDocument {
        let world = ResumeWorld::from_template(template);
        let document = world
            .compile()
            .expect("the fixture template should compile");
        crate::world::evict_cache();
        document
    }

    #[test]
    fn a_rotated_group_is_refused() {
        let document = laid_out("#rotate(12deg)[Slanted]");
        let Err(Error::Template(found)) = measure(&document) else {
            panic!("a rotation must not be walked as if it were a translation");
        };
        assert!(found[0].message.contains("rotated or scaled"), "{found:?}");
    }

    #[test]
    fn a_scaled_group_is_refused() {
        let document = laid_out("#scale(180%, reflow: false)[Big]");
        assert!(
            matches!(measure(&document), Err(Error::Template(_))),
            "a scale must not be walked as if it were a translation"
        );
    }

    #[test]
    fn text_drawn_off_the_page_is_refused() {
        let document = laid_out("#place(top, dy: -400pt)[Escaped]");
        let Err(Error::Template(found)) = measure(&document) else {
            panic!("content outside the page must not be reported as a finding");
        };
        assert!(found[0].message.contains("outside the page"), "{found:?}");
    }

    #[test]
    fn a_plain_page_walks_clean() {
        let document = laid_out("Ordinary text on an ordinary page.");
        let measured = measure(&document).expect("nothing here is out of the ordinary");
        assert_eq!(measured.pages, 1);
        assert!(measured.blocks.is_empty(), "nothing was marked");
    }
}
