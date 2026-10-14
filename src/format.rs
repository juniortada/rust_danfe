//! How the DANFE writes numbers, documents, dates and codes.

use rust_decimal::Decimal;

/// "1234.5" → "1.234,50" with `places` decimals; anything unreadable stays as is.
#[must_use]
pub fn br_number(value: &str, places: u32) -> String {
    let Ok(number) = value.trim().parse::<Decimal>() else {
        return value.to_string();
    };
    let width = places as usize;
    let fixed = format!("{:.width$}", number.round_dp(places));
    let (sign, digits) = fixed
        .strip_prefix('-')
        .map_or(("", fixed.as_str()), |rest| ("-", rest));
    let (integer, fraction) = digits.split_once('.').unwrap_or((digits, ""));
    let mut grouped = String::new();
    for (index, digit) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index) % 3 == 0 {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    if fraction.is_empty() {
        format!("{sign}{grouped}")
    } else {
        format!("{sign}{grouped},{fraction}")
    }
}

/// "2026-10-01T03:58:41-03:00" → "01/10/2026 03:58:41", in the document's own offset.
#[must_use]
pub fn date_time(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value).map_or_else(
        |_| value.to_string(),
        |at| at.format("%d/%m/%Y %H:%M:%S").to_string(),
    )
}

/// "2026-10-01" → "01/10/2026".
#[must_use]
pub fn date(value: &str) -> String {
    chrono::NaiveDate::parse_from_str(value, "%Y-%m-%d").map_or_else(
        |_| value.to_string(),
        |day| day.format("%d/%m/%Y").to_string(),
    )
}

/// The access key in eleven blocks of four.
#[must_use]
pub fn grouped_key(key: &str) -> String {
    key.as_bytes()
        .chunks(4)
        .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
        .collect::<Vec<_>>()
        .join(" ")
}

#[must_use]
pub fn cnpj(value: &str) -> String {
    if value.len() != 14 || !value.is_ascii() {
        return value.to_string();
    }
    format!(
        "{}.{}.{}/{}-{}",
        &value[..2],
        &value[2..5],
        &value[5..8],
        &value[8..12],
        &value[12..]
    )
}

#[must_use]
pub fn cpf(value: &str) -> String {
    if value.len() != 11 || !value.is_ascii() {
        return value.to_string();
    }
    format!(
        "{}.{}.{}-{}",
        &value[..3],
        &value[3..6],
        &value[6..9],
        &value[9..]
    )
}

/// "01310100" → "01310-100".
#[must_use]
pub fn zip_code(value: &str) -> String {
    if value.len() == 8 && value.is_ascii() {
        format!("{}-{}", &value[..5], &value[5..])
    } else {
        value.to_string()
    }
}

/// `tPag` as printed; `xPag` only names the "99 - Outros" payments.
#[must_use]
pub fn payment(code: &str, description: Option<String>) -> String {
    let label = match code {
        "01" => "Dinheiro",
        "02" => "Cheque",
        "03" => "Cartão de Crédito",
        "04" => "Cartão de Débito",
        "05" => "Crédito Loja",
        "10" => "Vale Alimentação",
        "11" => "Vale Refeição",
        "12" => "Vale Presente",
        "13" => "Vale Combustível",
        "15" => "Boleto Bancário",
        "16" => "Depósito Bancário",
        "17" => "PIX",
        "18" => "Transferência bancária",
        "19" => "Programa de fidelidade",
        "90" => "Sem pagamento",
        _ => "Outros",
    };
    description
        .filter(|_| code == "99")
        .unwrap_or_else(|| label.to_string())
}

/// `modFrete` as printed (Anexo II §3.1.10).
#[must_use]
pub const fn freight(code: &str) -> &'static str {
    match code.as_bytes() {
        b"0" => "0 - Por conta do Remetente (CIF)",
        b"1" => "1 - Por conta do Destinatário (FOB)",
        b"2" => "2 - Por conta de Terceiros",
        b"3" => "3 - Próprio por conta do Remetente",
        b"4" => "4 - Próprio por conta do Destinatário",
        _ => "9 - Sem Ocorrência de Transporte",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_numbers_the_brazilian_way() {
        assert_eq!(br_number("1234567.5", 2), "1.234.567,50");
        assert_eq!(br_number("3.0000", 4), "3,0000");
        assert_eq!(br_number("12", 0), "12");
        assert_eq!(br_number("-0.5", 2), "-0,50");
        assert_eq!(br_number("abc", 2), "abc");
    }

    #[test]
    fn formats_documents_and_keys() {
        assert_eq!(cnpj("11222333000181"), "11.222.333/0001-81");
        assert_eq!(cpf("52998224725"), "529.982.247-25");
        assert_eq!(zip_code("01310100"), "01310-100");
        assert_eq!(grouped_key("12345678"), "1234 5678");
        assert_eq!(date("2026-10-01"), "01/10/2026");
        assert_eq!(
            date_time("2026-10-01T03:58:41-03:00"),
            "01/10/2026 03:58:41"
        );
    }

    #[test]
    fn names_payments_and_freight() {
        assert_eq!(payment("17", None), "PIX");
        assert_eq!(payment("99", Some("Cashback".to_string())), "Cashback");
        assert_eq!(payment("01", Some("ignorado".to_string())), "Dinheiro");
        assert_eq!(freight("1"), "1 - Por conta do Destinatário (FOB)");
        assert_eq!(freight(""), "9 - Sem Ocorrência de Transporte");
    }
}
