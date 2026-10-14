//! What the DANFE prints, already formatted the way it is printed.
//!
//! Serialized as the templates' `data.json`; field names follow the template
//! variables.

use serde::Serialize;

/// A NF-e or NFC-e read from its XML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Invoice {
    /// `mod`: "55" (NF-e) or "65" (NFC-e).
    pub model: String,
    /// `tpAmb` = 2.
    pub homologation: bool,
    /// `tpEmis` = 9 (off-line contingency).
    pub offline: bool,
    pub number: String,
    pub number_padded: String,
    pub series: Option<String>,
    pub issued_at: String,
    pub operation: Option<String>,
    /// `tpNF`: "0" (entrada) or "1" (saída).
    pub direction: String,
    /// The key in eleven blocks of four.
    pub access_key: String,
    /// The key as in the XML, for the barcode.
    #[serde(skip)]
    pub key: String,
    /// Protocol number and authorization time.
    pub protocol: String,
    pub emitter: Emitter,
    pub recipient: Option<Recipient>,
    pub items: Vec<Item>,
    pub item_count: usize,
    pub totals: Totals,
    pub freight_mode: String,
    pub payments: Vec<Payment>,
    pub change: Option<String>,
    pub duplicates: Vec<Duplicate>,
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
    pub ipi: String,
    pub total: String,
    pub approximate_taxes: Option<String>,
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
