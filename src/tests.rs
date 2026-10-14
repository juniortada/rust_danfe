use typst::layout::{Frame, FrameItem};

use super::*;

const NFE: &str = include_str!("../tests/fixtures/nfe_autorizada_homologacao.xml");
const NFCE: &str = include_str!("../tests/fixtures/nfce_autorizada_homologacao.xml");
const REFORM: &str = include_str!("../tests/fixtures/nfe_reforma_tributaria.xml");

/// A 1×1 transparent PNG.
const PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0B, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x60, 0x00, 0x02, 0x00,
    0x00, 0x05, 0x00, 0x01, 0x7A, 0x5E, 0xAB, 0x3F, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44,
    0xAE, 0x42, 0x60, 0x82,
];

fn document(xml: &str, layout: Layout) -> typst_layout::PagedDocument {
    lay_out(&parse(xml).unwrap(), layout, &Options::default()).unwrap()
}

fn text_sizes(frame: &Frame, sizes: &mut Vec<f64>) {
    for (_, item) in frame.items() {
        match item {
            FrameItem::Group(group) => text_sizes(&group.frame, sizes),
            FrameItem::Text(text) => sizes.push(text.size.to_pt()),
            _ => {}
        }
    }
}

fn smallest_text(document: &typst_layout::PagedDocument) -> f64 {
    let mut sizes = Vec::new();
    for page in document.pages() {
        text_sizes(&page.frame, &mut sizes);
    }
    sizes.into_iter().fold(f64::MAX, f64::min)
}

/// The NF-e with as many copies of its item as asked.
fn with_items(count: usize) -> String {
    let start = REFORM.find("<det nItem=\"1\">").unwrap();
    let end = REFORM.find("<det nItem=\"2\">").unwrap();
    let item = &REFORM[start..end];
    let items: String = (1..=count)
        .map(|number| item.replacen("nItem=\"1\"", &format!("nItem=\"{number}\""), 1))
        .collect();
    let last = REFORM.find("<total>").unwrap();
    format!("{}{}{}", &REFORM[..start], items, &REFORM[last..])
}

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
    assert_eq!(invoice.number_dotted, "000.000.001");
    assert_eq!(invoice.series, "001");
    assert_eq!(
        invoice.emitter.tax_regime.as_deref(),
        Some("1 - SIMPLES NACIONAL")
    );
    // dhRecbto in UTC is printed in the offset of dhEmi.
    assert_eq!(
        invoice.protocol.as_deref(),
        Some("100000000000002 - 01/10/2026 03:58:42")
    );
    assert_eq!(invoice.consumer, "CONSUMIDOR CPF: 529.982.247-25");

    let consumer = parse(NFCE).unwrap();
    assert!(consumer.qrcode.as_deref().unwrap().starts_with("https://"));
    assert!(consumer.recipient.is_none());
    assert_eq!(consumer.consumer, "CONSUMIDOR NÃO IDENTIFICADO");
}

