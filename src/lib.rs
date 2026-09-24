//! Compile a YAML resume to a one-page PDF and report what is wrong with it.

pub mod diagnostic;
pub mod schema;

pub use diagnostic::Diagnostic;
pub use schema::{Resume, load};
