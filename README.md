# danfe

Gera o DANFE (NF-e, modelo 55) e o DANFE NFC-e (modelo 65) em PDF a partir do XML autorizado, sem banco, rede nem
relógio: o mesmo XML gera sempre o mesmo PDF.

```toml
[dependencies]
danfe = "0.1"
```

```rust
let xml = std::fs::read_to_string("nota.xml")?;

// O layout que o documento pede: NF-e em A4, NFC-e em bobina de 80 mm.
let pdf = danfe::render_default(&xml)?;

// Ou um layout escolhido.
let pdf = danfe::render(&xml, danfe::Layout::A4Portrait)?;

// De arquivo para arquivo.
danfe::generate("nota.xml", "nota.pdf", danfe::Layout::A4Portrait)?;

// Só os dados que o DANFE imprime.
let nota = danfe::parse(&xml)?;
```

## Licença

MIT ou Apache-2.0, à sua escolha.
