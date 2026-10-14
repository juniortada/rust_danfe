//! Reads an authorized `nfeProc` into an [`Invoice`].

use roxmltree::{Document, Node};

use crate::{
    format,
    model::{Address, Duplicate, Emitter, Invoice, Item, Payment, Recipient, Totals},
    Error,
};

trait Find<'a, 'input> {
    fn child(&self, name: &str) -> Option<Node<'a, 'input>>;
    fn children_named(&self, name: &'static str) -> Vec<Node<'a, 'input>>;
    fn text_of(&self, name: &str) -> Option<String>;
}

impl<'a, 'input> Find<'a, 'input> for Node<'a, 'input> {
    fn child(&self, name: &str) -> Option<Self> {
        self.children()
            .find(|node| node.is_element() && node.tag_name().name() == name)
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

/// A money field, "0" when absent.
fn money(node: Option<Node<'_, '_>>, name: &str) -> String {
    format::br_number(
        &node
            .and_then(|node| node.text_of(name))
            .unwrap_or_else(|| "0".to_string()),
        2,
    )
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
        phone: address.text_of("fone"),
    })
}

fn item(det: Node<'_, '_>) -> Item {
    let prod = det.child("prod");
    let icms = det
        .child("imposto")
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
    Item {
        code: text(prod, "cProd"),
        description: text(prod, "xProd"),
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
        icms_rate: money(icms, "pICMS"),
    }
}

fn totals(totals: Node<'_, '_>) -> Totals {
    let node = Some(totals);
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
        ipi: money(node, "vIPI"),
        total: money(node, "vNF"),
        approximate_taxes: totals
            .text_of("vTotTrib")
            .map(|value| format::br_number(&value, 2)),
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

/// # Errors
/// [`Error::InvalidXml`] for anything that is not a NF-e, [`Error::NotAuthorized`]
/// without the authorization protocol.
pub fn parse(xml: &str) -> Result<Invoice, Error> {
    let document = Document::parse(xml).map_err(|err| invalid(&err.to_string()))?;
    let root = document.root_element();
    let inf = descendant(root, "infNFe").ok_or_else(|| invalid("sem infNFe"))?;
    let ide = inf.child("ide").ok_or_else(|| invalid("sem ide"))?;
    let emit = inf.child("emit").ok_or_else(|| invalid("sem emit"))?;
    let icms_totals = inf
        .child("total")
        .and_then(|total| total.child("ICMSTot"))
        .ok_or_else(|| invalid("sem ICMSTot"))?;
    let protocol = descendant(root, "infProt").ok_or(Error::NotAuthorized)?;
    let key = inf
        .attribute("Id")
        .and_then(|id| id.strip_prefix("NFe"))
        .ok_or_else(|| invalid("sem chave de acesso"))?
        .to_string();
    let number = ide.text_of("nNF").unwrap_or_default();
    let items: Vec<Item> = inf.children_named("det").into_iter().map(item).collect();
    let supplement = descendant(root, "infNFeSupl");
    Ok(Invoice {
        model: ide.text_of("mod").unwrap_or_default(),
        homologation: ide.text_of("tpAmb").as_deref() == Some("2"),
        offline: ide.text_of("tpEmis").as_deref() == Some("9"),
        number_padded: format!("{number:0>9}"),
        number: format::br_number(&number, 0),
        series: ide.text_of("serie"),
        issued_at: format::date_time(&ide.text_of("dhEmi").unwrap_or_default()),
        operation: ide.text_of("natOp"),
        direction: if ide.text_of("tpNF").as_deref() == Some("0") {
            "0"
        } else {
            "1"
        }
        .to_string(),
        access_key: format::grouped_key(&key),
        key,
        protocol: format!(
            "{} - {}",
            protocol.text_of("nProt").unwrap_or_default(),
            format::date_time(&protocol.text_of("dhRecbto").unwrap_or_default())
        ),
        emitter: Emitter {
            name: emit.text_of("xNome"),
            trade_name: emit.text_of("xFant"),
            document: document_label(emit),
            state_registration: emit.text_of("IE"),
            address: address(emit.child("enderEmit")),
        },
        recipient: inf.child("dest").map(|dest| Recipient {
            name: dest.text_of("xNome"),
            document: document_label(dest),
            state_registration: dest.text_of("IE"),
            address: address(dest.child("enderDest")),
        }),
        item_count: items.len(),
        items,
        totals: totals(icms_totals),
        freight_mode: format::freight(&text(inf.child("transp"), "modFrete")).to_string(),
        payments: payments(inf),
        change: inf
            .child("pag")
            .and_then(|pag| pag.text_of("vTroco"))
            .map(|change| format::br_number(&change, 2)),
        duplicates: duplicates(inf),
        additional: inf.child("infAdic").and_then(|info| info.text_of("infCpl")),
        qrcode: supplement.and_then(|node| node.text_of("qrCode")),
        url_chave: supplement.and_then(|node| node.text_of("urlChave")),
    })
}
