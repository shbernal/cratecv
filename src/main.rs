//! The command line. Everything it does is a call into the library; what lives
//! here is argument parsing, file handling, and deciding what to say.

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use clap::{Parser, Subcommand};
use cratecv::config::{Flags, Resolved};
use cratecv::{Error, Report, Resume};

/// Exit codes are part of the contract: callers branch on them. They track
/// severity, not which check spoke.
mod code {
    use std::process::ExitCode;

    /// Compiled, within the page limit, nothing at error severity.
    pub const OK: ExitCode = ExitCode::SUCCESS;
    /// A check failed.
    pub const FAILED: u8 = 1;
    /// The resume is invalid, or the arguments are.
    pub const INVALID: u8 = 2;
    /// A bug in this tool.
    pub const INTERNAL: u8 = 3;
}

#[derive(Parser)]
#[command(name = "cratecv", version = env!("CRATECV_VERSION"), about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Compile the resume to a PDF, refusing to run over the page limit.
    Build {
        input: PathBuf,
        /// A file, or a directory to name the file in.
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Write the PDF even when the page count is exceeded.
        #[arg(long)]
        allow_overflow: bool,
        #[arg(long)]
        max_pages: Option<usize>,
    },
    /// Report on the layout without writing anything.
    Check {
        input: PathBuf,
        /// Emit the report as JSON on stdout, and nothing else.
        #[arg(long)]
        json: bool,
        #[arg(long)]
        max_pages: Option<usize>,
    },
    /// Render what the page looks like, as PNG or, by extension, SVG.
    Preview {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        /// Resolution of the PNG, high enough to read by default.
        #[arg(long)]
        dpi: Option<f64>,
    },
    /// Recompile while the resume is being edited.
    Watch {
        input: PathBuf,
        #[arg(short, long)]
        output: Option<PathBuf>,
        #[arg(long)]
        max_pages: Option<usize>,
    },
    /// Print a JSON Schema for the resume format.
    Schema,
    /// Write a starter resume here, and a config file if there is none.
    Init {
        /// Where the starter resume goes. The current directory by default.
        directory: Option<PathBuf>,
    },
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(why) => fail(&why),
    }
}

fn run(cli: Cli) -> Result<ExitCode, Error> {
    match cli.command {
        Command::Build {
            input,
            output,
            allow_overflow,
            max_pages,
        } => build(
            &input,
            Flags {
                max_pages,
                output,
                ..Flags::default()
            },
            allow_overflow,
        ),
        Command::Check {
            input,
            json,
            max_pages,
        } => check(
            &input,
            json,
            Flags {
                max_pages,
                ..Flags::default()
            },
        ),
        Command::Preview { input, output, dpi } => preview(
            &input,
            Flags {
                dpi,
                output,
                ..Flags::default()
            },
        ),
        Command::Watch {
            input,
            output,
            max_pages,
        } => watch(
            &input,
            Flags {
                max_pages,
                output,
                ..Flags::default()
            },
        ),
        Command::Schema => schema(),
        Command::Init { directory } => init(directory.as_deref()),
    }
}

/// Read the resume and fold the four layers over it.
fn prepare(input: &Path, flags: Flags) -> Result<(String, Resume, Resolved), Error> {
    let yaml = std::fs::read_to_string(input)?;
    let resume = cratecv::load(&yaml).map_err(Error::Resume)?;
    let config = cratecv::config::load().map_err(Error::Resume)?;
    let settings = cratecv::config::resolve(&config, resume.cratecv.as_ref(), &flags)
        .map_err(Error::Resume)?;
    Ok((yaml, resume, settings))
}

fn build(input: &Path, flags: Flags, allow_overflow: bool) -> Result<ExitCode, Error> {
    let (yaml, resume, settings) = prepare(input, flags)?;
    let mut document = cratecv::compile_theme(&resume, &settings.theme)?;
    let measured = cratecv::layout::measure(&document)?;
    let report = cratecv::report::report(
        &measured,
        &cratecv::schema::Locator::new(&yaml),
        &settings.check,
    );

    eprint!("{}", cratecv::report::summary(&report, &shown(input)));

    // A page count over the limit is a defect, so nothing is written and a
    // previous good PDF at the same path survives. Loose lines are advice.
    if !report.fits() && !allow_overflow {
        eprintln!("Nothing written. Pass --allow-overflow to write it anyway.");
        return Ok(ExitCode::from(code::FAILED));
    }

    cratecv::describe(
        &mut document,
        &resume,
        &settings.pdf,
        settings.pdf.date.resolve(input),
    );
    let path = destination(&settings, &resume, "pdf");
    write(&path, &cratecv::export_pdf(&document, &settings.pdf)?)?;
    eprintln!("Wrote {}", path.display());
    Ok(exit_for(&report))
}

fn check(input: &Path, json: bool, flags: Flags) -> Result<ExitCode, Error> {
    let file = shown(input);
    let yaml = std::fs::read_to_string(input)?;
    let config = cratecv::config::load().map_err(Error::Resume)?;
    let settings = match cratecv::load(&yaml) {
        Ok(resume) => cratecv::config::resolve(&config, resume.cratecv.as_ref(), &flags),
        // The resume does not load, so its own layer has nothing to say. The
        // report still has to state what it would have measured against.
        Err(_) => cratecv::config::resolve(&config, None, &flags),
    }
    .map_err(Error::Resume)?;

    let report = cratecv::check(&yaml, &settings.check)?;

    if json {
        // Nothing but the report reaches stdout, so this stays pipeable.
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|why| Error::Export(why.to_string()))?
        );
    }

    if !report.schema_errors.is_empty() {
        if !json {
            eprintln!("{file} does not load:");
        }
        for found in &report.schema_errors {
            eprintln!("  {file}:{found}");
        }
        return Ok(ExitCode::from(code::INVALID));
    }
    if !json {
        eprint!("{}", cratecv::report::summary(&report, &file));
    }
    Ok(exit_for(&report))
}

