//! What the DANFE prints, already formatted the way it is printed.
//!
//! Serialized as the templates' `data.json`; field names follow the template
//! variables. A field the XML does not have is `None` and prints nothing
//! (NT 2026.010 §4.4).

use serde::Serialize;

/// A NF-e or NFC-e read from its XML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Invoice {
    /// `mod`: "55" (NF-e) or "65" (NFC-e).
    pub model: String,
    /// `tpImp`: 1 and 2 DANFE, 4 and 5 DANFE NFC-e, 6 DANFE Simplificado Tipo 2.
    pub print_format: String,
    /// `tpAmb` = 2.
    pub homologation: bool,
    /// `tpEmis` = 9 (off-line contingency): printed before the authorization.
    pub offline: bool,
    pub number: String,
    /// "000000123".
    pub number_padded: String,
    /// "000.000.123" (Anexo II §3.9).
    pub number_dotted: String,
    /// "001".
    pub series: String,
    /// "dd/mm/aaaa hh:mm:ss" in the document's own offset.
    pub issued_at: String,
    /// "dd/mm/aaaa".
    pub issued_on: String,
    pub exit_on: Option<String>,
    pub exit_time: Option<String>,
    pub operation: Option<String>,
    /// `tpNF`: "0" (entrada) or "1" (saída).
    pub direction: String,
    /// The key in eleven blocks of four.
    pub access_key: String,
    /// The key as in the XML, for the barcode.
    #[serde(skip)]
    pub key: String,
    /// The key has letters (alphanumeric CNPJ, NT Conjunta 2025.001).
    pub key_alphanumeric: bool,
    /// Protocol number and authorization time; `None` before the
    /// authorization (off-line contingency).
    pub protocol: Option<String>,
    pub emitter: Emitter,
    pub recipient: Option<Recipient>,
    /// The NFC-e consumer line (Manual DANFE NFC-e §3.1.6).
    pub consumer: String,
    pub items: Vec<Item>,
    pub item_count: usize,
    pub totals: Totals,
    /// Totals of the consumption tax reform (NT 2026.010 §4.1).
    pub reform: ReformTotals,
    pub issqn: Option<Issqn>,
    pub transport: Transport,
    pub payments: Vec<Payment>,
    pub change: Option<String>,
    pub duplicates: Vec<Duplicate>,
    /// `infAdFisco`.
    pub fiscal_info: Option<String>,
    /// `infCpl`.
    pub additional: Option<String>,
    pub qrcode: Option<String>,
    pub url_chave: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Emitter {
    pub name: Option<String>,
    pub trade_name: Option<String>,
    /// Formatted CNPJ or CPF.
    pub document: Option<String>,
    pub state_registration: Option<String>,
    /// `IEST`.
    pub st_registration: Option<String>,
    /// `IM`.
    pub municipal_registration: Option<String>,
    /// `CRT` with its description (NT 2026.010 §4.2).
    pub tax_regime: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Recipient {
    pub name: Option<String>,
    /// Formatted CNPJ, CPF or foreign id.
    pub document: Option<String>,
    pub state_registration: Option<String>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Address {
    /// Street, number and complement.
    pub street: String,
    pub district: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip_code: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Item {
    pub code: String,
    pub description: String,
    /// `infAdProd`, printed right below the item (Anexo II §3.1.7).
    pub additional: Option<String>,
    pub ncm: String,
    /// Origin followed by CST or CSOSN.
    pub cst: String,
    pub cfop: String,
    pub unit: String,
    pub quantity: String,
    pub unit_price: String,
    pub total: String,
    pub discount: String,
    pub icms_base: String,
    pub icms_value: String,
    pub icms_rate: String,
    pub ipi: Option<ItemTax>,
    pub ibs_cbs: Option<ItemIbsCbs>,
    /// Imposto Seletivo.
    pub selective: Option<ItemTax>,
}

/// Base, rate and value of one tax on one item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemTax {
    pub base: String,
    pub rate: String,
    pub value: String,
}

/// IBS and CBS of one item (NT 2026.010 §4.3). The rates are the effective
/// ones when the XML has `gRed`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ItemIbsCbs {
    pub class: String,
    pub base: Option<String>,
    pub ibs_state_rate: Option<String>,
    pub ibs_state: Option<String>,
    pub ibs_city_rate: Option<String>,
    pub ibs_city: Option<String>,
    pub cbs_rate: Option<String>,
    pub cbs: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Totals {
    pub icms_base: String,
    pub icms: String,
    pub st_base: String,
    pub st: String,
    pub products: String,
    pub freight: String,
    pub insurance: String,
    pub discount: String,
    pub other: String,
    /// Freight, insurance and other charges, as the DANFE NFC-e prints them
    /// (Manual DANFE NFC-e §3.1.3).
    pub surcharge: String,
    pub ipi: String,
    pub total: String,
    pub approximate_taxes: Option<String>,
    pub fcp: Option<String>,
    pub fcp_st: Option<String>,
    pub difal_destination: Option<String>,
    pub fcp_destination: Option<String>,
    pub icms_mono_quantity: Option<String>,
    pub icms_mono: Option<String>,
    pub icms_mono_withheld_quantity: Option<String>,
    pub icms_mono_withheld: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReformTotals {
    pub cbs: Option<String>,
    pub ibs_state: Option<String>,
    pub ibs_city: Option<String>,
    pub selective: Option<String>,
    /// Only when the XML has the monophasic group (`gMono`).
    pub mono: Option<MonoTotals>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MonoTotals {
    pub ibs: Option<String>,
    pub cbs: Option<String>,
    pub ibs_withheld: Option<String>,
    pub cbs_withheld: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Issqn {
    pub services: String,
    pub base: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transport {
    /// `modFrete` with its description.
    pub mode: String,
    pub carrier: Option<Carrier>,
    pub vehicle: Option<Vehicle>,
    pub volumes: Vec<Volume>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Carrier {
    pub name: Option<String>,
    pub document: Option<String>,
    pub state_registration: Option<String>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Vehicle {
    pub plate: Option<String>,
    pub state: Option<String>,
    /// RNTC (código ANTT).
    pub antt: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Volume {
    pub quantity: Option<String>,
    pub kind: Option<String>,
    pub brand: Option<String>,
    pub numbering: Option<String>,
    pub gross_weight: Option<String>,
    pub net_weight: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Payment {
    pub method: String,
    pub amount: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Duplicate {
    pub number: Option<String>,
    pub due_date: Option<String>,
    pub amount: String,
}
