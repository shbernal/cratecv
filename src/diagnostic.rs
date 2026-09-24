//! What the caller is told when a resume does not load.

use std::fmt;

use serde::Serialize;

/// One problem with the input, located precisely enough to fix without
/// opening the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    /// Canonical path into the YAML, such as `sections[1].entries[0].details[2]`.
    pub path: String,
    /// 1-based line.
    pub line: usize,
    /// 1-based column.
    pub column: usize,
    pub message: String,
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)?;
        if !self.path.is_empty() {
            write!(f, " at {}", self.path)?;
        }
        write!(f, ": {}", self.message)
    }
}
