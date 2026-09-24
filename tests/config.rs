//! Four layers, most specific winning, and the metadata pass that follows.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const BIN: &str = env!("CARGO_BIN_EXE_cratecv");

struct Machine {
    home: PathBuf,
}

impl Machine {
    /// A scratch machine with its own config directory, so a test never reads
    /// or writes the one belonging to whoever is running it.
    fn new(name: &str) -> Self {
        let home = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(home.join("cratecv")).expect("a scratch config directory");
        Self { home }
    }

    fn with_config(self, yaml: &str) -> Self {
        std::fs::write(self.home.join("cratecv/config.yaml"), yaml).expect("write the config");
        self
    }

    fn file(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.home.join(name);
        std::fs::write(&path, contents).expect("write a fixture");
        path
    }

    fn run(&self, args: &[&str]) -> Output {
        Command::new(BIN)
            .args(args)
            .env("XDG_CONFIG_HOME", &self.home)
            .output()
            .expect("cratecv runs")
    }
}

fn code(output: &Output) -> i32 {
    output.status.code().expect("it exited on its own")
}

fn json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout is the report")
}

fn overflowing(settings: &str) -> String {
    let body = std::fs::read_to_string("tests/fixtures/overflow.yaml").unwrap();
    format!("{settings}{body}")
}

#[test]
fn a_typo_in_the_config_reads_like_a_typo_in_a_resume() {
    let machine = Machine::new("config-typo").with_config("check:\n  maxpages: 2\n");
    let out = machine.run(&["check", "examples/resume.yaml"]);
    assert_eq!(code(&out), 2);
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("maxpages"), "{said}");
    assert!(said.contains("2:3"), "with a line and column: {said}");
}

#[test]
fn the_config_is_optional() {
    let machine = Machine::new("config-absent");
    let out = machine.run(&["check", "examples/resume.yaml", "--json"]);
    assert_eq!(code(&out), 0);
    assert_eq!(json(&out)["settings"]["maxPages"], 1);
}

#[test]
fn a_resume_overrides_the_config_and_a_flag_overrides_the_resume() {
    let machine = Machine::new("layers").with_config("check:\n  maxPages: 1\n");
    let resume = machine.file(
        "two-pages.yaml",
        &overflowing("cratecv:\n  check:\n    maxPages: 2\n"),
    );
    let path = resume.to_str().unwrap();

    let out = machine.run(&["check", path, "--json"]);
    assert_eq!(code(&out), 0, "the resume says two pages are fine");
    assert_eq!(json(&out)["pages"], 2);
    assert_eq!(json(&out)["settings"]["maxPages"], 2);

    let out = machine.run(&["check", path, "--json", "--max-pages", "1"]);
    assert_eq!(code(&out), 1, "the flag is the most specific layer");
    assert_eq!(json(&out)["settings"]["maxPages"], 1);
}

#[test]
fn the_config_alone_can_raise_the_page_limit() {
    let machine = Machine::new("config-pages").with_config("check:\n  maxPages: 2\n");
    let resume = machine.file("two-pages.yaml", &overflowing(""));
    let out = machine.run(&["check", resume.to_str().unwrap(), "--json"]);
    assert_eq!(code(&out), 0);
    assert_eq!(json(&out)["pages"], 2);
}

#[test]
fn silent_says_it_was_silent_rather_than_saying_nothing() {
    let machine =
        Machine::new("silent").with_config("check:\n  looseLines:\n    severity: silent\n");
    let out = machine.run(&["check", "tests/fixtures/short-line.yaml", "--json"]);
    assert_eq!(code(&out), 0);
    let report = json(&out);
    assert_eq!(report["looseLines"].as_array().unwrap().len(), 0);
    assert_eq!(report["looseLineSeverity"], "silent");
    assert_eq!(report["settings"]["looseLines"]["severity"], "silent");
}

#[test]
fn error_severity_turns_the_same_advice_into_a_failure() {
    let machine =
        Machine::new("severity-error").with_config("check:\n  looseLines:\n    severity: error\n");
    let out = machine.run(&["check", "tests/fixtures/short-line.yaml", "--json"]);
    assert_eq!(code(&out), 1);
    assert_eq!(json(&out)["ok"], false);
    assert_eq!(json(&out)["looseLines"].as_array().unwrap().len(), 1);
}

