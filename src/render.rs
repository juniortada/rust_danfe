//! Typst compiled in: a template reads `data.json` and may embed generated
//! images (barcode, QR code) given as extra files. No clock and no network,
//! so the same input always gives the same PDF.

use std::{collections::HashMap, sync::LazyLock};

use typst::{
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime, Duration},
    syntax::{FileId, RootedPath, Source, VirtualPath, VirtualRoot},
    text::{Font, FontBook},
    utils::LazyHash,
    Library, LibraryExt, World,
};
use typst_layout::PagedDocument;

use crate::Error;

const MAIN: &str = "/main.typ";

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

static FONTS: LazyLock<(LazyHash<FontBook>, Vec<Font>)> = LazyLock::new(|| {
    let fonts: Vec<Font> = typst_assets::fonts()
        .flat_map(|data| Font::iter(Bytes::new(data)))
        .collect();
    (LazyHash::new(FontBook::from_fonts(&fonts)), fonts)
});

fn file_id(path: &str) -> Result<FileId, Error> {
    let vpath = VirtualPath::new(path).map_err(|err| Error::Render(format!("{path}: {err:?}")))?;
    Ok(RootedPath::new(VirtualRoot::Project, vpath).intern())
}

struct TemplateWorld {
    main: FileId,
    source: Source,
    files: HashMap<FileId, Bytes>,
}

impl World for TemplateWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &FONTS.0
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.main {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(std::path::PathBuf::from("source")))
        }
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        self.files
            .get(&id)
            .cloned()
            .ok_or_else(|| FileError::NotFound(std::path::PathBuf::from("file")))
    }

    fn font(&self, index: usize) -> Option<Font> {
        FONTS.1.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn joined<T>(errors: &[T], message: impl Fn(&T) -> String) -> Error {
    Error::Render(errors.iter().map(message).collect::<Vec<_>>().join("; "))
}

/// Compiles `template` with `data` as `data.json` and `files` (e.g.
/// `("/qrcode.svg", bytes)`) next to it.
pub fn compile(
    template: &str,
    data: &impl serde::Serialize,
    files: Vec<(String, Vec<u8>)>,
) -> Result<Vec<u8>, Error> {
    let main = file_id(MAIN)?;
    let mut world_files = HashMap::new();
    let json = serde_json::to_vec(data).map_err(|err| Error::Render(err.to_string()))?;
    world_files.insert(file_id("/data.json")?, Bytes::new(json));
    for (path, bytes) in files {
        world_files.insert(file_id(&path)?, Bytes::new(bytes));
    }
    let world = TemplateWorld {
        main,
        source: Source::new(main, template.to_string()),
        files: world_files,
    };
    let document: PagedDocument = typst::compile(&world)
        .output
        .map_err(|errors| joined(&errors, |error| error.message.to_string()))?;
    typst_pdf::pdf(&document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| joined(&errors, |error| error.message.to_string()))
}

/// The QR code as SVG, for templates to `image(...)`.
pub fn qr_svg(content: &str) -> Option<Vec<u8>> {
    let code = qrcode::QrCode::new(content.as_bytes()).ok()?;
    Some(
        code.render::<qrcode::render::svg::Color<'_>>()
            .min_dimensions(200, 200)
            .quiet_zone(false)
            .build()
            .into_bytes(),
    )
}
