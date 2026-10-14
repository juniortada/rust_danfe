//! Reads a NF-e or NFC-e XML (`nfeProc`, or the bare `NFe` of an off-line
//! contingency) into an [`Invoice`].

use roxmltree::{Document, Node};
use rust_decimal::Decimal;

use crate::{
    format,
    model::{
        Address, Carrier, Duplicate, Emitter, Invoice, Issqn, Item, ItemIbsCbs, ItemTax,
        MonoTotals, Payment, Recipient, ReformTotals, Totals, Transport, Vehicle, Volume,
    },
    Error,
};

trait Find<'a, 'input> {
    fn child(&self, name: &str) -> Option<Node<'a, 'input>>;
    fn path(&self, names: &[&str]) -> Option<Node<'a, 'input>>;
    fn children_named(&self, name: &'static str) -> Vec<Node<'a, 'input>>;
    fn text_of(&self, name: &str) -> Option<String>;
}

impl<'a, 'input> Find<'a, 'input> for Node<'a, 'input> {
    fn child(&self, name: &str) -> Option<Self> {
        self.children()
            .find(|node| node.is_element() && node.tag_name().name() == name)
    }

    fn path(&self, names: &[&str]) -> Option<Self> {
        names.iter().try_fold(*self, |node, name| node.child(name))
    }

    fn children_named(&self, name: &'static str) -> Vec<Self> {
        self.children()
            .filter(|node| node.is_element() && node.tag_name().name() == name)
            .collect()
    }

