//! Exit codes are the contract callers branch on, so they are asserted here.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_cratecv");

fn workspace(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("a scratch directory");
    dir
}

fn run(args: &[&str]) -> Output {
    Command::new(BIN).args(args).output().expect("cratecv runs")
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("it exited on its own")
}

#[test]
fn a_clean_resume_builds_and_exits_zero() {
    let dir = workspace("clean-build");
    let out = run(&[
        "build",
        "tests/fixtures/clean.yaml",
        "-o",
        dir.to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(
        dir.join("clean-resume.pdf").exists(),
        "named from the resume"
    );
}

#[test]
fn advice_at_warn_exits_zero_and_still_writes() {
    let dir = workspace("warn-build");
    let pdf = dir.join("out.pdf");
    let out = run(&[
        "build",
        "tests/fixtures/short-line.yaml",
        "-o",
        pdf.to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 0);
    assert!(pdf.exists());
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("short line"), "{said}");
}

#[test]
fn running_over_the_page_limit_exits_one_and_writes_nothing() {
    let dir = workspace("overflow-build");
    let pdf = dir.join("out.pdf");
    std::fs::write(&pdf, b"a previous good build").unwrap();

    let out = run(&[
        "build",
        "tests/fixtures/overflow.yaml",
        "-o",
        pdf.to_str().unwrap(),
    ]);
    assert_eq!(code(&out), 1);
    assert_eq!(
        std::fs::read(&pdf).unwrap(),
        b"a previous good build",
        "the last good PDF must survive a bad run"
    );
}

#[test]
fn allow_overflow_writes_but_still_reports_the_failure() {
    let dir = workspace("overflow-allowed");
    let pdf = dir.join("out.pdf");
    let out = run(&[
        "build",
        "tests/fixtures/overflow.yaml",
        "-o",
        pdf.to_str().unwrap(),
        "--allow-overflow",
    ]);
    assert_eq!(code(&out), 1, "the flag decides the write, not the verdict");
    assert!(pdf.metadata().unwrap().len() > 1000);
}

#[test]
fn raising_the_page_limit_makes_the_same_resume_pass() {
    let dir = workspace("overflow-allowed-pages");
    let out = run(&[
        "build",
        "tests/fixtures/overflow.yaml",
        "-o",
        dir.to_str().unwrap(),
        "--max-pages",
        "2",
    ]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert!(dir.join("overflowing-resume.pdf").exists());
}

#[test]
fn json_output_is_the_only_thing_on_stdout() {
    let out = run(&["check", "examples/resume.yaml", "--json"]);
    assert_eq!(code(&out), 0);
    let report: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout is JSON and nothing else");
    assert_eq!(report["pages"], 1);
    assert_eq!(report["ok"], true);
    assert!(!report["looseLines"].as_array().unwrap().is_empty());
}

#[test]
fn an_invalid_resume_exits_two_and_says_where() {
    let dir = workspace("invalid");
    let yaml = dir.join("broken.yaml");
    std::fs::write(&yaml, "name: Ada\ncontact: {}\nsections: []\n").unwrap();

    let out = run(&["check", yaml.to_str().unwrap()]);
    assert_eq!(code(&out), 2);
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("sections"), "{said}");
    assert!(said.contains(":3:"), "{said}");
}

#[test]
fn a_broken_resume_reports_as_json_too() {
    let dir = workspace("invalid-json");
    let yaml = dir.join("broken.yaml");
    std::fs::write(&yaml, "name: Ada\ncontact: {}\nsections: []\n").unwrap();

    let out = run(&["check", yaml.to_str().unwrap(), "--json"]);
    assert_eq!(code(&out), 2);
    let report: serde_json::Value = serde_json::from_slice(&out.stdout).expect("still JSON");
    assert_eq!(report["ok"], false);
    assert_eq!(report["schemaErrors"][0]["path"], "sections");
}

#[test]
fn bad_arguments_exit_two() {
    assert_eq!(code(&run(&["polish", "examples/resume.yaml"])), 2);
    assert_eq!(code(&run(&["build"])), 2);
}

#[test]
fn preview_writes_a_png_and_an_svg_by_extension() {
    let dir = workspace("preview");
    let png = dir.join("page.png");
    assert_eq!(
        code(&run(&[
            "preview",
            "tests/fixtures/clean.yaml",
            "-o",
            png.to_str().unwrap()
        ])),
        0
    );
    assert!(std::fs::read(&png).unwrap().starts_with(b"\x89PNG"));

    let svg = dir.join("page.svg");
    assert_eq!(
        code(&run(&[
            "preview",
            "tests/fixtures/clean.yaml",
            "-o",
            svg.to_str().unwrap()
        ])),
        0
    );
    assert!(std::fs::read_to_string(&svg).unwrap().starts_with("<svg"));
}
