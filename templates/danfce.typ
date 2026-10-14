#let data = json("data.json")
#set page(width: 80mm, height: auto, margin: (x: 3mm, y: 4mm))
#set text(font: "DejaVu Sans Mono", size: 7pt, lang: "pt")
#set par(spacing: 0.4em)
#let e = data.emitter
#let a = e.address

#align(center)[
  #text(weight: "bold")[#if e.trade_name != none { e.trade_name } else { e.name }] \
  #e.name \
  CNPJ #e.document #h(4pt) IE #e.state_registration \
  #a.street, #a.district, #a.city - #a.state \
  #v(2pt)
  #text(weight: "bold")[DANFE NFC-e] - Documento Auxiliar \
  da Nota Fiscal de Consumidor Eletrônica
]
#if data.homologation [#align(center)[#text(weight: "bold")[EMITIDA EM AMBIENTE DE HOMOLOGAÇÃO - SEM VALOR FISCAL]]]
#if data.offline [#align(center)[#text(weight: "bold")[EMITIDA EM CONTINGÊNCIA - pendente de autorização]]]
#line(length: 100%, stroke: 0.5pt)
#table(
  columns: (auto, 1fr, auto, auto, auto),
  stroke: none, inset: (x: 1pt, y: 1.2pt),
  align: (left, left, right, right, right),
  [*Cód.*], [*Descrição*], [*Qtde Un*], [*V. unit.*], [*V. total*],
  ..data.items.map(i => (i.code, i.description, [#i.quantity #i.unit], i.unit_price, i.total)).flatten()
)
#line(length: 100%, stroke: 0.5pt)
#table(
  columns: (1fr, auto), stroke: none, inset: (x: 0pt, y: 1.2pt),
  [Qtd. total de itens], align(right)[#data.item_count],
  [Valor total R\$], align(right)[#data.totals.products],
  ..if data.totals.discount != "0,00" { ([Descontos R\$], align(right)[#data.totals.discount]) } else { () },
  ..if data.totals.other != "0,00" { ([Acréscimos R\$], align(right)[#data.totals.other]) } else { () },
  [*Valor a pagar R\$*], align(right)[*#data.totals.total*],
  [*Forma de pagamento*], align(right)[*Valor pago R\$*],
  ..data.payments.map(p => (p.method, align(right)[#p.amount])).flatten(),
  ..if data.change != none { ([Troco R\$], align(right)[#data.change]) } else { () },
)
#line(length: 100%, stroke: 0.5pt)
#align(center)[
  Consulte pela chave de acesso em \
  #if data.url_chave != none [#data.url_chave \ ]
  #text(weight: "bold")[#data.access_key]
]
#line(length: 100%, stroke: 0.5pt)
#align(center)[
  #if data.recipient != none [CONSUMIDOR - #data.recipient.document #if data.recipient.name != none [- #data.recipient.name]] else [CONSUMIDOR NÃO IDENTIFICADO] \
  #v(2pt)
  #text(weight: "bold")[NFC-e nº #data.number_padded Série #data.series #data.issued_at] \
  Protocolo de autorização: #data.protocol \
  #v(3pt)
  #image("qrcode.svg", width: 32mm)
  #if data.totals.approximate_taxes != none [Tributos totais incidentes (Lei Federal 12.741/2012): R\$ #data.totals.approximate_taxes]
  #if data.additional != none [#text(size: 6pt)[#data.additional]]
]
