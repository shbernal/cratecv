//! YAML in, validated resume out, or diagnostics precise enough to act on.
//!
//! Loading is two passes over one parse. `marked-yaml` keeps a span for every
//! node, so the typed resume is deserialized from that tree and the rules are
//! then checked against the typed values, looking each violation's span back up
//! by its path.

mod locate;
mod path;
mod types;
mod validate;

use marked_yaml::{LoadError, LoaderOptions, Node};

use crate::diagnostic::Diagnostic;

pub use locate::Locator;
pub use path::Path;
pub use types::{Bullet, Contact, Entry, Headline, Layout, Resume, Section, Skill, Source, THEMES};

/// Parse and validate a resume.
pub fn load(yaml: &str) -> Result<Resume, Vec<Diagnostic>> {
    let node = parse(yaml)?;
    let resume: Resume = marked_yaml::from_node(&node).map_err(|error| vec![from_node(&error)])?;

    let violations = validate::validate(&resume, &node);
    if violations.is_empty() {
        Ok(resume)
    } else {
        Err(violations)
    }
}

/// Load a settings file with the same strict, span-aware loader the resume
/// uses, so a typo in either gets the same diagnostic.
pub fn load_settings<T: serde::de::DeserializeOwned>(yaml: &str) -> Result<T, Vec<Diagnostic>> {
    let node = parse(yaml)?;
    marked_yaml::from_node(&node).map_err(|error| vec![from_node(&error)])
}

/// Parse YAML into the marked node tree both passes read.
pub(crate) fn parse(yaml: &str) -> Result<Node, Vec<Diagnostic>> {
    let options = LoaderOptions::default().error_on_duplicate_keys(true);
    marked_yaml::parse_yaml_with_options(0, yaml, options).map_err(|error| vec![from_load(&error)])
}

fn from_node(error: &marked_yaml::FromNodeError) -> Diagnostic {
    let (line, column) = match error.start_mark() {
        Some(mark) => (mark.line(), mark.column()),
        None => (1, 1),
    };
    Diagnostic {
        path: error.path().unwrap_or_default().to_owned(),
        line,
        column,
        message: error.to_string(),
    }
}

fn from_load(error: &LoadError) -> Diagnostic {
    let mark = match error {
        LoadError::TopLevelMustBeMapping(m)
        | LoadError::TopLevelMustBeSequence(m)
        | LoadError::UnexpectedAnchor(m)
        | LoadError::MappingKeyMustBeScalar(m)
        | LoadError::UnexpectedTag(m)
        | LoadError::ScanError(m, _) => Some(*m),
        LoadError::DuplicateKey(inner) => inner.key.span().start().copied(),
    };
    Diagnostic {
        path: String::new(),
        line: mark.map_or(1, |m| m.line()),
        column: mark.map_or(1, |m| m.column()),
        message: error.to_string(),
    }
}
