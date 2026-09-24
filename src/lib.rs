//! Compile a YAML resume to a one-page PDF and report what is wrong with it.

pub mod diagnostic;
pub mod error;
pub mod layout;
pub mod report;
pub mod schema;
pub mod world;

pub use diagnostic::Diagnostic;
pub use error::Error;
pub use report::{Check, Report};
pub use schema::{Resume, load};

use typst_layout::PagedDocument;
use typst_library::foundations::{Dict, Smart};
use world::ResumeWorld;

/// Lay a validated resume out.
pub fn compile(resume: &Resume) -> Result<PagedDocument, Error> {
    let theme = resume
        .cratecv
        .as_ref()
        .and_then(|settings| settings.theme.as_deref())
        .unwrap_or("default");
    let world = ResumeWorld::new(resume, theme, Dict::new())?;
    let document = world.compile();
    world::evict_cache();
    document
}

/// Load, lay out and judge a resume in one step: everything `check` does.
pub fn check(yaml: &str, settings: &Check) -> Result<Report, Error> {
    let resume = match load(yaml) {
        Ok(resume) => resume,
        Err(errors) => return Ok(Report::rejected(errors, settings)),
    };
    let document = compile(&resume)?;
    let measured = layout::measure(&document)?;
    Ok(report::report(
        &measured,
        &schema::Locator::new(yaml),
        settings,
    ))
}

/// Export a laid-out document to PDF bytes.
pub fn export_pdf(document: &PagedDocument) -> Result<Vec<u8>, Error> {
    let options = typst_pdf::PdfOptions {
        creator: Smart::Custom(Some(creator())),
        ..Default::default()
    };
    typst_pdf::pdf(document, &options)
        .map_err(|errors| Error::Export(errors.iter().map(|e| e.message.to_string()).collect()))
}

fn creator() -> String {
    format!("cratecv {}", env!("CARGO_PKG_VERSION"))
}