#[test]
fn reads_the_consumption_tax_reform() {
    let invoice = parse(REFORM).unwrap();
    let full = invoice.items[0].ibs_cbs.as_ref().unwrap();
    assert_eq!(full.class, "000001");
    assert_eq!(full.base.as_deref(), Some("23,98"));
    assert_eq!(full.ibs_state_rate.as_deref(), Some("0,10"));
    assert_eq!(full.cbs_rate.as_deref(), Some("0,90"));
    assert_eq!(full.cbs.as_deref(), Some("0,22"));
    // gRed: the effective rate (NT 2026.010 §4.3).
    let reduced = invoice.items[1].ibs_cbs.as_ref().unwrap();
    assert_eq!(reduced.ibs_state_rate.as_deref(), Some("0,04"));
    assert_eq!(reduced.cbs_rate.as_deref(), Some("0,36"));
    let selective = invoice.items[0].selective.as_ref().unwrap();
    assert_eq!(
        (selective.rate.as_str(), selective.value.as_str()),
        ("2,00", "0,48")
    );
    assert_eq!(invoice.items[0].ipi.as_ref().unwrap().value, "0,96");
    assert_eq!(
        invoice.items[0].additional.as_deref(),
        Some("Lote 42, validade 03/2027")
    );
    assert!(invoice.items[1].selective.is_none());

    let reform = &invoice.reform;
    assert_eq!(reform.cbs.as_deref(), Some("0,33"));
    assert_eq!(reform.ibs_state.as_deref(), Some("0,03"));
    assert_eq!(reform.ibs_city.as_deref(), Some("0,00"));
    assert_eq!(reform.selective.as_deref(), Some("0,48"));
    assert!(reform.mono.is_none());
    assert_eq!(
        invoice.emitter.tax_regime.as_deref(),
        Some("3 - REGIME NORMAL")
    );
    assert_eq!(
        invoice.emitter.st_registration.as_deref(),
        Some("111222333444")
    );
    assert_eq!(
        invoice.fiscal_info.as_deref(),
        Some("Valores totais do ICMS interestadual: DIFAL da UF destino R$ 0,00.")
    );
    assert_eq!(
        (invoice.exit_on.as_deref(), invoice.exit_time.as_deref()),
        (Some("02/12/2026"), Some("11:00:00"))
    );
    assert_eq!(invoice.totals.surcharge, "10,00");

    let transport = &invoice.transport;
    assert_eq!(transport.mode, "0 - Por conta do Remetente (CIF)");
    assert_eq!(
        transport.carrier.as_ref().unwrap().document.as_deref(),
        Some("12.345.678/0001-95")
    );
    assert_eq!(
        transport.vehicle.as_ref().unwrap().plate.as_deref(),
        Some("ABC1D23")
    );
    assert_eq!(
        transport.volumes[0].gross_weight.as_deref(),
        Some("13,200 kg")
    );
    assert_eq!(invoice.duplicates.len(), 2);
    assert_eq!(
        invoice.duplicates[1].due_date.as_deref(),
        Some("02/02/2027")
    );

    // Without the reform groups the fields stay blank.
    let before = parse(NFE).unwrap();
    assert_eq!(before.reform.cbs, None);
    assert!(before.items[0].ibs_cbs.is_none());
}

#[test]
fn renders_every_document_and_the_same_xml_gives_the_same_pdf() {
    for xml in [NFE, NFCE, REFORM] {
        let pdf = render_default(xml).unwrap();
        assert!(pdf.starts_with(b"%PDF"));
        assert_eq!(pdf, render_default(xml).unwrap());
    }
}

#[test]
fn pages_have_the_size_of_their_layout() {
    let a4 = document(REFORM, Layout::A4Portrait);
    let size = a4.pages()[0].frame.size();
    assert!((size.x.to_mm() - 210.0).abs() < 0.5 && (size.y.to_mm() - 297.0).abs() < 0.5);
    let roll = document(NFCE, Layout::Roll80mm);
    assert!((roll.pages()[0].frame.width().to_mm() - 80.0).abs() < 0.5);
    assert_eq!(roll.pages().len(), 1);
}

/// The reform NF-e printed as DANFE Simplificado Tipo 2, with its QR code.
fn simplified_type_2() -> String {
    REFORM
        .replacen("<tpImp>1</tpImp>", "<tpImp>6</tpImp>", 1)
        .replacen(
            "</infNFe>",
            "</infNFe><infNFeSupl><qrCode>https://www.homologacao.nfce.fazenda.sp.gov.br/qrcode?p=35261211222333000181550010000012341624557528|3|2</qrCode><urlChave>https://www.homologacao.nfce.fazenda.sp.gov.br/consulta</urlChave></infNFeSupl>",
            1,
        )
}

#[test]
fn every_document_prints_in_the_layouts_its_norm_allows() {
    let landscape = document(REFORM, Layout::A4Landscape);
    let size = landscape.pages()[0].frame.size();
    assert!((size.x.to_mm() - 297.0).abs() < 0.5 && (size.y.to_mm() - 210.0).abs() < 0.5);
    let narrow = document(NFCE, Layout::Roll58mm);
    assert!((narrow.pages()[0].frame.width().to_mm() - 58.0).abs() < 0.5);
    let nfce_a4 = document(NFCE, Layout::A4Portrait);
    assert!((nfce_a4.pages()[0].frame.width().to_mm() - 210.0).abs() < 0.5);

    let simplified = simplified_type_2();
    for layout in [Layout::Roll80mm, Layout::Roll58mm, Layout::A4Portrait] {
        assert!(render(&simplified, layout).unwrap().starts_with(b"%PDF"));
    }
    assert_eq!(
        parse(&simplified).unwrap().reform.ibs.as_deref(),
        Some("0,03")
    );
    assert!(matches!(
        render(&simplified, Layout::A4Landscape),
        Err(Error::UnsupportedLayout {
            document: "DANFE Simplificado Tipo 2",
            ..
        })
    ));
    assert!(matches!(
        render(NFCE, Layout::A4Landscape),
        Err(Error::UnsupportedLayout {
            document: "DANFE NFC-e",
            ..
        })
    ));
}

