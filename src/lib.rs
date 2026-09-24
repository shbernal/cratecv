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

/// Render one page to PNG at the given resolution.
pub fn export_png(document: &PagedDocument, dpi: f64) -> Result<Vec<u8>, Error> {
    let page = document
        .pages()
        .first()
        .ok_or_else(|| Error::Export("the document has no pages".into()))?;
    let options = typst_render::RenderOptions {
        pixel_per_pt: typst_utils::Scalar::new(dpi / 72.0),
        ..Default::default()
    };
    typst_render::render(page, &options)
        .encode_png()
        .map_err(|why| Error::Export(why.to_string()))
}

/// Render the whole document to one SVG, pages stacked.
pub fn export_svg(document: &PagedDocument) -> String {
    typst_svg::svg_merged(
        document,
        &typst_svg::SvgOptions::default(),
        typst_library::layout::Abs::pt(12.0),
    )
}

fn creator() -> String {
    format!("cratecv {}", env!("CARGO_PKG_VERSION"))
}
