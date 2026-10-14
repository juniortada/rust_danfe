//! The Code 128 barcode of the access key (MOC 7.0 Anexo II §2), drawn as SVG.

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
const START_B: usize = 104;
const START_C: usize = 105;
const STOP: usize = 106;

/// Code 128 symbol values: set C for an even-length all-digit text (the
/// access key), set B otherwise; with check symbol and stop.
#[must_use]
pub fn code128_symbols(text: &str) -> Option<Vec<usize>> {
    let digits_only = !text.is_empty()
        && text.len().is_multiple_of(2)
        && text.bytes().all(|b| b.is_ascii_digit());
    let mut symbols = Vec::new();
    if digits_only {
        symbols.push(START_C);
        for pair in text.as_bytes().chunks(2) {
            symbols.push(usize::from(pair[0] - b'0') * 10 + usize::from(pair[1] - b'0'));
        }
    } else {
        symbols.push(START_B);
        for byte in text.bytes() {
            if !(32..=126).contains(&byte) {
                return None;
            }
            symbols.push(usize::from(byte - 32));
        }
    }
    let check = symbols
        .iter()
        .enumerate()
        .map(|(index, symbol)| index.max(1) * symbol)
        .sum::<usize>()
        % 103;
    symbols.push(check);
    symbols.push(STOP);
    Some(symbols)
}

/// Modules of a Code 128 barcode.
#[must_use]
pub fn code128_modules(text: &str) -> Option<Vec<bool>> {
    let mut modules = Vec::new();
    for symbol in code128_symbols(text)? {
        for (index, width) in CODE128[symbol].bytes().enumerate() {
            modules.extend(std::iter::repeat_n(
                index % 2 == 0,
                usize::from(width - b'0'),
            ));
        }
    }
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
    fn uses_set_c_for_access_keys_and_checks_the_sum() {
        let key = "35260911444777000161550010000012341123456787";
        let symbols = code128_symbols(key).unwrap();
        assert_eq!(symbols[0], START_C);
        assert_eq!(symbols.len(), 1 + 22 + 2);
        assert_eq!(*symbols.last().unwrap(), STOP);
        assert_eq!(code128_modules(key).unwrap().len(), 11 * 24 + 13);
    }

    #[test]
    fn draws_the_modules_as_svg() {
        let svg = String::from_utf8(svg(&code128_modules("AB-1").unwrap(), 40)).unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(code128_symbols("ação").is_none());
    }
}
