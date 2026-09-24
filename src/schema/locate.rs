//! Looking a YAML line back up from a path.
//!
//! The layout report names a bullet by the path the template wrote into its
//! marker. A caller fixing that bullet wants a line number, so the parsed
//! document is kept around to resolve one.

use marked_yaml::Node;

use super::path::Path;

pub struct Locator {
    root: Node,
}

impl Locator {
    /// Parse the resume once more, purely for positions. The document already
    /// validated, so a failure here cannot happen and resolves to line 1.
    pub fn new(yaml: &str) -> Self {
        let root = super::parse(yaml).unwrap_or_else(|_| {
            Node::from(marked_yaml::types::MarkedMappingNode::new_empty(
                marked_yaml::Span::new_blank(),
            ))
        });
        Self { root }
    }

    /// The 1-based line the given path sits on, or the nearest ancestor's.
    pub fn line(&self, path: &str) -> usize {
        Path::parse(path)
            .map(|path| path.locate(&self.root).0)
            .unwrap_or(1)
    }
}
