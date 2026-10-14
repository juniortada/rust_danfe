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

/// A rate with two to four decimals: "18.0000" → "18,00", "0.0525" → "0,0525".
#[must_use]
pub fn rate(value: &str) -> String {
    let full = br_number(value, 4);
    let Some((integer, fraction)) = full.split_once(',') else {
        return full;
    };
    let trimmed = fraction.trim_end_matches('0');
    let fraction = if trimmed.len() < 2 {
        &fraction[..2]
    } else {
        trimmed
    };
    format!("{integer},{fraction}")
}

/// "123" → "000.000.123" (Anexo II §3.9).
#[must_use]
pub fn dotted_number(value: &str) -> String {
    let padded = format!("{:0>9}", value.trim());
    if padded.len() != 9 || !padded.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.to_string();
    }
    format!("{}.{}.{}", &padded[..3], &padded[3..6], &padded[6..])
}

/// "2026-10-01T03:58:41-03:00" → "01/10/2026 03:58:41", in the document's own offset.
#[must_use]
pub fn date_time(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value).map_or_else(
        |_| value.to_string(),
        |at| at.format("%d/%m/%Y %H:%M:%S").to_string(),
    )
}

/// `value` in the offset of `reference`: the authorization time comes from
/// the SEFAZ, often in UTC, and is printed in the emitter's local time
/// (Manual DANFE NFC-e §3.1.7).
#[must_use]
pub fn local_date_time(value: &str, reference: &str) -> String {
    match (
        chrono::DateTime::parse_from_rfc3339(value),
        chrono::DateTime::parse_from_rfc3339(reference),
    ) {
        (Ok(at), Ok(local)) => at
            .with_timezone(local.offset())
            .format("%d/%m/%Y %H:%M:%S")
            .to_string(),
        _ => date_time(value),
    }
}

/// The date of a `dhEmi`/`dhSaiEnt`: "01/10/2026".
#[must_use]
pub fn date_part(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value).map_or_else(
        |_| value.to_string(),
        |at| at.format("%d/%m/%Y").to_string(),
    )
}

/// The time of a `dhSaiEnt`: "03:58:41".
#[must_use]
pub fn time_part(value: &str) -> String {
    chrono::DateTime::parse_from_rfc3339(value).map_or_else(
        |_| value.to_string(),
        |at| at.format("%H:%M:%S").to_string(),
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

/// `AA.AAA.AAA/AAAA-99`: the alphanumeric CNPJ keeps its letters (NT Conjunta 2025.001).
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

/// "1131234567" → "(11) 3123-4567"; "11987654321" → "(11) 98765-4321".
#[must_use]
pub fn phone(value: &str) -> String {
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.to_string();
    }
    match value.len() {
        10 => format!("({}) {}-{}", &value[..2], &value[2..6], &value[6..]),
        11 => format!("({}) {}-{}", &value[..2], &value[2..7], &value[7..]),
        _ => value.to_string(),
    }
}

/// `CRT` as printed (MOC 7.0 Anexo I C21; NT 2024.001).
#[must_use]
pub fn tax_regime(code: &str) -> String {
    let label = match code {
        "1" => "SIMPLES NACIONAL",
        "2" => "SIMPLES NACIONAL - EXCESSO DE SUBLIMITE DE RECEITA BRUTA",
        "3" => "REGIME NORMAL",
        "4" => "SIMPLES NACIONAL - MICROEMPREENDEDOR INDIVIDUAL (MEI)",
        _ => return code.to_string(),
    };
    format!("{code} - {label}")
}

/// `tPag` as printed, from the table of payment methods of the Portal
/// Nacional da NF-e (published on 04/03/2026); `xPag` names the "99 - Outros".
#[must_use]
pub fn payment(code: &str, description: Option<String>) -> String {
    let label = match code {
        "01" => "Dinheiro",
        "02" => "Cheque",
        "03" => "Cartão de Crédito",
        "04" => "Cartão de Débito",
        "05" => "Cartão da Loja, Crediário",
        "10" => "Vale Alimentação",
        "11" => "Vale Refeição",
        "12" => "Vale Presente",
        "13" => "Vale Combustível",
        "14" => "Duplicata Mercantil",
        "15" => "Boleto Bancário",
        "16" => "Depósito Bancário",
        "17" => "PIX - Dinâmico",
        "18" => "TED",
        "19" => "Programa de fidelidade, Cashback, Crédito Virtual",
        "20" => "PIX - Estático",
        "21" => "Crédito em Loja",
        "22" => "Pagamento Eletrônico não Informado",
        "23" => "PIX - Automático",
        "24" => "TEF - Book Transfer",
        "90" => "Sem Pagamento",
        "91" => "Pagamento Posterior",
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
        assert_eq!(rate("18.0000"), "18,00");
        assert_eq!(rate("0.0525"), "0,0525");
        assert_eq!(rate("0.9"), "0,90");
        assert_eq!(dotted_number("123"), "000.000.123");
    }

    #[test]
    fn formats_documents_and_keys() {
        assert_eq!(cnpj("11222333000181"), "11.222.333/0001-81");
        assert_eq!(cnpj("12ABC34501DE35"), "12.ABC.345/01DE-35");
        assert_eq!(cpf("52998224725"), "529.982.247-25");
        assert_eq!(zip_code("01310100"), "01310-100");
        assert_eq!(phone("1131234567"), "(11) 3123-4567");
        assert_eq!(phone("11987654321"), "(11) 98765-4321");
        assert_eq!(grouped_key("12345678"), "1234 5678");
    }

    #[test]
    fn formats_dates() {
        assert_eq!(date("2026-10-01"), "01/10/2026");
        assert_eq!(
            date_time("2026-10-01T03:58:41-03:00"),
            "01/10/2026 03:58:41"
        );
        assert_eq!(date_part("2026-10-01T03:58:41-03:00"), "01/10/2026");
        assert_eq!(time_part("2026-10-01T03:58:41-03:00"), "03:58:41");
        assert_eq!(
            local_date_time("2026-10-01T06:58:42+00:00", "2026-10-01T03:58:41-03:00"),
            "01/10/2026 03:58:42"
        );
    }

    #[test]
    fn names_codes() {
        assert_eq!(payment("17", None), "PIX - Dinâmico");
        assert_eq!(payment("20", None), "PIX - Estático");
        assert_eq!(payment("99", Some("Cashback".to_string())), "Cashback");
        assert_eq!(payment("01", Some("ignorado".to_string())), "Dinheiro");
        assert_eq!(freight("1"), "1 - Por conta do Destinatário (FOB)");
        assert_eq!(freight(""), "9 - Sem Ocorrência de Transporte");
        assert_eq!(
            tax_regime("4"),
            "4 - SIMPLES NACIONAL - MICROEMPREENDEDOR INDIVIDUAL (MEI)"
        );
    }
}
