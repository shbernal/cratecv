//! The report, measured against fixtures whose numbers can be checked by hand.

use cratecv::layout::measure;
use cratecv::report::{Check, LooseKind, Report, Severity, report};
use cratecv::schema::Locator;

fn judge(name: &str, check: &Check) -> Report {
    let yaml = std::fs::read_to_string(format!("tests/fixtures/{name}.yaml"))
        .expect("the fixture should exist");
    let resume = cratecv::load(&yaml).expect("the fixture should load");
    let document = cratecv::compile(&resume).expect("the fixture should compile");
    let measured = measure(&document).expect("the fixture should walk");
    report(&measured, &Locator::new(&yaml), check)
}

fn only(name: &str) -> cratecv::report::LooseLine {
    let mut found = judge(name, &Check::default()).loose_lines;
    assert_eq!(found.len(), 1, "expected one finding, got {found:?}");
    found.remove(0)
}

#[test]
fn a_resume_with_nothing_wrong_says_so() {
    let report = judge("clean", &Check::default());
    assert!(report.ok);
    assert_eq!(report.pages, 1);
    assert!(report.overflow_mm.is_none());
    assert!(report.schema_errors.is_empty());
    assert!(report.loose_lines.is_empty(), "{:?}", report.loose_lines);
    assert_eq!(report.wasted_lines, 0.0);
}

#[test]
fn a_block_that_never_wrapped_is_a_short_line() {
    let found = only("short-line");
    assert_eq!(found.kind, LooseKind::ShortLine);
    assert_eq!(found.path, "sections[0].entries[0].details[0]");
    assert_eq!(found.line, 9);
    assert!((0.27..0.30).contains(&found.fill), "fill {}", found.fill);
    assert!(found.room_words.unwrap() > 5);
}

#[test]
fn one_word_alone_on_the_last_line_is_a_widow() {
    let found = only("widow");
    assert_eq!(found.kind, LooseKind::Widow);
    assert_eq!(found.path, "sections[0].entries[0].details[0]");
    assert_eq!(found.line, 9);
    assert!(found.fill < 0.2, "fill {}", found.fill);
}

#[test]
fn a_last_line_that_barely_starts_is_a_short_last_line() {
    let found = only("short-last-line");
    assert_eq!(found.kind, LooseKind::ShortLastLine);
    assert_eq!(found.line, 9);
    assert!(found.fill < 0.35, "fill {}", found.fill);
}

#[test]
fn a_line_cut_short_names_the_word_that_did_not_fit() {
    let found = only("loose-line");
    assert_eq!(found.kind, LooseKind::LooseLine);
    assert_eq!(found.line, 9);
    assert!(1.0 - found.fill > 0.20, "fill {}", found.fill);
    assert_eq!(
        found.bumped.as_deref(),
        Some("reproducibilityverificationinfrastructureharness")
    );
}

#[test]
fn running_over_the_page_says_by_how_far() {
    let report = judge("overflow", &Check::default());
    assert!(!report.ok);
    assert_eq!(report.pages, 2);
    let over = report.overflow_mm.expect("it ran over");
    assert!((30.0..36.0).contains(&over), "over by {over}mm");
}

#[test]
fn two_pages_are_fine_when_two_are_allowed() {
    let check = Check {
        max_pages: 2,
        ..Check::default()
    };
    let report = judge("overflow", &check);
    assert!(report.ok);
    assert_eq!(report.pages, 2);
    assert!(report.overflow_mm.is_none());
    assert_eq!(report.max_pages, 2);
}

#[test]
fn silent_is_visible_in_the_report() {
    let mut check = Check::default();
    check.loose_lines.severity = Severity::Silent;
    let report = judge("short-line", &check);
    assert!(report.ok);
    assert!(report.loose_lines.is_empty());
    assert_eq!(report.loose_line_severity, Severity::Silent);
    assert_eq!(report.settings.loose_lines.severity, Severity::Silent);
}

#[test]
fn error_severity_fails_the_run_on_the_same_finding() {
    let mut check = Check::default();
    check.loose_lines.severity = Severity::Error;
    let report = judge("short-line", &check);
    assert!(!report.ok);
    assert_eq!(report.loose_lines.len(), 1);
}

#[test]
fn every_rule_reads_its_number_from_the_settings() {
    let mut check = Check::default();
    check.loose_lines.short_line = 0.1;
    assert!(judge("short-line", &check).loose_lines.is_empty());

    let mut check = Check::default();
    check.loose_lines.widow_words = 0;
    let found = &judge("widow", &check).loose_lines[0];
    assert_eq!(
        found.kind,
        LooseKind::ShortLastLine,
        "with no widow rule the same line is judged on its fill"
    );
}

#[test]
fn the_report_echoes_what_it_measured_against() {
    let report = judge("clean", &Check::default());
    let json = serde_json::to_value(&report).expect("the report serializes");
    assert_eq!(json["settings"]["looseLines"]["shortLine"], 0.75);
    assert_eq!(json["settings"]["maxPages"], 1);
    assert_eq!(json["looseLineSeverity"], "warn");
}
