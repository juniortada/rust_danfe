//! `cargo run --example generate -- nota.xml nota.pdf [a4|a4-paisagem|80mm|58mm]`
//!
//! Without the layout, the one the document calls for.

use std::process::ExitCode;

fn layout(name: &str) -> Option<danfe::Layout> {
    match name {
        "a4" => Some(danfe::Layout::A4Portrait),
        "a4-paisagem" => Some(danfe::Layout::A4Landscape),
        "80mm" => Some(danfe::Layout::Roll80mm),
        "58mm" => Some(danfe::Layout::Roll58mm),
        _ => None,
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let (Some(xml_path), Some(pdf_path)) = (args.first(), args.get(1)) else {
        eprintln!("uso: generate nota.xml nota.pdf [a4|a4-paisagem|80mm|58mm]");
        return ExitCode::FAILURE;
    };
    let result = if let Some(name) = args.get(2) {
        let Some(layout) = layout(name) else {
            eprintln!("layout desconhecido: {name}");
            return ExitCode::FAILURE;
        };
        danfe::generate(xml_path, pdf_path, layout)
    } else {
        std::fs::read_to_string(xml_path)
            .map_err(|err| danfe::Error::Render(err.to_string()))
            .and_then(|xml| danfe::render_default(&xml))
            .and_then(|pdf| {
                std::fs::write(pdf_path, pdf).map_err(|err| danfe::Error::Render(err.to_string()))
            })
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}
