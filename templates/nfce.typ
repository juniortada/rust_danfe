// DANFE NFC-e: Manual de Especificações Técnicas do DANFE NFC-e e QR Code, versão 6.0.
#let data = json("data.json")
#let e = data.emitter
#let narrow = data.paper == "58mm"
#let mono = ("Courier New", "Liberation Mono")
#let shown(value) = if value == none { "" } else { value }

// Manual §3.1.8 e MOC 7.0 Anexo II §3: em homologação, "SEM VALOR FISCAL" também em marca d'água.
#let watermark = if data.homologation {
  rotate(-60deg, align(center, text(fill: rgb(200, 0, 0, 45), weight: "bold", size: if narrow { 16pt } else { 22pt })[
    SEM VALOR FISCAL \ HOMOLOGAÇÃO
  ]))
}

// Manual §3.3: papel de no mínimo 56 mm e margens laterais de no mínimo 2 mm.
#set page(width: if narrow { 58mm } else { 80mm }, height: auto, margin: (x: 3mm, y: 4mm), background: watermark)
#set text(font: mono, size: if narrow { 6.5pt } else { 7.5pt }, lang: "pt", region: "br", hyphenate: false)
#set par(leading: 0.35em, spacing: 0.4em, justify: false)
#let rule = line(length: 100%, stroke: 0.5pt)
#let contingency = align(center, text(weight: "bold")[EMITIDA EM CONTINGÊNCIA \ Pendente de autorização])

// Divisão I.
#align(center, {
  if data.logo != none { image(data.logo, height: 12mm, fit: "contain"); linebreak() }
  let label = if shown(e.document).len() == 14 { "CPF" } else { "CNPJ" }
  [#label: #shown(e.document) #if e.state_registration != none [ IE: #e.state_registration]]
  linebreak()
  text(weight: "bold", shown(e.name))
  let a = e.address
  if a != none [ \ #a.street, #shown(a.district), #shown(a.city) - #shown(a.state)]
  linebreak()
  [Documento Auxiliar da Nota Fiscal de Consumidor Eletrônica]
})
// Divisão VIII: contingência em destaque, abaixo do cabeçalho.
#if data.offline { rule; contingency }
#rule

// Divisão II.
#if narrow {
  for (index, i) in data.items.enumerate() [
    #(index + 1) #i.code #i.description \
    #align(right)[#i.quantity #i.unit x #i.unit_price = #text(weight: "bold", i.total)]
  ]
} else {
  table(
    columns: (auto, 1fr, auto, auto, auto), stroke: none, inset: (x: 1pt, y: 1.2pt),
    align: (left, left, right, right, right),
    [*Código*], [*Descrição*], [*Qtde Un*], [*Vl unit*], [*Vl total*],
    ..data.items.map(i => (i.code, i.description, [#i.quantity #i.unit], i.unit_price, i.total)).flatten(),
  )
}
#rule

// Divisão III.
#let t = data.totals
#table(
  columns: (1fr, auto), stroke: none, inset: (x: 0pt, y: 1.2pt),
  [Qtde. total de itens], align(right)[#data.item_count],
  [Valor total R\$], align(right)[#t.products],
  ..if t.surcharge != "0,00" { ([Acréscimos R\$], align(right)[#t.surcharge]) } else { () },
  ..if t.discount != "0,00" { ([Desconto R\$], align(right)[#t.discount]) } else { () },
  ..if t.surcharge != "0,00" or t.discount != "0,00" { ([*Valor a pagar R\$*], align(right)[*#t.total*]) } else { () },
  [*FORMA PAGAMENTO*], align(right)[*VALOR PAGO R\$*],
  ..data.payments.map(p => (p.method, align(right)[#p.amount])).flatten(),
  ..if data.change != none { ([Troco R\$], align(right)[#data.change]) } else { () },
)
#rule

// Divisão IV.
#align(center)[
  Consulte pela Chave de Acesso em \
  #shown(data.url_chave) \
  #text(weight: "bold", size: if narrow { 6pt } else { 6.4pt }, data.access_key)
]
#rule

// Divisões VI e VII.
#let r = data.recipient
#align(center, {
  text(weight: "bold", data.consumer)
  if r != none and r.name != none [ \ #r.name]
  if r != none and r.address != none [ \ #r.address.street, #shown(r.address.district), #shown(r.address.city) - #shown(r.address.state)]
  v(3pt)
  text(weight: "bold")[NFC-e nº #data.number_padded Série #data.series #data.issued_at]
  if data.establishment_copy [ \ Via do Estabelecimento]
  if data.protocol != none [ \ Protocolo de autorização: #data.protocol]
})
// Divisão VIII: contingência também abaixo da identificação.
#if data.offline { v(2pt); contingency }

// Divisão V (§3.4): no mínimo 25 mm, com a zona de silêncio na própria imagem.
#v(3pt)
#align(center, image("qrcode.svg", width: if narrow { 28mm } else { 32mm }))

// Divisão VIII.
#if data.homologation or data.fiscal_info != none {
  rule
  if data.homologation { align(center, text(weight: "bold")[EMITIDA EM AMBIENTE DE HOMOLOGAÇÃO – SEM VALOR FISCAL]) }
  if data.fiscal_info != none { align(center, data.fiscal_info) }
}

// Divisão IX.
#if data.additional != none or t.approximate_taxes != none {
  rule
  set text(size: if narrow { 6pt } else { 6.5pt })
  if t.approximate_taxes != none [Tributos totais incidentes (Lei Federal 12.741/2012): R\$ #t.approximate_taxes \ ]
  shown(data.additional)
}
