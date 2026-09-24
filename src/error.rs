//! What can go wrong, kept apart by whose fault it is.

use std::fmt;

use crate::diagnostic::Diagnostic;

#[derive(Debug)]
pub enum Error {
    /// The resume does not load or breaks a rule. The caller can fix it.
    Resume(Vec<Diagnostic>),
    /// The template does not compile. That is a bug in this tool.
    Template(Vec<Diagnostic>),
    /// Exporting the compiled document failed.
    Export(String),
    Io(std::io::Error),
}

impl Error {
    pub fn diagnostics(&self) -> &[Diagnostic] {
        match self {
            Error::Resume(found) | Error::Template(found) => found,
            _ => &[],
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Resume(found) => write!(f, "{} problem(s) in the resume", found.len()),
            Error::Template(_) => f.write_str("the built-in template failed to compile"),
            Error::Export(why) => write!(f, "could not export the document: {why}"),
            Error::Io(why) => why.fmt(f),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(why: std::io::Error) -> Self {
        Error::Io(why)
    }
}
