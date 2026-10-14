//! DANFE (NF-e, modelo 55) and DANFE NFC-e (modelo 65) as PDF, from the
//! authorized XML alone.
//!
//! ```no_run
//! let xml = std::fs::read_to_string("nota.xml")?;
//! let pdf = danfe::render_default(&xml)?;
//! std::fs::write("nota.pdf", pdf)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use std::{fmt, path::Path};

mod barcode;
pub mod format;
mod model;
mod render;
mod xml;

pub use model::{Address, Duplicate, Emitter, Invoice, Item, Payment, Recipient, Totals};

const DANFE_A4: &str = include_str!("../templates/danfe.typ");
const NFCE_80MM: &str = include_str!("../templates/danfce.typ");

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
    #[error("falha ao gerar o PDF: {0}")]
    Render(String),
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
    render_invoice(&invoice, layout)
}

/// The PDF in `layout`.
///
/// # Errors
/// [`Error::UnsupportedLayout`] when the document is not printed in that
/// layout; see [`Error`] for the rest.
pub fn render(xml: &str, layout: Layout) -> Result<Vec<u8>, Error> {
    render_invoice(&parse(xml)?, layout)
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

fn render_invoice(invoice: &Invoice, layout: Layout) -> Result<Vec<u8>, Error> {
    match (invoice.model.as_str(), layout) {
        ("65", Layout::Roll80mm) => {
            let qr = invoice
                .qrcode
                .as_deref()
                .and_then(render::qr_svg)
                .ok_or_else(|| Error::InvalidXml("NFC-e sem QR code".to_string()))?;
            render::compile(NFCE_80MM, invoice, vec![("/qrcode.svg".to_string(), qr)])
        }
        ("55", Layout::A4Portrait) => {
            let modules = barcode::code128_modules(&invoice.key)
                .ok_or_else(|| Error::InvalidXml("chave de acesso inválida".to_string()))?;
            render::compile(
                DANFE_A4,
                invoice,
                vec![("/chave.svg".to_string(), barcode::svg(&modules, 50))],
            )
        }
        ("65", _) => Err(Error::UnsupportedLayout {
            document: "DANFE NFC-e",
            layout,
        }),
        ("55", _) => Err(Error::UnsupportedLayout {
            document: "DANFE",
            layout,
        }),
        (model, _) => Err(Error::UnsupportedDocument(format!("modelo {model}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NFE: &str = include_str!("../tests/fixtures/nfe_autorizada_homologacao.xml");
    const NFCE: &str = include_str!("../tests/fixtures/nfce_autorizada_homologacao.xml");

    #[test]
    fn reads_what_the_danfe_prints() {
        let invoice = parse(NFE).unwrap();
        assert_eq!(invoice.model, "55");
        assert!(invoice.homologation);
        assert_eq!(
            invoice.emitter.document.as_deref(),
            Some("11.222.333/0001-81")
        );
        assert_eq!(
            invoice.recipient.as_ref().unwrap().document.as_deref(),
            Some("529.982.247-25")
        );
        assert_eq!(invoice.items[0].cst, "0102");
        assert_eq!(invoice.totals.total, "23,98");
        assert_eq!(invoice.payments[0].method, "Dinheiro");
        assert_eq!(invoice.access_key.split(' ').count(), 11);

        let consumer = parse(NFCE).unwrap();
        assert!(consumer.qrcode.as_deref().unwrap().starts_with("https://"));
        assert!(consumer.recipient.is_none());
    }

    #[test]
    fn renders_the_danfe_and_the_danfe_nfce() {
        for xml in [NFE, NFCE] {
            assert!(render_default(xml).unwrap().starts_with(b"%PDF"));
        }
    }

    #[test]
    fn the_same_xml_gives_the_same_pdf() {
        assert_eq!(render_default(NFE).unwrap(), render_default(NFE).unwrap());
    }

    #[test]
    fn refuses_what_is_not_authorized_or_not_printable() {
        assert!(matches!(parse("<NFe/>"), Err(Error::InvalidXml(_))));
        let without_protocol = NFE.replace("infProt", "outroGrupo");
        assert_eq!(parse(&without_protocol), Err(Error::NotAuthorized));
        assert!(matches!(
            render(NFE, Layout::Roll80mm),
            Err(Error::UnsupportedLayout { .. })
        ));
    }
}
