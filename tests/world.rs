//! The compiler sees only what the binary carries.

use cratecv::world::{FAMILY, ResumeWorld};
use typst_library::World;
use typst_library::foundations::Dict;

fn example() -> cratecv::Resume {
    cratecv::load(include_str!("../examples/resume.yaml")).expect("the example should load")
}

fn world() -> ResumeWorld {
    ResumeWorld::new(&example(), "default", Dict::new()).expect("the default theme exists")
}

#[test]
fn the_font_book_holds_only_the_embedded_faces() {
    let world = world();
    let families: Vec<_> = world.book().families().map(|(name, _)| name).collect();
    assert_eq!(families, [FAMILY]);

    let book = world.book();
    let weights: Vec<_> = book
        .families()
        .flat_map(|(_, indices)| indices)
        .filter_map(|index| book.info(index))
        .map(|info| (info.variant.weight.to_number(), info.variant.style))
        .collect();
    assert_eq!(weights.len(), 4, "four faces are compiled in: {weights:?}");
    assert!(weights.contains(&(400, typst_library::text::FontStyle::Normal)));
    assert!(weights.contains(&(600, typst_library::text::FontStyle::Normal)));
    assert!(weights.contains(&(800, typst_library::text::FontStyle::Normal)));
    assert!(weights.contains(&(400, typst_library::text::FontStyle::Italic)));
}

#[test]
fn every_embedded_face_loads() {
    let world = world();
    for index in 0..4 {
        assert!(world.font(index).is_some(), "face {index} should load");
    }
    assert!(world.font(4).is_none(), "there is no fifth face");
}

#[test]
fn an_unknown_theme_never_reaches_the_compiler() {
    let Err(error) = ResumeWorld::new(&example(), "brutalist", Dict::new()) else {
        panic!("an unknown theme has no source to serve");
    };
    assert_eq!(error.diagnostics()[0].path, "cratecv.theme");
}

#[test]
fn the_template_only_sees_validated_data() {
    let world = world();
    let data = world
        .file(cratecv::world::data_id())
        .expect("the resume is served to the template");
    let json: serde_json::Value =
        serde_json::from_slice(data.as_slice()).expect("it is serialized by this tool");
    assert!(
        json.get("cratecv").is_none(),
        "the settings block is consumed, never rendered"
    );
    assert_eq!(json["name"], "Mira Halvorsen");
}

#[test]
fn the_example_compiles_and_exports() {
    let document = cratecv::compile(&example()).expect("the example should compile");
    assert!(!document.pages().is_empty());
    let pdf = cratecv::export_pdf(&document).expect("the document should export");
    assert!(pdf.starts_with(b"%PDF-"));
}
