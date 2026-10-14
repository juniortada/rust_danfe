//! Typst compiled in: a template reads `data.json` and may embed generated
//! images (barcode, QR code, logo) given as extra files. No clock and no
//! network, so the same input always gives the same PDF.
//!
//! Anexo II §3.7 asks for Times New Roman or Courier New. The crate embeds
//! Liberation Serif and Liberation Mono (SIL OFL 1.1, metrically compatible
//! with them); the templates prefer Times New Roman and Courier New, so fonts
//! given in [`crate::Options::fonts`] win when they are those.

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

const EMBEDDED_FONTS: [&[u8]; 4] = [
    include_bytes!("../fonts/LiberationSerif-Regular.ttf"),
    include_bytes!("../fonts/LiberationSerif-Bold.ttf"),
    include_bytes!("../fonts/LiberationMono-Regular.ttf"),
    include_bytes!("../fonts/LiberationMono-Bold.ttf"),
];

static LIBRARY: LazyLock<LazyHash<Library>> = LazyLock::new(|| LazyHash::new(Library::default()));

static EMBEDDED: LazyLock<Vec<Font>> = LazyLock::new(|| {
    EMBEDDED_FONTS
        .iter()
        .flat_map(|data| Font::iter(Bytes::new(*data)))
        .collect()
});

fn file_id(path: &str) -> Result<FileId, Error> {
    let vpath = VirtualPath::new(path).map_err(|err| Error::Render(format!("{path}: {err:?}")))?;
    Ok(RootedPath::new(VirtualRoot::Project, vpath).intern())
}

struct TemplateWorld {
    main: FileId,
    source: Source,
    files: HashMap<FileId, Bytes>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
}

impl World for TemplateWorld {
    fn library(&self) -> &LazyHash<Library> {
        &LIBRARY
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
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
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<Duration>) -> Option<Datetime> {
        None
    }
}

fn joined<T>(errors: &[T], message: impl Fn(&T) -> String) -> Error {
    Error::Render(errors.iter().map(message).collect::<Vec<_>>().join("; "))
}

/// Lays out `template` with `data` as `data.json`, `files` (e.g.
/// `("/qrcode.svg", bytes)`) next to it and `extra_fonts` before the
/// embedded ones.
pub fn document(
    template: &str,
    data: &impl serde::Serialize,
    files: Vec<(String, Vec<u8>)>,
    extra_fonts: &[Vec<u8>],
) -> Result<PagedDocument, Error> {
    let main = file_id(MAIN)?;
    let mut world_files = HashMap::new();
    let json = serde_json::to_vec(data).map_err(|err| Error::Render(err.to_string()))?;
    world_files.insert(file_id("/data.json")?, Bytes::new(json));
    for (path, bytes) in files {
        world_files.insert(file_id(&path)?, Bytes::new(bytes));
    }
    let mut fonts: Vec<Font> = extra_fonts
        .iter()
        .flat_map(|data| Font::iter(Bytes::new(data.clone())))
        .collect();
    fonts.extend(EMBEDDED.iter().cloned());
    let world = TemplateWorld {
        main,
        source: Source::new(main, template.to_string()),
        files: world_files,
        book: LazyHash::new(FontBook::from_fonts(&fonts)),
        fonts,
    };
    typst::compile(&world)
        .output
        .map_err(|errors| joined(&errors, |error| error.message.to_string()))
}

/// The PDF of a laid out document.
pub fn pdf(document: &PagedDocument) -> Result<Vec<u8>, Error> {
    typst_pdf::pdf(document, &typst_pdf::PdfOptions::default())
        .map_err(|errors| joined(&errors, |error| error.message.to_string()))
}

/// The QR code as SVG, error correction level M and a four-module quiet
/// zone (Manual DANFE NFC-e §3.4 and §4.5).
pub fn qr_svg(content: &str) -> Option<Vec<u8>> {
    let code =
        qrcode::QrCode::with_error_correction_level(content.as_bytes(), qrcode::EcLevel::M).ok()?;
    Some(
        code.render::<qrcode::render::svg::Color<'_>>()
            .min_dimensions(200, 200)
            .quiet_zone(true)
            .build()
            .into_bytes(),
    )
}
