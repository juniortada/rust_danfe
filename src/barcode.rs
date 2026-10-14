//! The Code 128 barcode of the access key, drawn as SVG.
//!
//! A numeric key is all set C (MOC 7.0 Anexo II §2). A key with an
//! alphanumeric CNPJ starts in set C and switches to set A for the letters
//! and back (NT Conjunta 2025.001 §6).

use std::fmt::Write;

/// Bar widths of every Code 128 symbol 0–106 (106 is STOP, 7 bars).
const CODE128: [&str; 107] = [
    "212222", "222122", "222221", "121223", "121322", "131222", "122213", "122312", "132212",
    "221213", "221312", "231212", "112232", "122132", "122231", "113222", "123122", "123221",
    "223211", "221132", "221231", "213212", "223112", "312131", "311222", "321122", "321221",
    "312212", "322112", "322211", "212123", "212321", "232121", "111323", "131123", "131321",
    "112313", "132113", "132311", "211313", "231113", "231311", "112133", "112331", "132131",
    "113123", "113321", "133121", "313121", "211331", "231131", "213113", "213311", "213131",
    "311123", "311321", "331121", "312113", "312311", "332111", "314111", "221411", "431111",
    "111224", "111422", "121124", "121421", "141122", "141221", "112214", "112412", "122114",
    "122411", "142112", "142211", "241211", "221114", "413111", "241112", "134111", "111242",
    "121142", "121241", "114212", "124112", "124211", "411212", "421112", "421211", "212141",
    "214121", "412121", "111143", "111341", "131141", "114113", "114311", "411113", "411311",
    "113141", "114131", "311141", "411131", "211412", "211214", "211232", "2331112",
];
const CODE_C: usize = 99;
const CODE_A: usize = 101;
const START_C: usize = 105;
const STOP: usize = 106;
/// The light margin on each side, in modules (Anexo II §2).
const QUIET_ZONE: usize = 10;

/// How many digits start at `from`.
fn digit_run(bytes: &[u8], from: usize) -> usize {
    bytes[from..]
        .iter()
        .take_while(|byte| byte.is_ascii_digit())
        .count()
}

/// Symbol values of the key, with the check symbol and STOP; `None` for a
/// character Code 128 sets A and C cannot carry.
#[must_use]
pub fn code128_symbols(text: &str) -> Option<Vec<usize>> {
    let bytes = text.as_bytes();
    if bytes.is_empty() || !bytes.iter().all(|byte| (b' '..=b'_').contains(byte)) {
        return None;
    }
    let mut symbols = vec![START_C];
    let mut in_c = true;
    let mut index = 0;
    while index < bytes.len() {
        let run = digit_run(bytes, index);
        if in_c && run >= 2 {
            symbols
                .push(usize::from(bytes[index] - b'0') * 10 + usize::from(bytes[index + 1] - b'0'));
            index += 2;
        } else if !in_c && run >= 2 && run.is_multiple_of(2) {
            symbols.push(CODE_C);
            in_c = true;
        } else {
            if in_c {
                symbols.push(CODE_A);
                in_c = false;
            }
            symbols.push(usize::from(bytes[index] - b' '));
            index += 1;
        }
    }
    let check = symbols
        .iter()
        .enumerate()
        .map(|(position, symbol)| position.max(1) * symbol)
        .sum::<usize>()
        % 103;
    symbols.push(check);
    symbols.push(STOP);
    Some(symbols)
}

/// Modules (`true` = bar) of the key, with the quiet zone on both sides.
#[must_use]
pub fn code128_modules(text: &str) -> Option<Vec<bool>> {
    let mut modules = vec![false; QUIET_ZONE];
    for symbol in code128_symbols(text)? {
        for (index, width) in CODE128[symbol].bytes().enumerate() {
            modules.extend(std::iter::repeat_n(
                index % 2 == 0,
                usize::from(width - b'0'),
            ));
        }
    }
    modules.extend(std::iter::repeat_n(false, QUIET_ZONE));
    Some(modules)
}

/// An SVG of the modules, one unit wide each, scaled by the template.
#[must_use]
pub fn svg(modules: &[bool], height: u32) -> Vec<u8> {
    let mut bars = String::new();
    let mut x = 0;
    while x < modules.len() {
        if modules[x] {
            let start = x;
            while x < modules.len() && modules[x] {
                x += 1;
            }
            let _ = write!(
                bars,
                r#"<rect x="{start}" y="0" width="{}" height="{height}"/>"#,
                x - start
            );
        } else {
            x += 1;
        }
    }
    format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {} {height}" preserveAspectRatio="none"><g fill="black">{bars}</g></svg>"#,
        modules.len()
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_numeric_key_is_all_set_c() {
        let key = "35260911444777000161550010000012341123456787";
        let symbols = code128_symbols(key).unwrap();
        assert_eq!(symbols[0], START_C);
        assert_eq!(symbols.len(), 1 + 22 + 2);
        assert_eq!(*symbols.last().unwrap(), STOP);
        assert_eq!(
            code128_modules(key).unwrap().len(),
            2 * QUIET_ZONE + 11 * 24 + 13
        );
    }

    #[test]
    fn follows_the_example_of_the_nt_for_letters() {
        // NT Conjunta 2025.001 §6: 5225AB83 → START C, 52, 25, CODE A, 'A', 'B', CODE C, 83, DV 30.
        assert_eq!(
            code128_symbols("5225AB83").unwrap(),
            vec![START_C, 52, 25, CODE_A, 33, 34, CODE_C, 83, 30, STOP]
        );
    }

    #[test]
    fn an_odd_digit_run_after_letters_takes_one_digit_in_set_a() {
        let symbols = code128_symbols("12A345").unwrap();
        assert_eq!(&symbols[..6], &[START_C, 12, CODE_A, 33, 19, CODE_C]);
        assert_eq!(symbols[6], 45);
    }

    #[test]
    fn refuses_what_sets_a_and_c_cannot_carry() {
        assert!(code128_symbols("ação").is_none());
        assert!(code128_symbols("").is_none());
        assert!(
            String::from_utf8(svg(&code128_modules("12AB").unwrap(), 40))
                .unwrap()
                .starts_with("<svg")
        );
    }
}
