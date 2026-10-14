# Changelog

## [Não publicado]

- Leitura do XML e geração do DANFE da NF-e (A4 retrato) e do DANFE NFC-e (bobina 80 mm), extraídas do backend do
  Falcão ERP.
- Conformidade com o MOC 7.0 Anexo II: fontes, tamanhos mínimos, margens, folhas adicionais com "FOLHA x/y",
  canhoto no rodapé, `infAdFisco`, transportador e volumes, ISSQN e duplicatas.
- NT 2026.010: bloco "Total do IBS/CBS/IS", CRT e tributos da Reforma Tributária por item.
- CNPJ alfanumérico: Code 128 híbrido C/A com zona de silêncio.
- DANFE NFC-e conforme o Manual v6.0: consumidor, contingência off-line, homologação e QR Code nível M.
- Marca d'água "SEM VALOR FISCAL" em homologação.
- `Options`: fontes próprias, logotipo e via do estabelecimento.
- Layouts: NF-e em A4 paisagem; DANFE NFC-e em bobina de 58 mm e em A4; DANFE Simplificado Tipo 2 (`tpImp=6`).
- `default_layout` público: o layout que o `tpImp` pede.
