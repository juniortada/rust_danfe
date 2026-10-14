# danfe

Gera o DANFE (NF-e, modelo 55) e o DANFE NFC-e (modelo 65) em PDF a partir do XML, sem banco, rede nem relógio:
o mesmo XML gera sempre o mesmo PDF.

```toml
[dependencies]
danfe = "0.1"
```

```rust
let xml = std::fs::read_to_string("nota.xml")?;

// O layout que o tpImp pede: NF-e em A4 (retrato ou paisagem), NFC-e e DANFE Simplificado Tipo 2 em bobina de 80 mm.
let pdf = danfe::render_default(&xml)?;

// Ou um layout escolhido.
let pdf = danfe::render(&xml, danfe::Layout::A4Portrait)?;

// Com logotipo, fontes próprias ou a via do estabelecimento.
let options = danfe::Options {
    logo: Some(std::fs::read("logo.png")?),
    ..danfe::Options::default()
};
let pdf = danfe::render_with(&xml, danfe::Layout::A4Portrait, &options)?;

// De arquivo para arquivo.
danfe::generate("nota.xml", "nota.pdf", danfe::Layout::A4Portrait)?;

// Só os dados que o DANFE imprime.
let nota = danfe::parse(&xml)?;
```

Pela linha de comando: `cargo run --example generate -- nota.xml nota.pdf [a4|a4-paisagem|80mm|58mm]`.

| Documento | Layouts |
|---|---|
| NF-e, `tpImp` 1 ou 2 (DANFE) | `A4Portrait`, `A4Landscape` |
| NF-e, `tpImp` 6 (DANFE Simplificado Tipo 2) | `Roll80mm`, `Roll58mm`, `A4Portrait` |
| NFC-e (DANFE NFC-e) | `Roll80mm`, `Roll58mm`, `A4Portrait` |

Outro layout responde `Error::UnsupportedLayout`; o DANFE Simplificado (`tpImp=3`) e a NF-e sem DANFE (`tpImp=0`)
respondem `Error::UnsupportedDocument`.

## Normas

| Documento | O que a crate atende |
|---|---|
| MOC 7.0, Anexo II | Leiaute A4 retrato e paisagem, tamanhos mínimos de fonte, margens, folhas adicionais com "FOLHA x/y", Code 128 da chave, "SEM VALOR FISCAL" em homologação |
| NT 2026.010 (DANFE da Reforma Tributária) | Bloco "Total do IBS/CBS/IS", CRT do emitente e bases, alíquotas (efetiva quando há `gRed`) e valores de IBS, CBS e IS por item |
| NT Conjunta 2025.001 e NT 2026.004 | Chave com CNPJ alfanumérico em Code 128 híbrido (C/A), com 14 cm de largura |
| Manual do DANFE NFC-e e QR Code v6.0 | Divisões I a IX em 80 mm, 58 mm e A4, consumidor, contingência off-line, homologação, QR Code nível M com zona de silêncio |
| NT 2026.003 e NT 2026.002 | DANFE Simplificado Tipo 2 (`tpImp=6`), com a divisão III-A do IBS/CBS/IS |
| Tabela de meios de pagamento (04/03/2026) | Descrição de todos os códigos de `tPag` |

O Anexo II pede Times New Roman ou Courier New. A crate embute Liberation Serif e Liberation Mono (SIL OFL 1.1, com
as mesmas métricas); quem tiver a licença das originais passa os arquivos em `Options::fonts` e os templates as usam.

## Licença

MIT ou Apache-2.0, à sua escolha. As fontes Liberation seguem a SIL Open Font License 1.1 (`fonts/OFL.txt`).
