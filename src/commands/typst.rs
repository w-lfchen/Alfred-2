use anyhow::Context;
use time::{PrimitiveDateTime, UtcDateTime};
use typst::{
    Library, LibraryExt,
    diag::{FileResult, Severity, SourceDiagnostic, Warned},
    ecow::EcoVec,
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::FontStore;
use typst_layout::PagedDocument;

struct World {
    library: LazyHash<Library>,
    fonts: FontStore,
    source: Source,
    date_time: Datetime,
}

impl World {
    fn new(document: String) -> Self {
        let mut y = FontStore::default();
        y.extend(typst_kit::fonts::embedded());
        /*
        use std::{fs::File, io::Read, path::Path};
        let mut c = String::new();
        let mut f = File::open(Path::new("test.typ")).unwrap();
        f.read_to_string(&mut c).unwrap();
        */
        let now = UtcDateTime::now();
        Self {
            library: LazyHash::new(Library::default()),
            fonts: y,
            source: Source::new(
                FileId::unique(RootedPath::new(
                    VirtualRoot::Project,
                    VirtualPath::new("<empty>").unwrap(),
                )),
                document,
            ),
            date_time: Datetime::Datetime(PrimitiveDateTime::new(now.date(), now.time())),
        }
    }
}

impl typst::World for World {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        self.fonts.book()
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            FileResult::Ok(self.source.clone())
        } else {
            FileResult::Err(typst::diag::FileError::AccessDenied)
        }
    }

    fn file(&self, _: FileId) -> FileResult<Bytes> {
        FileResult::Err(typst::diag::FileError::AccessDenied)
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.font(index)
    }

    fn today(&self, offset: Option<Duration>) -> Option<Datetime> {
        Some(match offset {
            Some(offset) => self.date_time + offset,
            None => self.date_time,
        })
    }
}

// render the text into a png
// currently, only the first page is returned
// TODO: change default page size
pub(super) fn render_png(document: String) -> Result<(Option<Vec<u8>>, String), anyhow::Error> {
    let world = World::new(document);
    let Warned { output, warnings } = typst::compile::<PagedDocument>(&world);
    Ok(match output {
        Ok(document) => {
            let first_page = document.pages().first().context("no first page found")?;
            let png = typst_render::render(first_page, 2.0)
                // .save_png(Path::new("a.png"))
                .encode_png()?;
            (Some(png), format_diagnostics(warnings))
        }
        Err(errors) => (None, format_diagnostics(errors)),
    })
}

/// basic formatting: new line for each warning
fn format_diagnostics(diags: EcoVec<SourceDiagnostic>) -> String {
    diags.iter().fold(String::new(), |acc, s| {
        format!(
            "{acc}\n[{}] {}",
            match s.severity {
                Severity::Error => "error",
                Severity::Warning => "warning",
            },
            &s.message,
        )
    })
}
