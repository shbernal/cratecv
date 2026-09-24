//! The Typst compiler, embedded, over a file tree that exists only in memory.
//!
//! Nothing here touches the filesystem. The template is compiled into the
//! binary, the fonts are compiled into the binary, and the resume reaching the
//! template is re-serialized from the validated Rust types rather than passed
//! through as the user's bytes, so a theme can only ever see data that already
//! satisfied the schema. It is served as JSON: it is the same `json()` call
//! shape in the template as `yaml()`, and it puts the escaping in serde's hands
//! rather than in a hand-rolled emitter's.
//!
//! The seam for a user template directory under `~/.config/cratecv/templates/`
//! is the file map: another entry, resolved before the built-in one. It is not
//! built.

mod fonts;

use std::collections::HashMap;

use typst::LibraryExt;
use typst::diag::{FileError, FileResult, SourceDiagnostic, Warned};
use typst::syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot};
use typst::utils::LazyHash;
use typst_layout::PagedDocument;
use typst_library::foundations::{Bytes, Datetime, Dict, Duration};
use typst_library::text::{Font, FontBook};
use typst_library::{Library, World, WorldExt};

pub use fonts::FAMILY;

use crate::diagnostic::Diagnostic;
use crate::error::Error;
use crate::schema::Resume;

/// Where the template reads the resume from.
const DATA: &str = "/resume.json";
/// The template Typst starts at.
const MAIN: &str = "/main.typ";

const DEFAULT_TEMPLATE: &str = include_str!("../../templates/default.typ");

/// Resolve a built-in theme name to its source.
fn theme_source(name: &str) -> Option<&'static str> {
    match name {
        "default" => Some(DEFAULT_TEMPLATE),
        _ => None,
    }
}

pub struct ResumeWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    main: FileId,
    sources: HashMap<FileId, Source>,
    files: HashMap<FileId, Bytes>,
}

impl ResumeWorld {
    /// Build a world serving `resume` to the named theme, with `inputs`
    /// reaching the template as `sys.inputs`.
    pub fn new(resume: &Resume, theme: &str, inputs: Dict) -> Result<Self, Error> {
        let template = theme_source(theme).ok_or_else(|| {
            Error::Resume(vec![Diagnostic {
                path: "cratecv.theme".into(),
                line: 1,
                column: 1,
                message: format!("unknown theme `{theme}`"),
            }])
        })?;

        let data = serde_json::to_vec(resume)
            .map_err(|why| Error::Export(format!("could not serialize the resume: {why}")))?;

        let main = file_id(MAIN);
        let mut sources = HashMap::new();
        sources.insert(main, Source::new(main, template.to_owned()));

        let mut files = HashMap::new();
        files.insert(file_id(DATA), Bytes::new(data));

        let (fonts, book) = fonts::embedded();
        Ok(Self {
            library: LazyHash::new(Library::builder().with_inputs(inputs).build()),
            book: LazyHash::new(book),
            fonts,
            main,
            sources,
            files,
        })
    }

    /// Lay the resume out. Warnings are dropped: the template is ours, so a
    /// warning from it is a bug to fix rather than news for the caller.
    pub fn compile(&self) -> Result<PagedDocument, Error> {
        let Warned {
            output,
            warnings: _,
        } = typst::compile::<PagedDocument>(self);
        output.map_err(|errors| Error::Template(self.describe(errors.as_slice())))
    }

    fn describe(&self, errors: &[SourceDiagnostic]) -> Vec<Diagnostic> {
        errors
            .iter()
            .map(|error| {
                let span = error.span;
                let located = span
                    .id()
                    .zip(self.range(span))
                    .and_then(|(id, range)| Some((id, self.source(id).ok()?, range)))
                    .map(|(id, source, range)| {
                        let (line, column) = source
                            .lines()
                            .byte_to_line_column(range.start)
                            .unwrap_or((0, 0));
                        (id.vpath().get_with_slash().to_owned(), line + 1, column + 1)
                    });
                let (path, line, column) = located.unwrap_or_else(|| (String::new(), 1, 1));
                Diagnostic {
                    path,
                    line,
                    column,
                    message: format!("{} (in the built-in template)", error.message),
                }
            })
            .collect()
    }
}

/// Where the template reads the resume from. Exposed so a test can read back
/// exactly what a theme is given.
pub fn data_id() -> FileId {
    file_id(DATA)
}

fn file_id(path: &str) -> FileId {
    let vpath = VirtualPath::new(path).expect("virtual paths here are literals");
    RootedPath::new(VirtualRoot::Project, vpath).intern()
}

impl World for ResumeWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        self.sources
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(id.vpath().get_with_slash().into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        if let Some(bytes) = self.files.get(&id) {
            return Ok(bytes.clone());
        }
        self.source(id)
            .map(|source| Bytes::from_string(source.text().to_owned()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    /// Always `None`. Nothing in a resume depends on the day it was built, and
    /// a compiler that cannot read the clock cannot produce two different PDFs
    /// from one input.
    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

/// Keeps `comemo`'s incremental cache from growing across compilations.
pub fn evict_cache() {
    comemo::evict(0);
}