    fn text_of(&self, name: &str) -> Option<String> {
        self.child(name)
            .and_then(|node| node.text())
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
    }
}

fn descendant<'a, 'input>(node: Node<'a, 'input>, name: &str) -> Option<Node<'a, 'input>> {
    node.descendants()
        .find(|child| child.is_element() && child.tag_name().name() == name)
}

fn invalid(message: &str) -> Error {
    Error::InvalidXml(message.to_string())
}

fn text(node: Option<Node<'_, '_>>, name: &str) -> String {
    node.and_then(|node| node.text_of(name)).unwrap_or_default()
}

/// A money field, "0,00" when absent.
fn money(node: Option<Node<'_, '_>>, name: &str) -> String {
    optional_money(node, name).unwrap_or_else(|| "0,00".to_string())
}

/// A money field the DANFE leaves blank when the XML does not have it.
fn optional_money(node: Option<Node<'_, '_>>, name: &str) -> Option<String> {
    node.and_then(|node| node.text_of(name))
        .map(|value| format::br_number(&value, 2))
}

fn optional_rate(node: Option<Node<'_, '_>>, name: &str) -> Option<String> {
    node.and_then(|node| node.text_of(name))
        .map(|value| format::rate(&value))
}

fn document_label(node: Node<'_, '_>) -> Option<String> {
    node.text_of("CNPJ")
        .map(|cnpj| format::cnpj(&cnpj))
        .or_else(|| node.text_of("CPF").map(|cpf| format::cpf(&cpf)))
        .or_else(|| node.text_of("idEstrangeiro"))
}

fn address(node: Option<Node<'_, '_>>) -> Option<Address> {
    let address = node?;
    let street = [
        address.text_of("xLgr"),
        address.text_of("nro"),
        address.text_of("xCpl"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join(", ");
    Some(Address {
        street,
        district: address.text_of("xBairro"),
        city: address.text_of("xMun"),
        state: address.text_of("UF"),
        zip_code: address.text_of("CEP").map(|cep| format::zip_code(&cep)),
        phone: address.text_of("fone").map(|phone| format::phone(&phone)),
    })
}

/// The rate printed for an IBS or CBS group: the effective one when the
/// rate is reduced (`gRed`), the current one otherwise (NT 2026.010 §4.3).
fn reform_rate(group: Option<Node<'_, '_>>, rate: &str) -> Option<String> {
    let group = group?;
    group
        .child("gRed")
        .and_then(|reduced| optional_rate(Some(reduced), "pAliqEfet"))
        .or_else(|| optional_rate(Some(group), rate))
}

fn ibs_cbs(tax: Option<Node<'_, '_>>) -> Option<ItemIbsCbs> {
    let group = tax?.child("IBSCBS")?;
    let values = group.child("gIBSCBS");
    let state = values.and_then(|values| values.child("gIBSUF"));
    let city = values.and_then(|values| values.child("gIBSMun"));
    let cbs = values.and_then(|values| values.child("gCBS"));
    Some(ItemIbsCbs {
        class: group.text_of("cClassTrib").unwrap_or_default(),
        base: optional_money(values, "vBC"),
        ibs_state_rate: reform_rate(state, "pIBSUF"),
        ibs_state: optional_money(state, "vIBSUF"),
        ibs_city_rate: reform_rate(city, "pIBSMun"),
        ibs_city: optional_money(city, "vIBSMun"),
        cbs_rate: reform_rate(cbs, "pCBS"),
        cbs: optional_money(cbs, "vCBS"),
    })
}

fn item(det: Node<'_, '_>) -> Item {
    let prod = det.child("prod");
    let tax = det.child("imposto");
    let icms = tax
        .and_then(|tax| tax.child("ICMS"))
        .and_then(|icms| icms.children().find(Node::is_element));
    let cst = icms.map_or_else(String::new, |group| {
        format!(
            "{}{}",
            group.text_of("orig").unwrap_or_default(),
            group
                .text_of("CST")
                .or_else(|| group.text_of("CSOSN"))
                .unwrap_or_default()
        )
    });
    let ipi = tax.and_then(|tax| tax.path(&["IPI", "IPITrib"]));
    let selective = tax.and_then(|tax| tax.child("IS"));
    Item {
        code: text(prod, "cProd"),
        description: text(prod, "xProd"),
        additional: det.text_of("infAdProd"),
        ncm: text(prod, "NCM"),
        cst,
        cfop: text(prod, "CFOP"),
        unit: text(prod, "uCom"),
        quantity: format::br_number(&text(prod, "qCom"), 4),
        unit_price: format::br_number(&text(prod, "vUnCom"), 2),
        total: format::br_number(&text(prod, "vProd"), 2),
        discount: money(prod, "vDesc"),
        icms_base: money(icms, "vBC"),
        icms_value: money(icms, "vICMS"),
        icms_rate: optional_rate(icms, "pICMS").unwrap_or_else(|| "0,00".to_string()),
        ipi: ipi.map(|ipi| ItemTax {
            base: money(Some(ipi), "vBC"),
            rate: optional_rate(Some(ipi), "pIPI").unwrap_or_else(|| "0,00".to_string()),
            value: money(Some(ipi), "vIPI"),
        }),
        ibs_cbs: ibs_cbs(tax),
        selective: selective
            .filter(|group| group.child("vIS").is_some())
            .map(|group| ItemTax {
                base: money(Some(group), "vBCIS"),
                rate: optional_rate(Some(group), "pIS").unwrap_or_else(|| "0,00".to_string()),
                value: money(Some(group), "vIS"),
            }),
    }
}

fn decimal(node: Node<'_, '_>, name: &str) -> Decimal {
    node.text_of(name)
        .and_then(|value| value.parse().ok())
        .unwrap_or_default()
}

fn totals(icms: Node<'_, '_>) -> Totals {
    let node = Some(icms);
    let surcharge = decimal(icms, "vFrete") + decimal(icms, "vSeg") + decimal(icms, "vOutro");
    Totals {
        icms_base: money(node, "vBC"),
        icms: money(node, "vICMS"),
        st_base: money(node, "vBCST"),
        st: money(node, "vST"),
        products: money(node, "vProd"),
        freight: money(node, "vFrete"),
        insurance: money(node, "vSeg"),
        discount: money(node, "vDesc"),
        other: money(node, "vOutro"),
        surcharge: format::br_number(&surcharge.to_string(), 2),
        ipi: money(node, "vIPI"),
        total: money(node, "vNF"),
        approximate_taxes: optional_money(node, "vTotTrib"),
        fcp: optional_money(node, "vFCP"),
        fcp_st: optional_money(node, "vFCPST"),
        difal_destination: optional_money(node, "vICMSUFDest"),
        fcp_destination: optional_money(node, "vFCPUFDest"),
        icms_mono_quantity: icms
            .text_of("qBCMono")
            .map(|value| format::br_number(&value, 4)),
        icms_mono: optional_money(node, "vICMSMono"),
        icms_mono_withheld_quantity: icms
            .text_of("qBCMonoReten")
            .map(|value| format::br_number(&value, 4)),
        icms_mono_withheld: optional_money(node, "vICMSMonoReten"),
    }
}

/// `total/IBSCBSTot` and `total/ISTot` (NT 2025.002 W31–W59b).
fn reform(total: Option<Node<'_, '_>>) -> ReformTotals {
    let ibs_cbs = total.and_then(|total| total.child("IBSCBSTot"));
    let mono = ibs_cbs.and_then(|totals| totals.child("gMono"));
    ReformTotals {
        cbs: optional_money(ibs_cbs.and_then(|totals| totals.child("gCBS")), "vCBS"),
        ibs: optional_money(ibs_cbs.and_then(|totals| totals.child("gIBS")), "vIBS"),
        ibs_state: optional_money(
            ibs_cbs.and_then(|totals| totals.path(&["gIBS", "gIBSUF"])),
            "vIBSUF",
        ),
        ibs_city: optional_money(
            ibs_cbs.and_then(|totals| totals.path(&["gIBS", "gIBSMun"])),
            "vIBSMun",
        ),
        selective: optional_money(total.and_then(|total| total.child("ISTot")), "vIS"),
        mono: mono.map(|mono| MonoTotals {
            ibs: optional_money(Some(mono), "vIBSMono"),
            cbs: optional_money(Some(mono), "vCBSMono"),
            ibs_withheld: optional_money(Some(mono), "vIBSMonoReten"),
            cbs_withheld: optional_money(Some(mono), "vCBSMonoReten"),
        }),
    }
}

fn issqn(total: Option<Node<'_, '_>>) -> Option<Issqn> {
    let issqn = total?.child("ISSQNtot")?;
    Some(Issqn {
        services: money(Some(issqn), "vServ"),
        base: money(Some(issqn), "vBC"),
        value: money(Some(issqn), "vISS"),
    })
}

fn weight(node: Node<'_, '_>, name: &str) -> Option<String> {
    node.text_of(name)
        .map(|value| format!("{} kg", format::br_number(&value, 3)))
}

fn transport(inf: Node<'_, '_>) -> Transport {
    let transp = inf.child("transp");
    Transport {
        mode: format::freight(&text(transp, "modFrete")).to_string(),
        carrier: transp
            .and_then(|transp| transp.child("transporta"))
            .map(|carrier| Carrier {
                name: carrier.text_of("xNome"),
                document: document_label(carrier),
                state_registration: carrier.text_of("IE"),
                address: carrier.text_of("xEnder"),
                city: carrier.text_of("xMun"),
                state: carrier.text_of("UF"),
            }),
        vehicle: transp
            .and_then(|transp| transp.child("veicTransp"))
            .map(|vehicle| Vehicle {
                plate: vehicle.text_of("placa"),
                state: vehicle.text_of("UF"),
                antt: vehicle.text_of("RNTC"),
            }),
        volumes: transp
            .map(|transp| {
                transp
                    .children_named("vol")
                    .into_iter()
                    .map(|volume| Volume {
                        quantity: volume.text_of("qVol"),
                        kind: volume.text_of("esp"),
                        brand: volume.text_of("marca"),
                        numbering: volume.text_of("nVol"),
                        gross_weight: weight(volume, "pesoB"),
                        net_weight: weight(volume, "pesoL"),
                    })
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn payments(inf: Node<'_, '_>) -> Vec<Payment> {
    inf.child("pag")
        .map(|pag| {
            pag.children_named("detPag")
                .into_iter()
                .map(|payment| Payment {
                    method: format::payment(
                        &payment.text_of("tPag").unwrap_or_default(),
                        payment.text_of("xPag"),
                    ),
                    amount: money(Some(payment), "vPag"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn duplicates(inf: Node<'_, '_>) -> Vec<Duplicate> {
    inf.child("cobr")
        .map(|cobr| {
            cobr.children_named("dup")
                .into_iter()
                .map(|dup| Duplicate {
                    number: dup.text_of("nDup"),
                    due_date: dup.text_of("dVenc").map(|due| format::date(&due)),
                    amount: money(Some(dup), "vDup"),
                })
                .collect()
        })
        .unwrap_or_default()
}

/// "CONSUMIDOR CPF: …" or "CONSUMIDOR NÃO IDENTIFICADO" (Manual DANFE NFC-e §3.1.6).
fn consumer(dest: Option<Node<'_, '_>>) -> String {
    dest.and_then(|dest| {
        dest.text_of("CNPJ")
            .map(|cnpj| format!("CONSUMIDOR CNPJ: {}", format::cnpj(&cnpj)))
            .or_else(|| {
                dest.text_of("CPF")
                    .map(|cpf| format!("CONSUMIDOR CPF: {}", format::cpf(&cpf)))
            })
            .or_else(|| {
                dest.text_of("idEstrangeiro")
                    .map(|foreign| format!("CONSUMIDOR Id. Estrangeiro: {foreign}"))
            })
    })
    .unwrap_or_else(|| "CONSUMIDOR NÃO IDENTIFICADO".to_string())
}

/// # Errors
/// [`Error::InvalidXml`] for anything that is not a NF-e, [`Error::NotAuthorized`]
/// without the authorization protocol outside an off-line contingency.
pub fn parse(xml: &str) -> Result<Invoice, Error> {
    let document = Document::parse(xml).map_err(|err| invalid(&err.to_string()))?;
    let root = document.root_element();
    let inf = descendant(root, "infNFe").ok_or_else(|| invalid("sem infNFe"))?;
    let ide = inf.child("ide").ok_or_else(|| invalid("sem ide"))?;
    let emit = inf.child("emit").ok_or_else(|| invalid("sem emit"))?;
    let total = inf.child("total");
    let icms_totals = total
        .and_then(|total| total.child("ICMSTot"))
        .ok_or_else(|| invalid("sem ICMSTot"))?;
    let offline = ide.text_of("tpEmis").as_deref() == Some("9");
    let protocol = match descendant(root, "infProt") {
        Some(protocol) => Some(format!(
            "{} - {}",
            protocol.text_of("nProt").unwrap_or_default(),
            format::local_date_time(
                &protocol.text_of("dhRecbto").unwrap_or_default(),
                &ide.text_of("dhEmi").unwrap_or_default()
            )
        )),
        None if offline => None,
        None => return Err(Error::NotAuthorized),
    };
    let key = inf
        .attribute("Id")
        .and_then(|id| id.strip_prefix("NFe"))
        .ok_or_else(|| invalid("sem chave de acesso"))?
        .to_string();
    let number = ide.text_of("nNF").unwrap_or_default();
    let issued = ide.text_of("dhEmi").unwrap_or_default();
    let departure = ide.text_of("dhSaiEnt");
    let items: Vec<Item> = inf.children_named("det").into_iter().map(item).collect();
    let dest = inf.child("dest");
    let supplement = descendant(root, "infNFeSupl");
    let info = inf.child("infAdic");
    Ok(Invoice {
        model: ide.text_of("mod").unwrap_or_default(),
        print_format: ide.text_of("tpImp").unwrap_or_default(),
        homologation: ide.text_of("tpAmb").as_deref() == Some("2"),
        offline,
        number_padded: format!("{number:0>9}"),
        number_dotted: format::dotted_number(&number),
        number: format::br_number(&number, 0),
        series: format!("{:0>3}", ide.text_of("serie").unwrap_or_default()),
        issued_at: format::date_time(&issued),
        issued_on: format::date_part(&issued),
        exit_on: departure.as_deref().map(format::date_part),
        exit_time: departure.as_deref().map(format::time_part),
        operation: ide.text_of("natOp"),
        direction: if ide.text_of("tpNF").as_deref() == Some("0") {
            "0"
        } else {
            "1"
        }
        .to_string(),
        access_key: format::grouped_key(&key),
        key_alphanumeric: key.bytes().any(|byte| byte.is_ascii_alphabetic()),
        key,
        protocol,
        emitter: Emitter {
            name: emit.text_of("xNome"),
            trade_name: emit.text_of("xFant"),
            document: document_label(emit),
            state_registration: emit.text_of("IE"),
            st_registration: emit.text_of("IEST"),
            municipal_registration: emit.text_of("IM"),
            tax_regime: emit.text_of("CRT").map(|crt| format::tax_regime(&crt)),
            address: address(emit.child("enderEmit")),
        },
        recipient: dest.map(|dest| Recipient {
            name: dest.text_of("xNome"),
            document: document_label(dest),
            state_registration: dest.text_of("IE"),
            address: address(dest.child("enderDest")),
        }),
        consumer: consumer(dest),
        item_count: items.len(),
        items,
        totals: totals(icms_totals),
        reform: reform(total),
        issqn: issqn(total),
        transport: transport(inf),
        payments: payments(inf),
        change: optional_money(inf.child("pag"), "vTroco"),
        duplicates: duplicates(inf),
        fiscal_info: info.and_then(|info| info.text_of("infAdFisco")),
        additional: info.and_then(|info| info.text_of("infCpl")),
        qrcode: supplement.and_then(|node| node.text_of("qrCode")),
        url_chave: supplement.and_then(|node| node.text_of("urlChave")),
    })
}
