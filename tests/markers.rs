//! The marked blocks must reach the frame tree. There is no fallback route:
//! if this fails, nothing can be attributed back to the YAML.

use typst_layout::PagedDocument;
use typst_library::foundations::Value;
use typst_library::introspection::{MetadataElem, Tag};
use typst_library::layout::{Frame, FrameItem};

fn markers(frame: &Frame, out: &mut Vec<(String, f64)>) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => markers(&group.frame, out),
            FrameItem::Tag(Tag::Start(content, _)) => {
                let Some(metadata) = content.to_packed::<MetadataElem>() else {
                    continue;
                };
                let Value::Dict(dict) = &metadata.value else {
                    continue;
                };
                let path = match dict.get("path") {
                    Ok(Value::Str(text)) => Some(text.to_string()),
                    _ => None,
                };
                let available = match dict.get("available") {
                    Ok(Value::Float(pt)) => Some(*pt),
                    Ok(Value::Int(pt)) => Some(*pt as f64),
                    _ => None,
                };
                if let (Some(path), Some(available)) = (path, available) {
                    out.push((path, available));
                }
            }
            _ => {}
        }
    }
}

fn marked(document: &PagedDocument) -> Vec<(String, f64)> {
    let mut out = Vec::new();
    for page in document.pages() {
        markers(&page.frame, &mut out);
    }
    out
}

fn example() -> PagedDocument {
    let resume =
        cratecv::load(include_str!("../examples/resume.yaml")).expect("the example should load");
    cratecv::compile(&resume).expect("the example should compile")
}

#[test]
fn every_bullet_and_skill_value_is_marked() {
    let document = example();
    let found = marked(&document);
    let paths: Vec<&str> = found.iter().map(|(path, _)| path.as_str()).collect();

    assert!(
        paths.contains(&"sections[0].entries[0].details[0]"),
        "the first bullet is missing from {paths:?}"
    );
    assert!(
        paths.contains(&"sections[3].skills[0]"),
        "the first skill value is missing from {paths:?}"
    );
    // Eight bullets and three skill rows. The badges section is deliberately
    // absent: a badge is sized to its own text, so it has no measure to fall
    // short of.
    assert_eq!(found.len(), 11, "marked: {paths:?}");
    assert!(
        !paths.iter().any(|path| path.starts_with("sections[4]")),
        "badges are not measured: {paths:?}"
    );
}

#[test]
fn every_marker_carries_the_measure_it_was_laid_out_against() {
    for (path, available) in marked(&example()) {
        assert!(
            available > 100.0 && available < 600.0,
            "{path} reports an implausible measure of {available}pt"
        );
    }
}

#[test]
fn nothing_but_bullets_and_skill_values_is_marked() {
    for (path, _) in marked(&example()) {
        assert!(
            path.contains(".details[") || path.contains(".skills["),
            "{path} is measured, but only bullets and skill values should be"
        );
    }
}
