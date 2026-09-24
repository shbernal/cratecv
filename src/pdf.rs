//! Metadata the PDF writer does not expose.
//!
//! Typst covers title, author, keywords, date and Creator. It does not write a
//! Producer at all, and `PdfOptions` has no way to ask for one, so it is
//! written here by a pass over the finished bytes — in the Info dictionary and
//! in the XMP packet together, because a reader that trusts one and not the
//! other will otherwise see a contradiction.
//!
//! The ceiling is worth stating plainly: this changes metadata and nothing
//! else. A document claiming a different origin still carries this tool's font
//! subsets, object layout and structure tree. It makes output quiet, not
//! disguised.

use lopdf::{Dictionary, Document, Object};

use crate::config::PdfMeta;
use crate::error::Error;

/// XMP names the field in the Adobe PDF namespace.
const XMP_PRODUCER: &str = "pdf:Producer";

pub fn stamp(bytes: Vec<u8>, meta: &PdfMeta) -> Result<Vec<u8>, Error> {
    let mut document = Document::load_mem(&bytes).map_err(failed)?;
    info(&mut document, meta);
    xmp(&mut document, meta);

    let mut out = Vec::with_capacity(bytes.len());
    document
        .save_to(&mut out)
        .map_err(|why| Error::Export(format!("could not write the PDF metadata: {why}")))?;
    Ok(out)
}

/// PDF text strings are PDFDocEncoded unless they open with a UTF-16BE byte
/// order mark, and PDFDocEncoding is not UTF-8, so anything outside ASCII goes
/// in as UTF-16BE rather than as its own bytes.
fn text_string(value: &str) -> Object {
    if value.is_ascii() {
        return Object::string_literal(value);
    }
    let mut bytes = vec![0xFE, 0xFF];
    for unit in value.encode_utf16() {
        bytes.extend_from_slice(&unit.to_be_bytes());
    }
    Object::String(bytes, lopdf::StringFormat::Hexadecimal)
}

fn info(document: &mut Document, meta: &PdfMeta) {
    let producer = text_string(&meta.producer);
    match document.trailer.get(b"Info").and_then(Object::as_reference) {
        Ok(id) => {
            if let Ok(Object::Dictionary(dict)) = document.get_object_mut(id) {
                dict.set("Producer", producer);
            }
        }
        Err(_) => {
            let mut dict = Dictionary::new();
            dict.set("Producer", producer);
            let id = document.add_object(Object::Dictionary(dict));
            document.trailer.set("Info", Object::Reference(id));
        }
    }
}

fn xmp(document: &mut Document, meta: &PdfMeta) {
    let Some(id) = metadata_stream(document) else {
        return;
    };
    let Ok(Object::Stream(stream)) = document.get_object_mut(id) else {
        return;
    };
    let Ok(packet) = String::from_utf8(stream.content.clone()) else {
        return;
    };
    let patched = with_producer(&packet, &meta.producer);
    stream.set_plain_content(patched.into_bytes());
}

fn metadata_stream(document: &Document) -> Option<lopdf::ObjectId> {
    let catalog = document.catalog().ok()?;
    catalog.get(b"Metadata").and_then(Object::as_reference).ok()
}

/// Replace the Producer if the packet states one, and add it beside the other
/// `pdf:` properties otherwise.
fn with_producer(packet: &str, producer: &str) -> String {
    let element = format!("<{XMP_PRODUCER}>{}</{XMP_PRODUCER}>", escape(producer));
    let open = format!("<{XMP_PRODUCER}>");
    let close = format!("</{XMP_PRODUCER}>");

    if let Some(start) = packet.find(&open)
        && let Some(end) = packet[start..].find(&close)
    {
        let end = start + end + close.len();
        return format!("{}{element}{}", &packet[..start], &packet[end..]);
    }

    match packet.find("</rdf:Description>") {
        Some(at) => format!("{}{element}{}", &packet[..at], &packet[at..]),
        None => packet.to_owned(),
    }
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn failed(why: lopdf::Error) -> Error {
    Error::Export(format!("could not write the PDF metadata: {why}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACKET: &str =
        "<rdf:Description rdf:about=\"\"><pdf:PDFVersion>1.7</pdf:PDFVersion></rdf:Description>";

    #[test]
    fn a_missing_producer_is_added() {
        let out = with_producer(PACKET, "Somebody Else");
        assert!(
            out.contains("<pdf:Producer>Somebody Else</pdf:Producer>"),
            "{out}"
        );
        assert!(out.contains("<pdf:PDFVersion>"), "the rest survives: {out}");
    }

    #[test]
    fn an_existing_producer_is_replaced_not_duplicated() {
        let once = with_producer(PACKET, "First");
        let twice = with_producer(&once, "Second");
        assert_eq!(twice.matches("<pdf:Producer>").count(), 1, "{twice}");
        assert!(twice.contains("Second"), "{twice}");
        assert!(!twice.contains("First"), "{twice}");
    }

    #[test]
    fn a_non_ascii_name_goes_in_as_utf16() {
        let Object::String(bytes, lopdf::StringFormat::Hexadecimal) = text_string("Word®") else {
            panic!("anything outside ASCII needs the byte order mark");
        };
        assert_eq!(&bytes[..2], &[0xFE, 0xFF]);
        let Object::String(plain, lopdf::StringFormat::Literal) = text_string("Word") else {
            panic!("plain ASCII stays literal");
        };
        assert_eq!(plain, b"Word");
    }

    #[test]
    fn markup_in_a_name_is_escaped() {
        let out = with_producer(PACKET, "A & B <thing>");
        assert!(out.contains("A &amp; B &lt;thing&gt;"), "{out}");
    }
}
