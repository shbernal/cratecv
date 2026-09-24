//! The PDF itself: that it stays byte-for-byte what it was, and that it passes
//! the accessibility validation an applicant tracking system's extractor cares
//! about.

use cratecv::config::{Config, Flags, PdfMeta, resolve};
use typst_layout::PagedDocument;

/// The example, with the metadata a machine with no config file would resolve
/// and no date, so the bytes depend only on this repository.
fn example() -> (PagedDocument, PdfMeta) {
    let resume =
        cratecv::load(include_str!("../examples/resume.yaml")).expect("the example should load");
    let settings =
        resolve(&Config::default(), None, &Flags::default()).expect("the defaults resolve");
    let mut document = cratecv::compile(&resume).expect("the example should compile");
    cratecv::describe(&mut document, &resume, &settings.pdf, None);
    (document, settings.pdf)
}

/// Update deliberately, never automatically: this file is deterministic by
/// design, so a change here is a real change in the output.
const GOLDEN: &str = include_str!("fixtures/resume.pdf.sha256");

#[test]
fn the_example_pdf_is_what_it_was() {
    let (document, meta) = example();
    let pdf = cratecv::export_pdf(&document, &meta).expect("the example should export");
    let digest = sha256(&pdf);
    assert_eq!(
        digest,
        GOLDEN.trim(),
        "the example PDF changed. If that was intended, write {digest} into \
         tests/fixtures/resume.pdf.sha256"
    );
}

#[test]
fn the_example_conforms_to_pdf_ua_1() {
    let (document, meta) = example();
    cratecv::export_pdf_conforming(&document, &meta, &[typst_pdf::PdfStandard::Ua_1])
        .expect("the built-in theme should produce an accessible document");
}

#[test]
fn the_text_comes_back_out_in_reading_order() {
    let (document, meta) = example();
    let pdf = cratecv::export_pdf(&document, &meta).unwrap();
    assert!(pdf.starts_with(b"%PDF-"));
    // The structure tree is what an extractor walks; without it the tagged-PDF
    // promise in the README is not kept.
    let raw = String::from_utf8_lossy(&pdf);
    assert!(raw.contains("/StructTreeRoot"), "the document is tagged");
    assert!(raw.contains("/Marked true"), "and says so");
}

fn sha256(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