fn preview(input: &Path, flags: Flags) -> Result<ExitCode, Error> {
    let (_, resume, settings) = prepare(input, flags)?;
    let document = cratecv::compile_theme(&resume, &settings.theme)?;

    let svg = settings
        .output
        .as_deref()
        .is_some_and(|path| path.extension().is_some_and(|ext| ext == "svg"));
    let path = destination(&settings, &resume, if svg { "svg" } else { "png" });
    let bytes = if svg {
        cratecv::export_svg(&document).into_bytes()
    } else {
        cratecv::export_png(&document, settings.dpi)?
    };
    write(&path, &bytes)?;
    eprintln!("Wrote {}", path.display());
    Ok(code::OK)
}

/// Editors write in bursts, so a change is acted on once the file has been
/// quiet for this long.
const DEBOUNCE: Duration = Duration::from_millis(200);

fn watch(input: &Path, flags: Flags) -> Result<ExitCode, Error> {
    use notify::{RecursiveMode, Watcher};

    let input = input.canonicalize()?;
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        let _ = tx.send(event);
    })
    .map_err(watch_failed)?;

    // Editors replace rather than rewrite, so the directory is what to watch.
    let directory = input.parent().unwrap_or(Path::new(".")).to_owned();
    watcher
        .watch(&directory, RecursiveMode::NonRecursive)
        .map_err(watch_failed)?;

    eprintln!("Watching {}. Ctrl-C to stop.", input.display());
    let _ = build(&input, flags.clone(), true);

    while let Ok(event) = rx.recv() {
        let touched = event
            .map(|event: notify::Event| event.paths.contains(&input))
            .unwrap_or(false);
        if !touched {
            continue;
        }
        while rx.recv_timeout(DEBOUNCE).is_ok() {}
        eprintln!("---");
        if let Err(why) = build(&input, flags.clone(), true) {
            report_error(&why);
        }
    }
    Ok(code::OK)
}

fn schema() -> Result<ExitCode, Error> {
    println!(
        "{}",
        serde_json::to_string_pretty(&cratecv::json_schema())
            .map_err(|why| Error::Export(why.to_string()))?
    );
    Ok(code::OK)
}

fn init(directory: Option<&Path>) -> Result<ExitCode, Error> {
    let directory = directory.unwrap_or(Path::new("."));
    let resume = directory.join("resume.yaml");
    if resume.exists() {
        eprintln!("{} is already there, left alone.", resume.display());
    } else {
        write(&resume, cratecv::STARTER_RESUME.as_bytes())?;
        eprintln!("Wrote {}", resume.display());
    }

    let config = cratecv::config::path();
    if config.exists() {
        eprintln!("{} is already there, left alone.", config.display());
    } else {
        write(&config, cratecv::config::STARTER.as_bytes())?;
        eprintln!("Wrote {}", config.display());
    }
    Ok(code::OK)
}

fn watch_failed(why: notify::Error) -> Error {
    Error::Io(std::io::Error::other(why))
}

/// `-o` takes either a file or a directory. A directory receives the output
/// under a name derived from the resume, which is what a separate name flag
/// would otherwise be for.
fn destination(settings: &Resolved, resume: &Resume, extension: &str) -> PathBuf {
    let named = |dir: &Path| dir.join(format!("{}.{extension}", slug(&resume.name)));
    match settings.output.as_deref() {
        None => named(Path::new(".")),
        Some(path) if path.is_dir() => named(path),
        Some(path) if path.to_string_lossy().ends_with(std::path::MAIN_SEPARATOR) => named(path),
        Some(path) => path.to_owned(),
    }
}

fn slug(name: &str) -> String {
    let mut out = String::new();
    for character in name.chars() {
        if character.is_alphanumeric() {
            out.extend(character.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let trimmed = out.trim_matches('-');
    if trimmed.is_empty() {
        "resume".to_owned()
    } else {
        trimmed.to_owned()
    }
}

fn write(path: &Path, bytes: &[u8]) -> Result<(), Error> {
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, bytes)?;
    Ok(())
}

fn shown(input: &Path) -> String {
    input.display().to_string()
}

fn exit_for(report: &Report) -> ExitCode {
    if report.ok {
        code::OK
    } else {
        ExitCode::from(code::FAILED)
    }
}

fn fail(why: &Error) -> ExitCode {
    report_error(why);
    match why {
        Error::Resume(_) | Error::Io(_) => ExitCode::from(code::INVALID),
        Error::Template(_) | Error::Export(_) => ExitCode::from(code::INTERNAL),
    }
}

fn report_error(why: &Error) {
    eprintln!("{why}");
    for found in why.diagnostics() {
        eprintln!("  {found}");
    }
    if matches!(why, Error::Template(_)) {
        eprintln!("This is a bug in cratecv, not in your resume.");
    }
}
