//! Compile a YAML resume to a one-page PDF and report what is wrong with it.

pub mod config;
pub mod diagnostic;
pub mod error;
pub mod layout;
pub mod pdf;
pub mod report;
pub mod schema;
pub mod world;

pub use diagnostic::Diagnostic;
pub use error::Error;
pub use report::{Check, Report};
pub use schema::{Resume, load};

/// A JSON Schema for the resume format, generated from the types the parser
/// uses, so it cannot describe something the parser would reject.
pub fn json_schema() -> serde_json::Value {
    let mut schema = schemars::schema_for!(Resume);
    schema.insert(
        "title".to_owned(),
        serde_json::Value::String("cratecv resume".to_owned()),
    );
    schema.insert(
        "description".to_owned(),
        serde_json::Value::String(
            "A resume compiled by cratecv. Unknown keys are rejected, and every string              must carry text."
                .to_owned(),
        ),
    );
    schema.to_value()
}

/// What `cratecv init` writes: a resume small enough to read at a glance and
/// complete enough to compile.
pub const STARTER_RESUME: &str = include_str!("../templates/starter.yaml");

use typst_layout::PagedDocument;
use typst_library::foundations::{Dict, Smart};
use world::ResumeWorld;

/// Lay a validated resume out with the theme its own block names.
pub fn compile(resume: &Resume) -> Result<PagedDocument, Error> {
    let theme = resume
        .cratecv
        .as_ref()
        .and_then(|settings| settings.theme.as_deref())
        .unwrap_or("default");
    compile_theme(resume, theme)
}

/// Lay a validated resume out with a named theme.
pub fn compile_theme(resume: &Resume, theme: &str) -> Result<PagedDocument, Error> {
    let world = ResumeWorld::new(resume, theme, Dict::new())?;
    let document = world.compile();
    world::evict_cache();
    document
}

/// Fill in the document information Typst writes into the Info dictionary and
/// the XMP packet. Title and author come from the resume's name, because nobody
/// should have to state it twice.
pub fn describe(
    document: &mut PagedDocument,
    resume: &Resume,
    meta: &config::PdfMeta,
    when: Option<typst_library::foundations::Datetime>,
) {
    let info = document.info_mut();
    info.title = Some(format!("{} - Resume", resume.name).as_str().into());
    info.author = vec![resume.name.as_str().into()];
    info.keywords = meta
        .keywords
        .iter()
        .map(|word| word.as_str().into())
        .collect();
    info.date = Smart::Custom(when);
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

/// Export a laid-out document to PDF bytes, carrying the resolved metadata.
pub fn export_pdf(document: &PagedDocument, meta: &config::PdfMeta) -> Result<Vec<u8>, Error> {
    export_pdf_conforming(document, meta, &[])
}

/// Export while enforcing PDF standards. Typst validates at export and refuses
/// to write when it finds a critical problem, so asking for `ua-1` is a
/// conformance check as much as an export mode.
pub fn export_pdf_conforming(
    document: &PagedDocument,
    meta: &config::PdfMeta,
    standards: &[typst_pdf::PdfStandard],
) -> Result<Vec<u8>, Error> {
    let standards =
        typst_pdf::PdfStandards::new(standards).map_err(|why| Error::Export(format!("{why:?}")))?;
    let options = typst_pdf::PdfOptions {
        creator: Smart::Custom(Some(meta.creator.clone())),
        standards,
        ..Default::default()
    };
    let bytes = typst_pdf::pdf(document, &options)
        .map_err(|errors| Error::Export(errors.iter().map(|e| e.message.to_string()).collect()))?;
    pdf::stamp(bytes, meta)
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