#[test]
fn a_threshold_can_be_moved_without_silencing_the_check() {
    let machine =
        Machine::new("threshold").with_config("check:\n  looseLines:\n    shortLine: 0.1\n");
    let out = machine.run(&["check", "tests/fixtures/short-line.yaml", "--json"]);
    assert_eq!(code(&out), 0);
    let report = json(&out);
    assert_eq!(report["looseLines"].as_array().unwrap().len(), 0);
    assert_eq!(
        report["looseLineSeverity"], "warn",
        "still running, just quiet"
    );
    assert_eq!(report["settings"]["looseLines"]["shortLine"], 0.1);
}

#[test]
fn a_date_that_is_not_a_date_says_what_it_wanted() {
    let machine = Machine::new("bad-date").with_config("pdf:\n  date: last tuesday\n");
    let out = machine.run(&["check", "examples/resume.yaml"]);
    assert_eq!(code(&out), 2);
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("mtime"), "{said}");
}

#[test]
fn the_configured_origin_reaches_the_info_dictionary_and_the_xmp() {
    let machine = Machine::new("metadata").with_config(
        "pdf:\n  creator: Some Word Processor\n  producer: Some Word Processor\n  \
         date: \"2024-03-11T09:12:00Z\"\n  keywords: [resume]\n",
    );
    let pdf = machine.home.join("out.pdf");
    let out = machine.run(&["build", "examples/resume.yaml", "-o", pdf.to_str().unwrap()]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));

    let bytes = std::fs::read(&pdf).unwrap();
    let text = String::from_utf8_lossy(&bytes);
    assert!(
        text.contains("/Producer(Some Word Processor)")
            || text.contains("/Producer (Some Word Processor)"),
        "the Info dictionary names the producer"
    );
    assert!(
        text.contains("<pdf:Producer>Some Word Processor</pdf:Producer>"),
        "and so does the XMP packet"
    );
    assert!(
        text.contains("<xmp:CreatorTool>Some Word Processor</xmp:CreatorTool>"),
        "and the creator with it"
    );
    assert!(
        text.contains("D:20240311091200Z"),
        "the fixed date is written"
    );
    assert!(
        text.contains("(Mira Halvorsen)"),
        "title and author come from the resume"
    );
}

#[test]
fn two_builds_of_one_resume_are_byte_identical() {
    let machine = Machine::new("reproducible");
    let first = machine.home.join("first.pdf");
    let second = machine.home.join("second.pdf");
    for path in [&first, &second] {
        let out = machine.run(&[
            "build",
            "examples/resume.yaml",
            "-o",
            path.to_str().unwrap(),
        ]);
        assert_eq!(code(&out), 0);
    }
    assert_eq!(
        std::fs::read(&first).unwrap(),
        std::fs::read(&second).unwrap(),
        "the same input must always produce the same file"
    );
}

#[test]
fn init_writes_a_starter_and_never_overwrites() {
    let machine = Machine::new("init");
    let dir = machine.home.join("work");
    std::fs::create_dir_all(&dir).unwrap();

    let out = machine.run(&["init", dir.to_str().unwrap()]);
    assert_eq!(code(&out), 0);
    let resume = dir.join("resume.yaml");
    let config = machine.home.join("cratecv/config.yaml");
    assert!(resume.exists() && config.exists());

    // What it writes has to be valid, or the first thing a new user does fails.
    let out = machine.run(&["check", resume.to_str().unwrap(), "--json"]);
    assert_eq!(code(&out), 0, "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(json(&out)["pages"], 1);

    std::fs::write(&resume, "mine").unwrap();
    assert_eq!(code(&machine.run(&["init", dir.to_str().unwrap()])), 0);
    assert_eq!(std::fs::read_to_string(&resume).unwrap(), "mine");
}

#[test]
fn a_resume_may_not_state_settings_about_this_machine() {
    let machine = Machine::new("resume-scope");
    let resume = machine.file(
        "scoped.yaml",
        &overflowing("cratecv:\n  preview:\n    dpi: 300\n"),
    );
    let out = machine.run(&["check", resume.to_str().unwrap()]);
    assert_eq!(code(&out), 2);
    let said = String::from_utf8_lossy(&out.stderr);
    assert!(said.contains("preview"), "{said}");
}
