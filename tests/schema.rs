//! Every rule in the schema, against a broken fixture that names its path.

use cratecv::{Diagnostic, load};

const MINIMAL: &str = "\
name: Ada
contact: {}
sections:
  - title: Experience
    entries:
      - dates: 2020
";

fn fails(yaml: &str) -> Vec<Diagnostic> {
    load(yaml).expect_err("expected this resume to be rejected")
}

fn only(yaml: &str) -> Diagnostic {
    let mut found = fails(yaml);
    assert_eq!(found.len(), 1, "expected one diagnostic, got {found:?}");
    found.remove(0)
}

#[test]
fn the_example_loads_clean() {
    let yaml = include_str!("../examples/resume.yaml");
    let resume = load(yaml).expect("the example resume should load");
    assert_eq!(resume.name, "Mira Halvorsen");
    assert_eq!(resume.sections.len(), 5);
}

#[test]
fn an_entry_needs_only_dates() {
    let resume = load(MINIMAL).expect("a bare entry is legal");
    let entry = &resume.sections[0].entries.as_ref().unwrap()[0];
    assert_eq!(entry.dates, "2020");
    assert!(entry.role.is_none());
    assert!(entry.details.is_none());
}

#[test]
fn an_unknown_key_is_rejected_where_it_sits() {
    let found = only(
        "\
name: Ada
contact: {}
sections:
  - title: Experience
    entries:
      - dates: 2020
        employer: Somewhere
",
    );
    assert_eq!(found.path, "sections[0].entries[0].employer");
    assert_eq!(found.line, 7);
    assert!(found.message.contains("employer"), "{}", found.message);
}

#[test]
fn an_empty_string_is_rejected() {
    let found = only(
        "\
name: Ada
contact: {}
sections:
  - title: Experience
    entries:
      - dates: 2020
        role: \"\"
",
    );
    assert_eq!(found.path, "sections[0].entries[0].role");
    assert_eq!(found.line, 7);
    assert!(found.message.contains("empty"), "{}", found.message);
}

#[test]
fn a_section_takes_entries_or_skills_but_not_both() {
    let found = only(
        "\
name: Ada
contact: {}
sections:
  - title: Both
    entries:
      - dates: 2020
    skills:
      - label: Languages
        value: Rust
",
    );
    assert_eq!(found.path, "sections[0]");
    assert_eq!(found.line, 4);
    assert!(found.message.contains("not both"), "{}", found.message);
}

#[test]
fn a_section_needs_one_of_entries_or_skills() {
    let found = only(
        "\
name: Ada
contact: {}
sections:
  - title: Empty
",
    );
    assert_eq!(found.path, "sections[0]");
    assert!(found.message.contains("either"), "{}", found.message);
}

#[test]
fn only_a_skills_section_takes_a_layout() {
    let found = only(
        "\
name: Ada
contact: {}
sections:
  - title: Experience
    layout: badges
    entries:
      - dates: 2020
",
    );
    assert_eq!(found.path, "sections[0].layout");
    assert_eq!(found.line, 5);
}

#[test]
fn a_resume_needs_a_section() {
    let found = only(
        "\
name: Ada
contact: {}
sections: []
",
    );
    assert_eq!(found.path, "sections");
    assert_eq!(found.line, 3);
}

#[test]
fn an_unknown_theme_lists_what_exists() {
    let found = only(
        "\
name: Ada
contact: {}
cratecv:
  theme: brutalist
sections:
  - title: Experience
    entries:
      - dates: 2020
",
    );
    assert_eq!(found.path, "cratecv.theme");
    assert_eq!(found.line, 4);
    assert!(found.message.contains("default"), "{}", found.message);
}

#[test]
fn source_is_one_string_or_a_list() {
    let resume = load(
        "\
name: Ada
contact: {}
sections:
  - title: Experience
    entries:
      - dates: 2020
        details:
          - text: One
            source: a.yaml
          - text: Two
            source: [a.yaml, b.yaml]
",
    )
    .expect("both spellings of source are legal");
    let details = resume.sections[0].entries.as_ref().unwrap()[0]
        .details
        .as_ref()
        .unwrap();
    assert!(matches!(
        details[0].source,
        Some(cratecv::schema::Source::One(_))
    ));
    assert!(matches!(
        details[1].source,
        Some(cratecv::schema::Source::Many(_))
    ));
}

#[test]
fn every_violation_is_reported_at_once() {
    let found = fails(
        "\
name: \"\"
contact: {}
sections:
  - title: Experience
    entries:
      - dates: \"\"
        role: \"\"
",
    );
    let paths: Vec<_> = found.iter().map(|d| d.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "name",
            "sections[0].entries[0].dates",
            "sections[0].entries[0].role"
        ]
    );
}

#[test]
fn a_duplicate_key_is_not_silently_dropped() {
    let found = only(
        "\
name: Ada
name: Grace
contact: {}
sections:
  - title: Experience
    entries:
      - dates: 2020
",
    );
    assert!(
        found.message.to_lowercase().contains("duplicate"),
        "{}",
        found.message
    );
}
