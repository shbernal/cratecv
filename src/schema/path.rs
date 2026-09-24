//! The one canonical spelling of a location inside a resume.
//!
//! A path is both the join key the layout report uses to attribute a line back
//! to its bullet and the address a diagnostic resolves to recover a span.

use std::fmt;

use marked_yaml::Node;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Segment {
    Key(String),
    Index(usize),
}

/// A location inside a resume, rendered as `sections[1].entries[0].details[2]`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Path(Vec<Segment>);

impl Path {
    pub fn root() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn key(&self, key: &str) -> Self {
        let mut next = self.clone();
        next.0.push(Segment::Key(key.to_owned()));
        next
    }

    #[must_use]
    pub fn index(&self, index: usize) -> Self {
        let mut next = self.clone();
        next.0.push(Segment::Index(index));
        next
    }

    /// Where this path points in a parsed document, as a 1-based line and
    /// column. Falls back to the nearest ancestor that does resolve, so a
    /// diagnostic always has somewhere to point.
    pub fn locate(&self, root: &Node) -> (usize, usize) {
        let mut best = start_of(root);
        let mut node = root;
        for segment in &self.0 {
            let next = match segment {
                Segment::Key(key) => node.as_mapping().and_then(|m| m.get_node(key)),
                Segment::Index(index) => node.as_sequence().and_then(|s| s.get_node(*index)),
            };
            match next {
                Some(next) => {
                    node = next;
                    if let Some(mark) = start_of(node) {
                        best = Some(mark);
                    }
                }
                None => break,
            }
        }
        best.unwrap_or((1, 1))
    }
}

fn start_of(node: &Node) -> Option<(usize, usize)> {
    node.span().start().map(|m| (m.line(), m.column()))
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut first = true;
        for segment in &self.0 {
            match segment {
                Segment::Key(key) => {
                    if !first {
                        f.write_str(".")?;
                    }
                    f.write_str(key)?;
                }
                Segment::Index(index) => write!(f, "[{index}]")?,
            }
            first = false;
        }
        Ok(())
    }
}