#[test]
fn the_print_format_picks_the_default_layout() {
    let landscape = parse(&REFORM.replacen("<tpImp>1</tpImp>", "<tpImp>2</tpImp>", 1)).unwrap();
    assert_eq!(default_layout(&landscape), Ok(Layout::A4Landscape));
    assert_eq!(
        default_layout(&parse(REFORM).unwrap()),
        Ok(Layout::A4Portrait)
    );
    assert_eq!(default_layout(&parse(NFCE).unwrap()), Ok(Layout::Roll80mm));
    let simplified = parse(&simplified_type_2()).unwrap();
    assert_eq!(default_layout(&simplified), Ok(Layout::Roll80mm));
    let label = REFORM.replacen("<tpImp>1</tpImp>", "<tpImp>3</tpImp>", 1);
    assert!(matches!(
        render_default(&label),
        Err(Error::UnsupportedDocument(_))
    ));
}

#[test]
fn a_long_invoice_continues_on_more_sheets() {
    let document = document(&with_items(70), Layout::A4Portrait);
    assert!(
        document.pages().len() >= 2,
        "{} folhas",
        document.pages().len()
    );
}

#[test]
fn no_text_is_below_the_minimum_size() {
    // Anexo II §3.7: nothing in the DANFE is smaller than 5 pt.
    assert!(smallest_text(&document(REFORM, Layout::A4Portrait)) >= 5.0);
    assert!(smallest_text(&document(&with_items(70), Layout::A4Portrait)) >= 5.0);
}

#[test]
fn an_alphanumeric_key_gets_the_hybrid_barcode() {
    let key = "35261211222333000181550010000012341624557528";
    let alphanumeric = "352612112AB333000181550010000012341624557528";
    let xml = REFORM.replace(key, alphanumeric);
    let invoice = parse(&xml).unwrap();
    assert!(invoice.key_alphanumeric);
    assert!(barcode::code128_symbols(&invoice.key)
        .unwrap()
        .contains(&101));
    assert!(render_default(&xml).unwrap().starts_with(b"%PDF"));
}

#[test]
fn an_off_line_nfce_prints_without_protocol_and_nothing_else_does() {
    let start = NFCE.find("<protNFe").unwrap();
    let end = NFCE.find("</protNFe>").unwrap() + "</protNFe>".len();
    let unauthorized = format!("{}{}", &NFCE[..start], &NFCE[end..]);
    assert_eq!(parse(&unauthorized), Err(Error::NotAuthorized));

    let offline = unauthorized.replacen("<tpEmis>1</tpEmis>", "<tpEmis>9</tpEmis>", 1);
    let invoice = parse(&offline).unwrap();
    assert!(invoice.offline);
    assert_eq!(invoice.protocol, None);
    let copy = Options {
        establishment_copy: true,
        ..Options::default()
    };
    assert!(render_with(&offline, Layout::Roll80mm, &copy)
        .unwrap()
        .starts_with(b"%PDF"));
}

#[test]
fn takes_a_logo_and_extra_fonts() {
    let options = Options {
        logo: Some(PNG.to_vec()),
        fonts: vec![include_bytes!("../fonts/LiberationSerif-Regular.ttf").to_vec()],
        ..Options::default()
    };
    assert!(render_with(REFORM, Layout::A4Portrait, &options)
        .unwrap()
        .starts_with(b"%PDF"));
    assert!(render_with(NFCE, Layout::Roll80mm, &options)
        .unwrap()
        .starts_with(b"%PDF"));
    let gif = Options {
        logo: Some(b"GIF89a".to_vec()),
        ..Options::default()
    };
    assert_eq!(
        render_with(NFE, Layout::A4Portrait, &gif),
        Err(Error::UnsupportedLogo)
    );
}

#[test]
fn refuses_what_is_not_printable() {
    assert!(matches!(parse("<NFe/>"), Err(Error::InvalidXml(_))));
    let without_protocol = NFE.replace("infProt", "outroGrupo");
    assert_eq!(parse(&without_protocol), Err(Error::NotAuthorized));
    assert!(matches!(
        render(NFE, Layout::Roll80mm),
        Err(Error::UnsupportedLayout {
            document: "DANFE",
            ..
        })
    ));
}
