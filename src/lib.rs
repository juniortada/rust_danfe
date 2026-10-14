//! DANFE (NF-e, modelo 55) and DANFE NFC-e (modelo 65) as PDF, from the XML
//! alone, following the MOC 7.0 Anexo II, the Manual de Especificações
//! Técnicas do DANFE NFC-e e QR Code (v6.0) and NT 2026.010.
//!
//! ```no_run
//! let xml = std::fs::read_to_string("nota.xml")?;
//! let pdf = danfe::render_default(&xml)?;
//! std::fs::write("nota.pdf", pdf)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{fmt, path::Path};

use serde::Serialize;

mod barcode;
pub mod format;
mod model;
mod render;
mod xml;

pub use model::{
    Address, Carrier, Duplicate, Emitter, Invoice, Issqn, Item, ItemIbsCbs, ItemTax, MonoTotals,
    Payment, Recipient, ReformTotals, Totals, Transport, Vehicle, Volume,
};

const NFE_A4_PORTRAIT: &str = include_str!("../templates/nfe_a4_portrait.typ");
const NFCE: &str = include_str!("../templates/nfce.typ");

/// Paper and orientation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layout {
    A4Portrait,
    A4Landscape,
    Roll80mm,
    Roll58mm,
}

impl fmt::Display for Layout {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::A4Portrait => "A4 retrato",
            Self::A4Landscape => "A4 paisagem",
            Self::Roll80mm => "bobina 80 mm",
            Self::Roll58mm => "bobina 58 mm",
        })
    }
}

/// What the caller adds to the XML.
#[derive(Debug, Clone, Default)]
pub struct Options {
    /// Extra TTF/OTF files. The templates prefer "Times New Roman" (DANFE)
    /// and "Courier New" (DANFE NFC-e) to the embedded Liberation fonts, so
    /// passing those fonts makes the PDF use them (Anexo II §3.7).
    pub fonts: Vec<Vec<u8>>,
    /// The emitter's logo, PNG, JPEG or SVG (Anexo II §3.1.3; Manual DANFE
    /// NFC-e §3.1.1).
    pub logo: Option<Vec<u8>>,
    /// Prints "Via do Estabelecimento": the second copy of a DANFE NFC-e
    /// issued in off-line contingency (Manual DANFE NFC-e §3.1.8).
    pub establishment_copy: bool,
}

/// Why no PDF came out.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    #[error("XML do documento fiscal: {0}")]
    InvalidXml(String),
    #[error("XML do documento fiscal sem protocolo de autorização")]
    NotAuthorized,
    #[error("documento sem DANFE suportado: {0}")]
    UnsupportedDocument(String),
    #[error("o {document} não é impresso em {layout}")]
    UnsupportedLayout {
        document: &'static str,
        layout: Layout,
    },
    #[error("logotipo em formato não suportado: use PNG, JPEG ou SVG")]
    UnsupportedLogo,
    #[error("falha ao gerar o PDF: {0}")]
    Render(String),
}

/// What the templates read: the invoice and what the caller added.
#[derive(Serialize)]
struct Page<'a> {
    #[serde(flatten)]
    invoice: &'a Invoice,
    paper: &'static str,
    logo: Option<&'static str>,
    establishment_copy: bool,
}

/// What the DANFE prints, without rendering it.
///
/// # Errors
/// See [`Error`].
pub fn parse(xml: &str) -> Result<Invoice, Error> {
    xml::parse(xml)
}

/// The layout a document prints in when nobody chooses one.
fn default_layout(invoice: &Invoice) -> Layout {
    if invoice.model == "65" {
        Layout::Roll80mm
    } else {
        Layout::A4Portrait
    }
}

/// The PDF in the layout the document calls for.
///
/// # Errors
/// See [`Error`].
pub fn render_default(xml: &str) -> Result<Vec<u8>, Error> {
    let invoice = parse(xml)?;
    let layout = default_layout(&invoice);
    render_invoice(&invoice, layout, &Options::default())
}

/// The PDF in `layout`.
///
/// # Errors
/// [`Error::UnsupportedLayout`] when the document is not printed in that
/// layout; see [`Error`] for the rest.
pub fn render(xml: &str, layout: Layout) -> Result<Vec<u8>, Error> {
    render_with(xml, layout, &Options::default())
}

/// [`render`] with fonts, logo or the establishment copy.
///
/// # Errors
/// As [`render`], or [`Error::UnsupportedLogo`].
pub fn render_with(xml: &str, layout: Layout, options: &Options) -> Result<Vec<u8>, Error> {
    render_invoice(&parse(xml)?, layout, options)
}

/// Reads `xml_path` and writes the PDF to `pdf_path`.
///
/// # Errors
/// As [`render`], or [`Error::Render`] when a file cannot be read or written.
pub fn generate(
    xml_path: impl AsRef<Path>,
    pdf_path: impl AsRef<Path>,
    layout: Layout,
) -> Result<(), Error> {
    let xml = std::fs::read_to_string(xml_path).map_err(|err| Error::Render(err.to_string()))?;
    let pdf = render(&xml, layout)?;
    std::fs::write(pdf_path, pdf).map_err(|err| Error::Render(err.to_string()))
}

/// Where the template finds the logo, by its format.
fn logo_path(logo: &[u8]) -> Result<&'static str, Error> {
    let head = &logo[..logo.len().min(256)];
    if logo.starts_with(b"\x89PNG\r\n\x1a\n") {
        Ok("/logo.png")
    } else if logo.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Ok("/logo.jpg")
    } else if String::from_utf8_lossy(head).contains("<svg") {
        Ok("/logo.svg")
    } else {
        Err(Error::UnsupportedLogo)
    }
}

fn render_invoice(invoice: &Invoice, layout: Layout, options: &Options) -> Result<Vec<u8>, Error> {
    render::pdf(&lay_out(invoice, layout, options)?)
}

fn lay_out(
    invoice: &Invoice,
    layout: Layout,
    options: &Options,
) -> Result<typst_layout::PagedDocument, Error> {
    let (template, paper, image) = match (invoice.model.as_str(), layout) {
        ("55", Layout::A4Portrait) => {
            let modules = barcode::code128_modules(&invoice.key)
                .ok_or_else(|| Error::InvalidXml("chave de acesso inválida".to_string()))?;
            (
                NFE_A4_PORTRAIT,
                "a4",
                ("/chave.svg".to_string(), barcode::svg(&modules, 50)),
            )
        }
        ("65", Layout::Roll80mm) => {
            let qr = invoice
                .qrcode
                .as_deref()
                .and_then(render::qr_svg)
                .ok_or_else(|| Error::InvalidXml("NFC-e sem QR code".to_string()))?;
            (NFCE, "80mm", ("/qrcode.svg".to_string(), qr))
        }
        ("65", _) => {
            return Err(Error::UnsupportedLayout {
                document: "DANFE NFC-e",
                layout,
            })
        }
        ("55", _) => {
            return Err(Error::UnsupportedLayout {
                document: "DANFE",
                layout,
            })
        }
        (model, _) => return Err(Error::UnsupportedDocument(format!("modelo {model}"))),
    };
    let mut files = vec![image];
    let logo = match &options.logo {
        Some(bytes) => {
            let path = logo_path(bytes)?;
            files.push((path.to_string(), bytes.clone()));
            Some(path)
        }
        None => None,
    };
    let page = Page {
        invoice,
        paper,
        logo,
        establishment_copy: options.establishment_copy,
    };
    render::document(template, &page, files, &options.fonts)
}

#[cfg(test)]
mod tests;
